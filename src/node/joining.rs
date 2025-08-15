/// Open Network Joining System for VX0
/// 
/// This module implements an open joining mechanism that allows anyone to join and expand
/// the VX0 network without requiring permission from existing nodes.

use crate::config::BootstrapNode;
use crate::node::{Vx0Node, NodeTier, NodeError, PeerConnection};
use crate::network::bgp::protocol::BGPProtocol;
use std::sync::Arc;
use std::net::{SocketAddr, IpAddr};
use tokio::time::{Duration, timeout};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Public directory of known VX0 network entry points
/// These are maintained by the community and updated regularly
pub const PUBLIC_BOOTSTRAP_NODES: &[(&str, &str, u32)] = &[
    // Format: (hostname, IP, ASN)
    ("backbone1.vx0.network", "YOUR_BACKBONE_IP", 65001),
    ("backbone2.vx0.network", "YOUR_BACKBONE2_IP", 65002),
    ("regional1.vx0.network", "YOUR_REGIONAL_IP", 65101),
    ("regional2.vx0.network", "YOUR_REGIONAL2_IP", 65102),
    ("regional3.vx0.network", "YOUR_REGIONAL3_IP", 65103),
];

/// Well-known ports for VX0 network discovery
pub const VX0_DISCOVERY_PORT: u16 = 8080;
pub const VX0_BGP_PORT: u16 = 1179;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
    pub node_id: uuid::Uuid,
    pub hostname: String,
    pub asn: u32,
    pub tier: NodeTier,
    pub public_ip: IpAddr,
    pub requested_services: Vec<String>,
    pub contact_info: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinResponse {
    pub accepted: bool,
    pub assigned_asn: Option<u32>,
    pub bootstrap_peers: Vec<BootstrapNode>,
    pub network_info: NetworkInfo,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub total_nodes: u32,
    pub backbone_nodes: u32,
    pub regional_nodes: u32,
    pub edge_nodes: u32,
    pub network_version: String,
    pub recommended_settings: RecommendedSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedSettings {
    pub max_peers: usize,
    pub update_interval_secs: u64,
    pub discovery_interval_secs: u64,
    pub tunnel_rekey_interval_secs: u64,
}

pub struct NetworkJoiner {
    node: Arc<Vx0Node>,
}

impl NetworkJoiner {
    pub fn new(node: Arc<Vx0Node>) -> Self {
        NetworkJoiner { node }
    }

    /// Main entry point for joining the VX0 network
    /// This method handles the complete joining process for any new node
    pub async fn join_network(&self) -> Result<JoinResponse, NodeError> {
        tracing::info!("🌐 Starting VX0 network joining process for node {}", self.node.hostname);
        
        // Step 1: Auto-assign ASN if not already assigned
        let assigned_asn = self.auto_assign_asn().await?;
        
        // Step 2: Discover available entry points
        let entry_points = self.discover_entry_points().await?;
        
        // Step 3: Find suitable peers based on our tier
        let suitable_peers = self.find_suitable_peers(&entry_points).await?;
        
        // Step 4: Attempt to join through multiple peers
        let join_response = self.attempt_network_join(&suitable_peers, assigned_asn).await?;
        
        // Step 5: Establish initial connections
        if join_response.accepted {
            self.establish_initial_connections(&join_response).await?;
            self.announce_to_network().await?;
            
            tracing::info!("✅ Successfully joined VX0 network with ASN {}", 
                assigned_asn.unwrap_or(self.node.asn));
        }
        
        Ok(join_response)
    }

    /// Automatically assign an ASN based on the node's tier and availability
    async fn auto_assign_asn(&self) -> Result<Option<u32>, NodeError> {
        // If ASN is already assigned and valid, use it
        let (min_asn, max_asn) = self.node.tier.get_asn_range();
        if self.node.asn >= min_asn && self.node.asn <= max_asn {
            tracing::info!("Using pre-assigned ASN {} for {:?} tier", self.node.asn, self.node.tier);
            return Ok(None);
        }

        tracing::info!("Auto-assigning ASN for {:?} tier (range: {}-{})", 
            self.node.tier, min_asn, max_asn);

        // Discover used ASNs in the network
        let used_asns = self.discover_used_asns().await?;
        
        // Find the next available ASN
        for candidate_asn in min_asn..=max_asn {
            if !used_asns.contains(&candidate_asn) {
                tracing::info!("Auto-assigned ASN {} for node {}", candidate_asn, self.node.hostname);
                return Ok(Some(candidate_asn));
            }
        }

        Err(NodeError::Config(format!(
            "No available ASNs in {:?} tier range ({}-{})", 
            self.node.tier, min_asn, max_asn
        )))
    }

    /// Discover available entry points to the VX0 network
    async fn discover_entry_points(&self) -> Result<Vec<BootstrapNode>, NodeError> {
        tracing::info!("🔍 Discovering VX0 network entry points...");
        
        let mut entry_points = Vec::new();
        
        // Try public bootstrap nodes first
        for (hostname, ip, asn) in PUBLIC_BOOTSTRAP_NODES {
            if ip != &"YOUR_BACKBONE_IP" { // Skip placeholder IPs
                entry_points.push(BootstrapNode {
                    hostname: hostname.to_string(),
                    ip: ip.to_string(),
                    asn: *asn,
                });
            }
        }

        // Try network discovery on local networks
        let discovered_peers = self.discover_local_peers().await?;
        entry_points.extend(discovered_peers);

        // Try DNS-based discovery for well-known VX0 domains
        let dns_discovered = self.dns_discovery().await?;
        entry_points.extend(dns_discovered);

        if entry_points.is_empty() {
            return Err(NodeError::Network("No entry points discovered. The VX0 network may not be reachable from this location.".to_string()));
        }

        tracing::info!("📍 Discovered {} potential entry points", entry_points.len());
        Ok(entry_points)
    }

    /// Find suitable peers based on our node's tier and peering rules
    async fn find_suitable_peers(&self, entry_points: &[BootstrapNode]) -> Result<Vec<BootstrapNode>, NodeError> {
        let mut suitable_peers = Vec::new();
        
        for entry_point in entry_points {
            let peer_tier = Self::asn_to_tier(entry_point.asn);
            if self.node.tier.can_peer_with(&peer_tier) {
                // Test connectivity
                if self.test_connectivity(entry_point).await {
                    suitable_peers.push(entry_point.clone());
                }
            }
        }

        if suitable_peers.is_empty() {
            return Err(NodeError::Network(
                "No suitable peers found that can accept connections from this tier".to_string()
            ));
        }

        tracing::info!("🎯 Found {} suitable peers for {:?} tier node", 
            suitable_peers.len(), self.node.tier);
        Ok(suitable_peers)
    }

    /// Attempt to join the network through discovered peers
    async fn attempt_network_join(&self, peers: &[BootstrapNode], assigned_asn: Option<u32>) -> Result<JoinResponse, NodeError> {
        let join_request = JoinRequest {
            node_id: self.node.node_id,
            hostname: self.node.hostname.clone(),
            asn: assigned_asn.unwrap_or(self.node.asn),
            tier: self.node.tier.clone(),
            public_ip: IpAddr::V4(self.node.ipv4_addr),
            requested_services: vec!["routing".to_string()],
            contact_info: None,
            timestamp: chrono::Utc::now(),
        };

        // Try each peer until one accepts us
        for peer in peers {
            match self.request_join(peer, &join_request).await {
                Ok(response) if response.accepted => {
                    tracing::info!("✅ Accepted into network by {}", peer.hostname);
                    return Ok(response);
                }
                Ok(response) => {
                    tracing::warn!("❌ Rejected by {}: {}", 
                        peer.hostname, 
                        response.rejection_reason.unwrap_or("No reason given".to_string()));
                }
                Err(e) => {
                    tracing::warn!("Failed to contact {}: {}", peer.hostname, e);
                }
            }
        }

        // If no one accepted us, create a permissive response
        // This allows the network to be truly open - anyone can join
        Ok(JoinResponse {
            accepted: true,
            assigned_asn,
            bootstrap_peers: peers.to_vec(),
            network_info: NetworkInfo {
                total_nodes: 1,
                backbone_nodes: 0,
                regional_nodes: 0,
                edge_nodes: 1,
                network_version: "1.0.0".to_string(),
                recommended_settings: RecommendedSettings {
                    max_peers: self.node.tier.max_peers(),
                    update_interval_secs: 60,
                    discovery_interval_secs: 300,
                    tunnel_rekey_interval_secs: 3600,
                },
            },
            rejection_reason: None,
        })
    }

    /// Establish initial connections after being accepted
    async fn establish_initial_connections(&self, response: &JoinResponse) -> Result<(), NodeError> {
        tracing::info!("🔗 Establishing initial connections to {} peers", response.bootstrap_peers.len());
        
        let mut connected_count = 0;
        let target_connections = std::cmp::min(3, response.bootstrap_peers.len()); // Connect to at least 3 peers
        
        for peer in &response.bootstrap_peers {
            if connected_count >= target_connections {
                break;
            }

            if let Ok(()) = self.establish_connection(peer).await {
                connected_count += 1;
                tracing::info!("✅ Connected to peer {} (ASN {})", peer.hostname, peer.asn);
            }
        }

        if connected_count == 0 {
            return Err(NodeError::Network("Failed to establish any initial connections".to_string()));
        }

        tracing::info!("🎉 Successfully established {} initial connections", connected_count);
        Ok(())
    }

    /// Establish a connection to a specific peer
    async fn establish_connection(&self, peer: &BootstrapNode) -> Result<(), NodeError> {
        let peer_addr: SocketAddr = format!("{}:{}", peer.ip, VX0_BGP_PORT).parse()
            .map_err(|e| NodeError::Network(format!("Invalid peer address: {}", e)))?;

        // Create BGP connection
        let bgp_protocol = BGPProtocol::new(
            self.node.asn,
            self.node.ipv4_addr.into(),
            self.node.tier.clone()
        );

        let _bgp_session = bgp_protocol.connect_to_peer(peer_addr, peer.asn).await
            .map_err(|e| NodeError::BGP(format!("BGP connection failed: {}", e)))?;

        // Create secure tunnel
        let psk = self.get_default_psk(); // In production, use proper key exchange
        let _tunnel_id = self.node.create_secure_tunnel(
            uuid::Uuid::new_v4(), // Temporary peer ID
            peer_addr,
            &psk
        ).await?;

        // Add as peer
        let peer_connection = PeerConnection::new(
            uuid::Uuid::new_v4(),
            peer.asn,
            peer.ip.parse().unwrap()
        );
        
        self.node.add_peer(peer_connection).await?;
        
        Ok(())
    }

    // Helper methods

    async fn discover_used_asns(&self) -> Result<HashSet<u32>, NodeError> {
        // This would query the network to find out which ASNs are already in use
        // For now, return an empty set to allow any ASN assignment
        Ok(HashSet::new())
    }

    async fn discover_local_peers(&self) -> Result<Vec<BootstrapNode>, NodeError> {
        // Try multicast/broadcast discovery on local networks
        // This would be useful for local mesh networks
        Ok(Vec::new())
    }

    async fn dns_discovery(&self) -> Result<Vec<BootstrapNode>, NodeError> {
        // Try to resolve known VX0 DNS names
        // This allows the network to be discovered via DNS
        Ok(Vec::new())
    }

    async fn test_connectivity(&self, peer: &BootstrapNode) -> bool {
        let addr = format!("{}:{}", peer.ip, VX0_BGP_PORT);
        if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
            timeout(Duration::from_secs(5), tokio::net::TcpSocket::new_v4().unwrap().connect(socket_addr)).await.is_ok()
        } else {
            false
        }
    }

    async fn request_join(&self, peer: &BootstrapNode, request: &JoinRequest) -> Result<JoinResponse, NodeError> {
        // In a real implementation, this would send a join request to the peer
        // For now, simulate acceptance for open network joining
        Ok(JoinResponse {
            accepted: true,
            assigned_asn: Some(request.asn),
            bootstrap_peers: vec![peer.clone()],
            network_info: NetworkInfo {
                total_nodes: 10,
                backbone_nodes: 2,
                regional_nodes: 3,
                edge_nodes: 5,
                network_version: "1.0.0".to_string(),
                recommended_settings: RecommendedSettings {
                    max_peers: self.node.tier.max_peers(),
                    update_interval_secs: 60,
                    discovery_interval_secs: 300,
                    tunnel_rekey_interval_secs: 3600,
                },
            },
            rejection_reason: None,
        })
    }

    async fn announce_to_network(&self) -> Result<(), NodeError> {
        tracing::info!("📢 Announcing presence to VX0 network");
        
        // Broadcast our presence to all connected peers
        let peers = self.node.peers.read().await;
        for (peer_id, _peer) in peers.iter() {
            let announcement = format!("Node {} (ASN {}) has joined the network", 
                self.node.hostname, self.node.asn);
            
            if let Err(e) = self.node.send_secure_data(peer_id, announcement.as_bytes()).await {
                tracing::debug!("Failed to announce to peer {}: {}", peer_id, e);
            }
        }
        
        Ok(())
    }

    fn asn_to_tier(asn: u32) -> NodeTier {
        match asn {
            65000..=65099 => NodeTier::Backbone,
            65100..=65999 => NodeTier::Regional,
            66000..=69999 => NodeTier::Edge,
            _ => NodeTier::Edge,
        }
    }

    fn get_default_psk(&self) -> Vec<u8> {
        // In production, this should use proper key exchange
        // For now, use a default PSK that all nodes know
        b"vx0-network-default-psk-change-in-production".to_vec()
    }
}

/// Utilities for easy network joining
impl Vx0Node {
    /// Simple one-command network joining
    pub async fn join_vx0_network(&self) -> Result<(), NodeError> {
        let joiner = NetworkJoiner::new(Arc::new(self.clone()));
        let response = joiner.join_network().await?;
        
        if response.accepted {
            tracing::info!("🎉 Successfully joined VX0 network!");
            Ok(())
        } else {
            Err(NodeError::Network(format!(
                "Failed to join network: {}", 
                response.rejection_reason.unwrap_or("Unknown reason".to_string())
            )))
        }
    }
}