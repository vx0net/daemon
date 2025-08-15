use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::config::Vx0Config;
use crate::network::ike::tunnels::{TunnelManager, TunnelId};

pub mod manager;
pub mod peer;
pub mod discovery;
pub mod bootstrap;
pub mod joining;

pub type NodeId = Uuid;

#[derive(Debug, Clone)]
pub struct Vx0Node {
    pub node_id: NodeId,
    pub asn: u32,
    pub tier: NodeTier,
    pub location: GeographicLocation,
    pub ipv4_addr: Ipv4Addr,
    pub ipv6_addr: Ipv6Addr,
    pub hostname: String,
    pub peers: Arc<RwLock<HashMap<NodeId, PeerConnection>>>,
    pub services: Arc<RwLock<Vec<HostedService>>>,
    pub config: Vx0Config,
    pub tunnel_manager: Arc<TunnelManager>,
    pub active_tunnels: Arc<RwLock<HashMap<NodeId, TunnelId>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeTier {
    Backbone,    // Tier 1: Core routing infrastructure (ASN 65000-65099)
    Regional,    // Tier 2: Regional distribution hubs (ASN 65100-65999) 
    Edge,        // Tier 3: User-operated nodes (ASN 66000+)
}

impl NodeTier {
    pub fn get_asn_range(&self) -> (u32, u32) {
        match self {
            NodeTier::Backbone => (65000, 65099),  // 100 backbone ASNs
            NodeTier::Regional => (65100, 65999),  // 900 regional ASNs
            NodeTier::Edge => (66000, 69999),      // 4000 edge ASNs
        }
    }

    pub fn max_peers(&self) -> usize {
        match self {
            NodeTier::Backbone => 50,  // High connectivity
            NodeTier::Regional => 20,  // Moderate connectivity  
            NodeTier::Edge => 5,       // Limited connectivity
        }
    }

    pub fn can_peer_with(&self, other: &NodeTier) -> bool {
        match (self, other) {
            // Backbone can peer with backbone and regional
            (NodeTier::Backbone, NodeTier::Backbone) => true,
            (NodeTier::Backbone, NodeTier::Regional) => true,
            // Regional can peer with backbone, regional, and edge
            (NodeTier::Regional, NodeTier::Backbone) => true,
            (NodeTier::Regional, NodeTier::Regional) => true,
            (NodeTier::Regional, NodeTier::Edge) => true,
            // Edge can only peer with regional (no edge-to-edge)
            (NodeTier::Edge, NodeTier::Regional) => true,
            // All other combinations not allowed
            _ => false,
        }
    }

    pub fn route_advertisement_policy(&self) -> RoutePolicy {
        match self {
            NodeTier::Backbone => RoutePolicy::FullTable,     // Accept/advertise all routes
            NodeTier::Regional => RoutePolicy::RegionalFilter, // Filter by region/policy
            NodeTier::Edge => RoutePolicy::DefaultOnly,       // Only default route + local
        }
    }
}

#[derive(Debug, Clone)]
pub enum RoutePolicy {
    FullTable,       // Accept and advertise all routes
    RegionalFilter,  // Filter routes based on regional policies
    DefaultOnly,     // Only accept default route and advertise local services
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicLocation {
    pub country: String,
    pub region: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnection {
    pub peer_id: NodeId,
    pub peer_asn: u32,
    pub peer_addr: IpAddr,
    pub status: ConnectionStatus,
    pub metrics: ConnectionMetrics,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Authenticated,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    pub latency_ms: u64,
    pub packet_loss: f32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub routes_advertised: u32,
    pub routes_received: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostedService {
    pub service_id: Uuid,
    pub name: String,
    pub service_type: ServiceType,
    pub domain: String,
    pub port: u16,
    pub status: ServiceStatus,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    WebServer,
    EmailServer,
    FileServer,
    ChatServer,
    Database,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("BGP error: {0}")]
    BGP(String),
    #[error("IKE error: {0}")]
    IKE(String),
    #[error("Service error: {0}")]
    Service(String),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl Vx0Node {
    pub fn new(config: Vx0Config) -> Result<Self, NodeError> {
        let ipv4_addr = config.get_ipv4_addr()
            .map_err(|e| NodeError::Config(format!("Invalid IPv4 address: {}", e)))?;
        let ipv6_addr = config.get_ipv6_addr()
            .map_err(|e| NodeError::Config(format!("Invalid IPv6 address: {}", e)))?;
        
        let tier = match config.node.tier.as_str() {
            "Backbone" => NodeTier::Backbone,
            "Regional" => NodeTier::Regional,
            "Edge" => NodeTier::Edge,
            // Legacy support
            "Tier1" => NodeTier::Backbone,
            "Tier2" => NodeTier::Regional,
            _ => NodeTier::Edge,
        };

        // Validate ASN is within tier range
        let (min_asn, max_asn) = tier.get_asn_range();
        if config.node.asn < min_asn || config.node.asn > max_asn {
            return Err(NodeError::Config(format!(
                "ASN {} not valid for {:?} tier (valid range: {}-{})", 
                config.node.asn, tier, min_asn, max_asn
            )));
        }

        let location = GeographicLocation {
            country: "US".to_string(),
            region: "Unknown".to_string(), 
            city: config.node.location.clone(),
            latitude: 0.0,
            longitude: 0.0,
        };

        Ok(Vx0Node {
            node_id: Uuid::new_v4(),
            asn: config.node.asn,
            tier,
            location,
            ipv4_addr,
            ipv6_addr,
            hostname: config.node.hostname.clone(),
            peers: Arc::new(RwLock::new(HashMap::new())),
            services: Arc::new(RwLock::new(Vec::new())),
            config,
            tunnel_manager: Arc::new(TunnelManager::new()),
            active_tunnels: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn start(&self) -> Result<(), NodeError> {
        tracing::info!("Starting VX0 node {} (ASN: {})", self.hostname, self.asn);
        
        // Initialize services
        self.start_monitoring().await?;
        self.start_service_discovery().await?;
        
        tracing::info!("VX0 node started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), NodeError> {
        tracing::info!("Stopping VX0 node {}", self.hostname);
        
        // Close all peer connections
        let mut peers = self.peers.write().await;
        for (peer_id, peer) in peers.iter_mut() {
            peer.status = ConnectionStatus::Disconnected;
            tracing::debug!("Disconnected from peer {}", peer_id);
        }
        
        tracing::info!("VX0 node stopped");
        Ok(())
    }

    pub async fn add_peer(&self, peer: PeerConnection) -> Result<(), NodeError> {
        // Check if we've reached max peer limit for our tier
        let max_peers = self.tier.max_peers();
        let current_peers = self.get_peer_count().await;
        
        if current_peers >= max_peers {
            return Err(NodeError::Network(format!(
                "Maximum peer limit reached for {:?} tier ({}/{})", 
                self.tier, current_peers, max_peers
            )));
        }

        // Determine peer tier from ASN
        let peer_tier = Self::asn_to_tier(peer.peer_asn);
        
        // Check if this tier can peer with the other tier
        if !self.tier.can_peer_with(&peer_tier) {
            return Err(NodeError::Network(format!(
                "{:?} nodes cannot peer with {:?} nodes", 
                self.tier, peer_tier
            )));
        }

        let peer_id = peer.peer_id;
        let peer_asn = peer.peer_asn;
        
        let mut peers = self.peers.write().await;
        peers.insert(peer_id, peer);
        
        tracing::info!(
            "Added {:?} peer (ASN {}) to {:?} node", 
            peer_tier, peer_asn, self.tier
        );
        
        Ok(())
    }

    fn asn_to_tier(asn: u32) -> NodeTier {
        match asn {
            65000..=65099 => NodeTier::Backbone,
            65100..=65999 => NodeTier::Regional,
            66000..=69999 => NodeTier::Edge,
            _ => NodeTier::Edge, // Default fallback
        }
    }

    pub async fn remove_peer(&self, peer_id: &NodeId) -> Result<(), NodeError> {
        let mut peers = self.peers.write().await;
        peers.remove(peer_id);
        Ok(())
    }

    pub async fn get_peer_count(&self) -> usize {
        let peers = self.peers.read().await;
        peers.len()
    }

    pub async fn register_service(&self, service: HostedService) -> Result<(), NodeError> {
        if !service.domain.ends_with(".vx0") {
            return Err(NodeError::Service("Service domain must end with .vx0".to_string()));
        }
        
        let mut services = self.services.write().await;
        services.push(service);
        Ok(())
    }

    async fn start_monitoring(&self) -> Result<(), NodeError> {
        tracing::debug!("Starting monitoring for node {}", self.node_id);
        Ok(())
    }

    async fn start_service_discovery(&self) -> Result<(), NodeError> {
        tracing::debug!("Starting service discovery for node {}", self.node_id);
        Ok(())
    }

    // Tunnel management methods
    pub async fn create_secure_tunnel(&self, peer_id: NodeId, peer_addr: SocketAddr, psk: &[u8]) -> Result<TunnelId, NodeError> {
        tracing::info!("Creating secure tunnel to peer {} at {}", peer_id, peer_addr);
        
        let tunnel_id = self.tunnel_manager
            .create_tunnel(
                IpAddr::V4(self.ipv4_addr),
                IpAddr::from(peer_addr.ip()),
                peer_addr,
                psk,
            )
            .await
            .map_err(|e| NodeError::IKE(format!("Failed to create tunnel: {}", e)))?;

        // Store the tunnel mapping
        let mut tunnels = self.active_tunnels.write().await;
        tunnels.insert(peer_id, tunnel_id);

        tracing::info!("Secure tunnel {} established with peer {}", tunnel_id, peer_id);
        Ok(tunnel_id)
    }

    pub async fn send_secure_data(&self, peer_id: &NodeId, data: &[u8]) -> Result<(), NodeError> {
        let tunnels = self.active_tunnels.read().await;
        if let Some(tunnel_id) = tunnels.get(peer_id) {
            self.tunnel_manager
                .send_packet(tunnel_id, data)
                .await
                .map_err(|e| NodeError::IKE(format!("Failed to send secure data: {}", e)))?;
            Ok(())
        } else {
            Err(NodeError::IKE(format!("No tunnel found for peer {}", peer_id)))
        }
    }

    pub async fn close_tunnel(&self, peer_id: &NodeId) -> Result<(), NodeError> {
        let mut tunnels = self.active_tunnels.write().await;
        if let Some(tunnel_id) = tunnels.remove(peer_id) {
            self.tunnel_manager
                .close_tunnel(&tunnel_id)
                .await
                .map_err(|e| NodeError::IKE(format!("Failed to close tunnel: {}", e)))?;
            tracing::info!("Closed tunnel to peer {}", peer_id);
        }
        Ok(())
    }

    pub async fn get_tunnel_stats(&self, peer_id: &NodeId) -> Option<crate::network::ike::tunnels::TrafficStats> {
        let tunnels = self.active_tunnels.read().await;
        if let Some(tunnel_id) = tunnels.get(peer_id) {
            self.tunnel_manager.get_tunnel_stats(tunnel_id).await
        } else {
            None
        }
    }

    pub async fn list_active_tunnels(&self) -> Vec<(NodeId, TunnelId)> {
        let tunnels = self.active_tunnels.read().await;
        tunnels.iter().map(|(k, v)| (*k, *v)).collect()
    }

    pub async fn tunnel_health_check(&self) -> Result<HashMap<NodeId, bool>, NodeError> {
        let tunnels = self.active_tunnels.read().await;
        let mut health_status = HashMap::new();
        
        for (peer_id, tunnel_id) in tunnels.iter() {
            if let Some(tunnel) = self.tunnel_manager.get_tunnel(tunnel_id).await {
                health_status.insert(*peer_id, matches!(tunnel.status, crate::network::ike::tunnels::TunnelStatus::Established));
            } else {
                health_status.insert(*peer_id, false);
            }
        }
        
        Ok(health_status)
    }
}

impl Default for ConnectionMetrics {
    fn default() -> Self {
        ConnectionMetrics {
            latency_ms: 0,
            packet_loss: 0.0,
            bytes_sent: 0,
            bytes_received: 0,
            routes_advertised: 0,
            routes_received: 0,
        }
    }
}