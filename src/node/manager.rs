use crate::node::{Vx0Node, NodeError, ConnectionStatus};
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub struct NodeManager {
    node: Arc<Vx0Node>,
}

impl NodeManager {
    pub fn new(node: Arc<Vx0Node>) -> Self {
        NodeManager { node }
    }

    pub async fn run(&self) -> Result<(), NodeError> {
        let node = Arc::clone(&self.node);
        
        // Start peer management task
        let peer_manager = Arc::clone(&node);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = peer_manager.manage_peers().await {
                    tracing::error!("Peer management error: {}", e);
                }
            }
        });

        // Start health monitoring task
        let health_monitor = Arc::clone(&node);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                health_monitor.check_health().await;
            }
        });

        Ok(())
    }
}

impl Vx0Node {
    async fn manage_peers(&self) -> Result<(), NodeError> {
        let peers = self.peers.read().await;
        let peer_count = peers.len();
        
        tracing::debug!("Managing {} peer connections", peer_count);
        
        for (peer_id, peer) in peers.iter() {
            match peer.status {
                ConnectionStatus::Failed => {
                    tracing::warn!("Peer {} connection failed, attempting reconnect", peer_id);
                }
                ConnectionStatus::Disconnected => {
                    tracing::debug!("Peer {} is disconnected", peer_id);
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    async fn check_health(&self) {
        let peer_count = self.get_peer_count().await;
        let service_count = {
            let services = self.services.read().await;
            services.len()
        };
        
        tracing::debug!(
            "Node health check: {} peers, {} services", 
            peer_count, 
            service_count
        );
    }
}