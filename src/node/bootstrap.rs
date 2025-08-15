use crate::config::{BootstrapConfig, BootstrapNode};
use crate::node::{Vx0Node, PeerConnection, NodeError};
use crate::network::bgp::protocol::BGPProtocol;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

pub struct BootstrapManager {
    node: Arc<Vx0Node>,
    bootstrap_config: Option<BootstrapConfig>,
}

impl BootstrapManager {
    pub fn new(node: Arc<Vx0Node>, bootstrap_config: Option<BootstrapConfig>) -> Self {
        BootstrapManager {
            node,
            bootstrap_config,
        }
    }

    pub async fn discover_and_connect(&self) -> Result<(), NodeError> {
        if let Some(bootstrap) = &self.bootstrap_config {
            tracing::info!("Starting bootstrap discovery with {} seed nodes", bootstrap.nodes.len());
            
            for bootstrap_node in &bootstrap.nodes {
                if let Err(e) = self.connect_to_bootstrap_node(bootstrap_node).await {
                    tracing::warn!("Failed to connect to bootstrap node {}: {}", bootstrap_node.hostname, e);
                    continue;
                }
                
                // Small delay between connections
                sleep(Duration::from_millis(500)).await;
            }
        } else {
            tracing::info!("No bootstrap configuration found, running in standalone mode");
        }

        Ok(())
    }

    async fn connect_to_bootstrap_node(&self, bootstrap_node: &BootstrapNode) -> Result<(), NodeError> {
        tracing::info!("Attempting to connect to bootstrap node: {} (ASN {})", 
            bootstrap_node.hostname, bootstrap_node.asn);

        // Check if this node can peer with the bootstrap node based on tier rules
        let bootstrap_tier = Self::asn_to_tier(bootstrap_node.asn);
        if !self.node.tier.can_peer_with(&bootstrap_tier) {
            return Err(NodeError::Network(format!(
                "Cannot peer with {} - tier mismatch ({:?} cannot peer with {:?})",
                bootstrap_node.hostname, self.node.tier, bootstrap_tier
            )));
        }

        // Parse bootstrap node address
        let peer_addr: SocketAddr = format!("{}:{}", bootstrap_node.ip, 1179).parse()
            .map_err(|e| NodeError::Network(format!("Invalid bootstrap address: {}", e)))?;

        // Attempt BGP connection
        let bgp_protocol = BGPProtocol::new(
            self.node.asn,
            self.node.ipv4_addr.into(),
            self.node.tier.clone()
        );

        match bgp_protocol.connect_to_peer(peer_addr, bootstrap_node.asn).await {
            Ok(bgp_session) => {
                tracing::info!("Successfully established BGP session with {}", bootstrap_node.hostname);
                
                // Create peer connection
                let peer = PeerConnection::new(
                    uuid::Uuid::new_v4(), // We'll get the real node ID later
                    bootstrap_node.asn,
                    bootstrap_node.ip.parse().unwrap()
                );

                // Add peer to our node
                self.node.add_peer(peer).await?;
                
                tracing::info!("Added {} as peer", bootstrap_node.hostname);
            }
            Err(e) => {
                tracing::error!("Failed to establish BGP session with {}: {}", bootstrap_node.hostname, e);
                return Err(NodeError::BGP(format!("BGP connection failed: {}", e)));
            }
        }

        Ok(())
    }

    fn asn_to_tier(asn: u32) -> crate::node::NodeTier {
        match asn {
            65000..=65099 => crate::node::NodeTier::Backbone,
            65100..=65999 => crate::node::NodeTier::Regional,
            66000..=69999 => crate::node::NodeTier::Edge,
            _ => crate::node::NodeTier::Edge,
        }
    }

    pub async fn start_periodic_discovery(&self) {
        let bootstrap_config = self.bootstrap_config.clone();
        let node = Arc::clone(&self.node);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
            
            loop {
                interval.tick().await;
                
                if let Some(bootstrap) = &bootstrap_config {
                    // Check if we need more peers
                    let current_peers = node.get_peer_count().await;
                    let max_peers = node.tier.max_peers();
                    
                    if current_peers < max_peers / 2 {  // If we have less than half our max peers
                        tracing::info!("Low peer count ({}/{}), attempting to discover more peers", 
                            current_peers, max_peers);
                        
                        // Try to connect to more bootstrap nodes
                        for bootstrap_node in &bootstrap.nodes {
                            if current_peers >= max_peers {
                                break;
                            }
                            
                            // Check if we're already connected to this node
                            if Self::is_already_connected(&node, bootstrap_node).await {
                                continue;
                            }
                            
                            let bootstrap_manager = BootstrapManager::new(Arc::clone(&node), None);
                            if let Err(e) = bootstrap_manager.connect_to_bootstrap_node(bootstrap_node).await {
                                tracing::debug!("Periodic discovery connection failed: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }

    async fn is_already_connected(node: &Arc<Vx0Node>, bootstrap_node: &BootstrapNode) -> bool {
        let peers = node.peers.read().await;
        for peer in peers.values() {
            if peer.peer_asn == bootstrap_node.asn {
                return true;
            }
        }
        false
    }

    pub async fn announce_to_network(&self) -> Result<(), NodeError> {
        tracing::info!("Announcing node to VX0 network");
        
        // Create announcement with our node information
        let announcement = NodeAnnouncement {
            node_id: self.node.node_id,
            hostname: self.node.hostname.clone(),
            asn: self.node.asn,
            tier: self.node.tier.clone(),
            ipv4_addr: self.node.ipv4_addr,
            services: self.get_service_summary().await,
            timestamp: chrono::Utc::now(),
        };

        // Send announcement to all connected peers
        let peers = self.node.peers.read().await;
        for (peer_id, peer) in peers.iter() {
            if let Err(e) = self.send_announcement_to_peer(&announcement, peer).await {
                tracing::warn!("Failed to send announcement to peer {}: {}", peer_id, e);
            }
        }

        Ok(())
    }

    async fn get_service_summary(&self) -> Vec<ServiceSummary> {
        let services = self.node.services.read().await;
        services.iter().map(|service| ServiceSummary {
            name: service.name.clone(),
            domain: service.domain.clone(),
            service_type: service.service_type.clone(),
            port: service.port,
        }).collect()
    }

    async fn send_announcement_to_peer(&self, announcement: &NodeAnnouncement, peer: &PeerConnection) -> Result<(), NodeError> {
        // In a real implementation, this would send the announcement over the BGP connection
        tracing::debug!("Sending node announcement to peer ASN {}", peer.peer_asn);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NodeAnnouncement {
    pub node_id: uuid::Uuid,
    pub hostname: String,
    pub asn: u32,
    pub tier: crate::node::NodeTier,
    pub ipv4_addr: std::net::Ipv4Addr,
    pub services: Vec<ServiceSummary>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ServiceSummary {
    pub name: String,
    pub domain: String,
    pub service_type: crate::node::ServiceType,
    pub port: u16,
}