use crate::node::{NodeId, PeerConnection, Vx0Node};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use tokio::net::UdpSocket;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryMessage {
    pub message_type: DiscoveryMessageType,
    pub node_id: NodeId,
    pub asn: u32,
    pub hostname: String,
    pub addresses: Vec<IpAddr>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DiscoveryMessageType {
    Announce,
    Query,
    Response,
}

pub struct PeerDiscovery {
    socket: UdpSocket,
    known_peers: HashMap<NodeId, PeerConnection>,
}

impl PeerDiscovery {
    pub async fn new(bind_addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(bind_addr).await?;
        socket.set_broadcast(true)?;

        Ok(PeerDiscovery {
            socket,
            known_peers: HashMap::new(),
        })
    }

    pub async fn announce(&self, node: &Vx0Node) -> Result<(), Box<dyn std::error::Error>> {
        let announcement = DiscoveryMessage {
            message_type: DiscoveryMessageType::Announce,
            node_id: node.node_id,
            asn: node.asn,
            hostname: node.hostname.clone(),
            addresses: vec![IpAddr::V4(node.ipv4_addr), IpAddr::V6(node.ipv6_addr)],
            timestamp: chrono::Utc::now(),
        };

        let message = serde_json::to_vec(&announcement)?;

        // Broadcast to local network
        self.socket
            .send_to(&message, "255.255.255.255:8080")
            .await?;

        tracing::debug!("Announced node {} to network", node.node_id);
        Ok(())
    }

    pub async fn listen_for_peers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = [0; 1024];

        loop {
            match self.socket.recv_from(&mut buf).await {
                Ok((size, addr)) => {
                    if let Ok(message) = serde_json::from_slice::<DiscoveryMessage>(&buf[..size]) {
                        self.handle_discovery_message(message, addr.ip()).await;
                    }
                }
                Err(e) => {
                    tracing::error!("Error receiving discovery message: {}", e);
                }
            }
        }
    }

    async fn handle_discovery_message(&mut self, message: DiscoveryMessage, sender_addr: IpAddr) {
        match message.message_type {
            DiscoveryMessageType::Announce => {
                tracing::info!(
                    "Discovered peer {} (ASN: {}) at {}",
                    message.hostname,
                    message.asn,
                    sender_addr
                );

                if let std::collections::hash_map::Entry::Vacant(e) =
                    self.known_peers.entry(message.node_id)
                {
                    let peer = PeerConnection::new(message.node_id, message.asn, sender_addr);
                    e.insert(peer);
                }
            }
            DiscoveryMessageType::Query => {
                tracing::debug!("Received peer query from {}", sender_addr);
            }
            DiscoveryMessageType::Response => {
                tracing::debug!("Received peer response from {}", sender_addr);
            }
        }
    }

    pub fn get_discovered_peers(&self) -> Vec<&PeerConnection> {
        self.known_peers.values().collect()
    }
}
