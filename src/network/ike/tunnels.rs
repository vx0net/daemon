use crate::network::ike::{IKEError, IKESession};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type TunnelId = Uuid;

#[derive(Debug, Clone)]
pub struct IPSecTunnel {
    pub tunnel_id: TunnelId,
    pub local_addr: IpAddr,
    pub remote_addr: IpAddr,
    pub ike_session: IKESession,
    pub status: TunnelStatus,
    pub traffic_stats: TrafficStats,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum TunnelStatus {
    Negotiating,
    Established,
    Rekeying,
    Failed,
    Closed,
}

#[derive(Debug, Clone)]
pub struct TrafficStats {
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub packets_in: u64,
    pub packets_out: u64,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct TunnelManager {
    tunnels: Arc<RwLock<HashMap<TunnelId, IPSecTunnel>>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        TunnelManager {
            tunnels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_tunnel(
        &self,
        local_addr: IpAddr,
        remote_addr: IpAddr,
        peer_addr: SocketAddr,
        psk: &[u8],
    ) -> Result<TunnelId, IKEError> {
        let tunnel_id = Uuid::new_v4();

        tracing::info!("Creating IPSec tunnel {} to {}", tunnel_id, remote_addr);

        let mut ike_session = IKESession::new(peer_addr, 14)?; // DH Group 14
        ike_session.establish_tunnel(psk).await?;

        let tunnel = IPSecTunnel {
            tunnel_id,
            local_addr,
            remote_addr,
            ike_session,
            status: TunnelStatus::Established,
            traffic_stats: TrafficStats::new(),
            created_at: chrono::Utc::now(),
        };

        let mut tunnels = self.tunnels.write().await;
        tunnels.insert(tunnel_id, tunnel);

        tracing::info!("IPSec tunnel {} established successfully", tunnel_id);
        Ok(tunnel_id)
    }

    pub async fn close_tunnel(&self, tunnel_id: &TunnelId) -> Result<(), IKEError> {
        let mut tunnels = self.tunnels.write().await;

        if let Some(mut tunnel) = tunnels.remove(tunnel_id) {
            tunnel.ike_session.close().await?;
            tunnel.status = TunnelStatus::Closed;
            tracing::info!("Closed tunnel {}", tunnel_id);
        }

        Ok(())
    }

    pub async fn get_tunnel(&self, tunnel_id: &TunnelId) -> Option<IPSecTunnel> {
        let tunnels = self.tunnels.read().await;
        tunnels.get(tunnel_id).cloned()
    }

    pub async fn list_tunnels(&self) -> Vec<IPSecTunnel> {
        let tunnels = self.tunnels.read().await;
        tunnels.values().cloned().collect()
    }

    pub async fn send_packet(&self, tunnel_id: &TunnelId, packet: &[u8]) -> Result<(), IKEError> {
        let mut tunnels = self.tunnels.write().await;

        if let Some(tunnel) = tunnels.get_mut(tunnel_id) {
            if !matches!(tunnel.status, TunnelStatus::Established) {
                return Err(IKEError::Protocol("Tunnel not established".to_string()));
            }

            // Encrypt the packet
            let encrypted_packet = tunnel.ike_session.encrypt_payload(packet)?;

            // In a real implementation, we would send this through a raw socket or TUN interface
            tracing::debug!(
                "Sending encrypted packet through tunnel {} ({} bytes)",
                tunnel_id,
                encrypted_packet.len()
            );

            // Update traffic stats
            tunnel.traffic_stats.bytes_out += encrypted_packet.len() as u64;
            tunnel.traffic_stats.packets_out += 1;
            tunnel.traffic_stats.last_activity = chrono::Utc::now();
        } else {
            return Err(IKEError::Protocol("Tunnel not found".to_string()));
        }

        Ok(())
    }

    pub async fn receive_packet(
        &self,
        tunnel_id: &TunnelId,
        encrypted_packet: &[u8],
    ) -> Result<Vec<u8>, IKEError> {
        let mut tunnels = self.tunnels.write().await;

        if let Some(tunnel) = tunnels.get_mut(tunnel_id) {
            if !matches!(tunnel.status, TunnelStatus::Established) {
                return Err(IKEError::Protocol("Tunnel not established".to_string()));
            }

            // Decrypt the packet
            let decrypted_packet = tunnel.ike_session.decrypt_payload(encrypted_packet)?;

            tracing::debug!(
                "Received and decrypted packet through tunnel {} ({} bytes)",
                tunnel_id,
                decrypted_packet.len()
            );

            // Update traffic stats
            tunnel.traffic_stats.bytes_in += encrypted_packet.len() as u64;
            tunnel.traffic_stats.packets_in += 1;
            tunnel.traffic_stats.last_activity = chrono::Utc::now();

            Ok(decrypted_packet)
        } else {
            Err(IKEError::Protocol("Tunnel not found".to_string()))
        }
    }

    pub async fn rekey_tunnel(&self, tunnel_id: &TunnelId) -> Result<(), IKEError> {
        let mut tunnels = self.tunnels.write().await;

        if let Some(tunnel) = tunnels.get_mut(tunnel_id) {
            tunnel.status = TunnelStatus::Rekeying;
            tunnel.ike_session.rekey().await?;
            tunnel.status = TunnelStatus::Established;

            tracing::info!("Rekeyed tunnel {}", tunnel_id);
        }

        Ok(())
    }

    pub async fn get_tunnel_stats(&self, tunnel_id: &TunnelId) -> Option<TrafficStats> {
        let tunnels = self.tunnels.read().await;
        tunnels.get(tunnel_id).map(|t| t.traffic_stats.clone())
    }

    pub async fn cleanup_failed_tunnels(&self) {
        let mut tunnels = self.tunnels.write().await;
        let failed_tunnels: Vec<TunnelId> = tunnels
            .iter()
            .filter_map(|(id, tunnel)| {
                if matches!(tunnel.status, TunnelStatus::Failed) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        for tunnel_id in failed_tunnels {
            tunnels.remove(&tunnel_id);
            tracing::info!("Cleaned up failed tunnel {}", tunnel_id);
        }
    }
}

impl Default for TrafficStats {
    fn default() -> Self {
        Self::new()
    }
}

impl TrafficStats {
    pub fn new() -> Self {
        TrafficStats {
            bytes_in: 0,
            bytes_out: 0,
            packets_in: 0,
            packets_out: 0,
            last_activity: chrono::Utc::now(),
        }
    }
}

impl Default for TunnelManager {
    fn default() -> Self {
        Self::new()
    }
}
