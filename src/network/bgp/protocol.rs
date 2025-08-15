use crate::network::bgp::{BGPError, BGPOrigin, BGPSession, RouteEntry};
use crate::node::NodeTier;
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BGPMessage {
    pub message_type: BGPMessageType,
    pub asn: u32,
    pub router_id: IpAddr,
    pub routes: Vec<BGPRoute>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BGPMessageType {
    Open,
    Update,
    Keepalive,
    Notification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BGPRoute {
    pub network: IpNet,
    pub next_hop: IpAddr,
    pub as_path: Vec<u32>,
    pub origin: BGPOrigin,
    pub local_pref: u32,
    pub med: u32,
}

pub struct BGPProtocol {
    local_asn: u32,
    router_id: IpAddr,
    tier: NodeTier,
}

impl BGPProtocol {
    pub fn new(local_asn: u32, router_id: IpAddr, tier: NodeTier) -> Self {
        BGPProtocol {
            local_asn,
            router_id,
            tier,
        }
    }

    pub async fn start_server(&self, listen_addr: SocketAddr) -> Result<(), BGPError> {
        let listener = TcpListener::bind(listen_addr).await?;
        tracing::info!("BGP server listening on {}", listen_addr);

        let local_asn = self.local_asn;
        let router_id = self.router_id;
        let tier = self.tier.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        tracing::info!("BGP connection from {}", peer_addr);

                        let tier = tier.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_bgp_connection(
                                stream, peer_addr, local_asn, router_id, tier,
                            )
                            .await
                            {
                                tracing::error!("BGP connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("BGP listener error: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn connect_to_peer(
        &self,
        peer_addr: SocketAddr,
        peer_asn: u32,
    ) -> Result<BGPSession, BGPError> {
        tracing::info!("Connecting to BGP peer {} (ASN {})", peer_addr, peer_asn);

        let mut stream = TcpStream::connect(peer_addr).await?;

        // Send BGP OPEN message
        let open_msg = BGPMessage {
            message_type: BGPMessageType::Open,
            asn: self.local_asn,
            router_id: self.router_id,
            routes: vec![],
            timestamp: chrono::Utc::now(),
        };

        self.send_message(&mut stream, &open_msg).await?;

        // Receive BGP OPEN response
        let response = self.receive_message(&mut stream).await?;
        match response.message_type {
            BGPMessageType::Open => {
                tracing::info!("BGP session established with ASN {}", response.asn);

                // Create BGP session
                let session = BGPSession::new(
                    self.local_asn,
                    response.asn,
                    peer_addr.ip(),
                    std::sync::Arc::new(tokio::sync::RwLock::new(
                        crate::network::bgp::RouteTable::new(),
                    )),
                );

                Ok(session)
            }
            _ => Err(BGPError::Protocol("Invalid BGP OPEN response".to_string())),
        }
    }

    async fn handle_bgp_connection(
        mut stream: TcpStream,
        peer_addr: SocketAddr,
        local_asn: u32,
        router_id: IpAddr,
        tier: NodeTier,
    ) -> Result<(), BGPError> {
        // Receive BGP OPEN message
        let protocol = BGPProtocol::new(local_asn, router_id, tier);
        let open_msg = protocol.receive_message(&mut stream).await?;

        match open_msg.message_type {
            BGPMessageType::Open => {
                tracing::info!(
                    "Received BGP OPEN from ASN {} at {}",
                    open_msg.asn,
                    peer_addr
                );

                // Send BGP OPEN response
                let response = BGPMessage {
                    message_type: BGPMessageType::Open,
                    asn: local_asn,
                    router_id,
                    routes: vec![],
                    timestamp: chrono::Utc::now(),
                };

                protocol.send_message(&mut stream, &response).await?;

                // Start keepalive loop
                protocol.keepalive_loop(stream, open_msg.asn).await?;
            }
            _ => {
                return Err(BGPError::Protocol("Expected BGP OPEN message".to_string()));
            }
        }

        Ok(())
    }

    async fn keepalive_loop(&self, mut stream: TcpStream, peer_asn: u32) -> Result<(), BGPError> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Send keepalive
                    let keepalive = BGPMessage {
                        message_type: BGPMessageType::Keepalive,
                        asn: self.local_asn,
                        router_id: self.router_id,
                        routes: vec![],
                        timestamp: chrono::Utc::now(),
                    };

                    if let Err(e) = self.send_message(&mut stream, &keepalive).await {
                        tracing::error!("Failed to send keepalive to ASN {}: {}", peer_asn, e);
                        break;
                    }
                }

                result = self.receive_message(&mut stream) => {
                    match result {
                        Ok(msg) => {
                            self.handle_bgp_message(msg, peer_asn).await?;
                        }
                        Err(e) => {
                            tracing::error!("BGP message error from ASN {}: {}", peer_asn, e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_bgp_message(&self, msg: BGPMessage, peer_asn: u32) -> Result<(), BGPError> {
        match msg.message_type {
            BGPMessageType::Update => {
                tracing::info!(
                    "Received BGP UPDATE from ASN {} with {} routes",
                    peer_asn,
                    msg.routes.len()
                );
                for route in &msg.routes {
                    tracing::debug!(
                        "  Route: {} via {} (AS path: {:?})",
                        route.network,
                        route.next_hop,
                        route.as_path
                    );
                }
            }
            BGPMessageType::Keepalive => {
                tracing::debug!("Received BGP KEEPALIVE from ASN {}", peer_asn);
            }
            BGPMessageType::Notification => {
                tracing::warn!("Received BGP NOTIFICATION from ASN {}", peer_asn);
            }
            _ => {
                tracing::warn!("Unexpected BGP message type from ASN {}", peer_asn);
            }
        }

        Ok(())
    }

    async fn send_message(&self, stream: &mut TcpStream, msg: &BGPMessage) -> Result<(), BGPError> {
        let serialized = serde_json::to_vec(msg)?;
        let length = serialized.len() as u32;

        // Send length header (4 bytes) + message
        stream.write_u32(length).await?;
        stream.write_all(&serialized).await?;
        stream.flush().await?;

        Ok(())
    }

    async fn receive_message(&self, stream: &mut TcpStream) -> Result<BGPMessage, BGPError> {
        // Read length header
        let length = stream.read_u32().await?;

        if length > 65536 {
            // Reasonable message size limit
            return Err(BGPError::Protocol("Message too large".to_string()));
        }

        // Read message
        let mut buffer = vec![0u8; length as usize];
        stream.read_exact(&mut buffer).await?;

        let msg = serde_json::from_slice(&buffer)?;
        Ok(msg)
    }

    pub async fn advertise_routes(
        &self,
        stream: &mut TcpStream,
        routes: Vec<RouteEntry>,
    ) -> Result<(), BGPError> {
        let bgp_routes: Vec<BGPRoute> = routes
            .into_iter()
            .map(|route| BGPRoute {
                network: route.network,
                next_hop: route.next_hop,
                as_path: route.as_path,
                origin: route.origin,
                local_pref: route.local_pref,
                med: route.med,
            })
            .collect();

        let update_msg = BGPMessage {
            message_type: BGPMessageType::Update,
            asn: self.local_asn,
            router_id: self.router_id,
            routes: bgp_routes,
            timestamp: chrono::Utc::now(),
        };

        self.send_message(stream, &update_msg).await?;
        tracing::info!("Advertised {} routes via BGP", update_msg.routes.len());

        Ok(())
    }
}
