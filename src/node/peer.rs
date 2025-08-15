use crate::node::{ConnectionMetrics, ConnectionStatus, NodeId, PeerConnection};
use std::net::IpAddr;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

impl PeerConnection {
    pub fn new(peer_id: NodeId, peer_asn: u32, peer_addr: IpAddr) -> Self {
        PeerConnection {
            peer_id,
            peer_asn,
            peer_addr,
            status: ConnectionStatus::Disconnected,
            metrics: ConnectionMetrics::default(),
            last_seen: chrono::Utc::now(),
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.status = ConnectionStatus::Connecting;

        let addr = format!("{}:179", self.peer_addr);

        match timeout(Duration::from_secs(10), TcpStream::connect(&addr)).await {
            Ok(Ok(stream)) => {
                tracing::info!(
                    "Successfully connected to peer {} at {}",
                    self.peer_id,
                    addr
                );
                self.status = ConnectionStatus::Connected;
                self.last_seen = chrono::Utc::now();
                drop(stream); // For now, just test the connection
                Ok(())
            }
            Ok(Err(e)) => {
                tracing::error!("Failed to connect to peer {}: {}", self.peer_id, e);
                self.status = ConnectionStatus::Failed;
                Err(Box::new(e))
            }
            Err(_) => {
                tracing::error!("Connection to peer {} timed out", self.peer_id);
                self.status = ConnectionStatus::Failed;
                Err("Connection timeout".into())
            }
        }
    }

    pub async fn disconnect(&mut self) {
        self.status = ConnectionStatus::Disconnected;
        tracing::info!("Disconnected from peer {}", self.peer_id);
    }

    pub fn is_connected(&self) -> bool {
        matches!(
            self.status,
            ConnectionStatus::Connected | ConnectionStatus::Authenticated
        )
    }

    pub fn update_metrics(&mut self, latency_ms: u64, packet_loss: f32) {
        self.metrics.latency_ms = latency_ms;
        self.metrics.packet_loss = packet_loss;
        self.last_seen = chrono::Utc::now();
    }
}
