use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use std::sync::Arc;

pub mod session;
pub mod routing;
pub mod messages;
pub mod protocol;

#[derive(Debug, Clone)]
pub struct BGPSession {
    pub peer_asn: u32,
    pub local_asn: u32,
    pub peer_ip: IpAddr,
    pub state: BGPSessionState,
    pub route_table: Arc<RwLock<RouteTable>>,
    pub hold_time: u16,
    pub keepalive_time: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BGPSessionState {
    Idle,
    Connect,
    Active,
    OpenSent,
    OpenConfirm,
    Established,
}

#[derive(Debug, Clone)]
pub struct RouteTable {
    pub routes: HashMap<IpNet, RouteEntry>,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    pub network: IpNet,
    pub next_hop: IpAddr,
    pub as_path: Vec<u32>,
    pub origin: BGPOrigin,
    pub local_pref: u32,
    pub med: u32,
    pub communities: Vec<Community>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BGPOrigin {
    IGP = 0,    // Interior Gateway Protocol
    EGP = 1,    // Exterior Gateway Protocol  
    Incomplete = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub asn: u16,
    pub value: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum BGPError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Route error: {0}")]
    Route(String),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct BGPDaemon {
    local_asn: u32,
    router_id: IpAddr,
    listen_port: u16,
    sessions: Arc<RwLock<HashMap<IpAddr, BGPSession>>>,
    route_table: Arc<RwLock<RouteTable>>,
}

impl BGPDaemon {
    pub fn new(local_asn: u32, router_id: IpAddr, listen_port: u16) -> Self {
        BGPDaemon {
            local_asn,
            router_id,
            listen_port,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            route_table: Arc::new(RwLock::new(RouteTable::new())),
        }
    }

    pub async fn start(&self) -> Result<(), BGPError> {
        let listen_addr = format!("0.0.0.0:{}", self.listen_port);
        let listener = TcpListener::bind(&listen_addr).await?;
        
        tracing::info!("BGP daemon listening on {}", listen_addr);

        let sessions = Arc::clone(&self.sessions);
        let route_table = Arc::clone(&self.route_table);
        let local_asn = self.local_asn;

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        tracing::info!("BGP connection from {}", addr);
                        
                        let sessions = Arc::clone(&sessions);
                        let route_table = Arc::clone(&route_table);
                        
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, addr, local_asn, sessions, route_table).await {
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

    async fn handle_connection(
        _stream: TcpStream,
        addr: SocketAddr,
        local_asn: u32,
        sessions: Arc<RwLock<HashMap<IpAddr, BGPSession>>>,
        route_table: Arc<RwLock<RouteTable>>,
    ) -> Result<(), BGPError> {
        tracing::debug!("Handling BGP connection from {}", addr);
        
        let session = BGPSession::new(local_asn, 65002, addr.ip(), Arc::clone(&route_table));
        
        {
            let mut sessions = sessions.write().await;
            sessions.insert(addr.ip(), session);
        }

        // For now, just log the connection
        tracing::info!("BGP session established with {}", addr.ip());
        
        Ok(())
    }

    pub async fn add_route(&self, network: IpNet, next_hop: IpAddr, origin: BGPOrigin) -> Result<(), BGPError> {
        let route = RouteEntry {
            network,
            next_hop,
            as_path: vec![self.local_asn],
            origin,
            local_pref: 100,
            med: 0,
            communities: vec![],
            timestamp: chrono::Utc::now(),
        };

        let mut table = self.route_table.write().await;
        table.add_route(route)?;
        
        tracing::info!("Added route: {} via {}", network, next_hop);
        Ok(())
    }

    pub async fn get_routes(&self) -> Vec<RouteEntry> {
        let table = self.route_table.read().await;
        table.routes.values().cloned().collect()
    }
}

impl BGPSession {
    pub fn new(local_asn: u32, peer_asn: u32, peer_ip: IpAddr, route_table: Arc<RwLock<RouteTable>>) -> Self {
        BGPSession {
            peer_asn,
            local_asn,
            peer_ip,
            state: BGPSessionState::Idle,
            route_table,
            hold_time: 90,
            keepalive_time: 30,
        }
    }

    pub async fn establish(&mut self) -> Result<(), BGPError> {
        self.state = BGPSessionState::Connect;
        tracing::info!("Establishing BGP session with ASN {} at {}", self.peer_asn, self.peer_ip);
        
        // Simulate session establishment
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        self.state = BGPSessionState::Established;
        tracing::info!("BGP session established with {}", self.peer_ip);
        
        Ok(())
    }
}

impl RouteTable {
    pub fn new() -> Self {
        RouteTable {
            routes: HashMap::new(),
            version: 0,
        }
    }

    pub fn add_route(&mut self, route: RouteEntry) -> Result<(), BGPError> {
        self.routes.insert(route.network, route);
        self.version += 1;
        Ok(())
    }

    pub fn remove_route(&mut self, network: &IpNet) -> Option<RouteEntry> {
        if let Some(route) = self.routes.remove(network) {
            self.version += 1;
            Some(route)
        } else {
            None
        }
    }

    pub fn get_route(&self, network: &IpNet) -> Option<&RouteEntry> {
        self.routes.get(network)
    }

    pub fn get_all_routes(&self) -> Vec<&RouteEntry> {
        self.routes.values().collect()
    }
}