use crate::network::dns::{DNSError, DNSRecord, RecordType, Vx0DNS};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct Vx0DNSServer {
    dns: Vx0DNS,
    bind_addr: SocketAddr,
}

impl Vx0DNSServer {
    pub fn new(bind_addr: SocketAddr) -> Self {
        Vx0DNSServer {
            dns: Vx0DNS::new(),
            bind_addr,
        }
    }

    pub async fn start(&mut self) -> Result<(), DNSError> {
        let socket = UdpSocket::bind(self.bind_addr).await?;
        tracing::info!("VX0 DNS server started on {}", self.bind_addr);

        let mut buf = [0; 512];

        loop {
            match socket.recv_from(&mut buf).await {
                Ok((size, client_addr)) => {
                    tracing::debug!("DNS query from {} ({} bytes)", client_addr, size);

                    if let Err(e) = self.handle_query(&socket, &buf[..size], client_addr).await {
                        tracing::error!("Error handling DNS query: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("DNS server socket error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_query(
        &self,
        socket: &UdpSocket,
        query: &[u8],
        client_addr: SocketAddr,
    ) -> Result<(), DNSError> {
        // Simplified DNS query handling
        // In a real implementation, we would parse the DNS packet format

        let query_str = String::from_utf8_lossy(query);
        tracing::debug!("DNS query content: {}", query_str);

        // For testing purposes, let's simulate some common queries
        let response = if query_str.contains("vx0.network") {
            self.create_response("vx0.network", "10.0.1.1")
        } else if query_str.contains("gateway.vx0") {
            self.create_response("gateway.vx0", "10.0.0.1")
        } else if query_str.contains("node1.vx0") {
            self.create_response("node1.vx0", "10.0.2.1")
        } else if query_str.contains("node2.vx0") {
            self.create_response("node2.vx0", "10.0.2.2")
        } else {
            // Return NXDOMAIN response
            b"NXDOMAIN".to_vec()
        };

        socket.send_to(&response, client_addr).await?;
        tracing::debug!("Sent DNS response to {}", client_addr);

        Ok(())
    }

    fn create_response(&self, domain: &str, ip: &str) -> Vec<u8> {
        // This is a simplified response - in a real implementation,
        // we would create proper DNS response packets
        format!("{} IN A {}", domain, ip).into_bytes()
    }

    pub fn register_service(
        &mut self,
        domain: String,
        ip: std::net::IpAddr,
    ) -> Result<(), DNSError> {
        self.dns.register_service(domain, ip)
    }

    pub fn add_record(&mut self, record: DNSRecord) {
        let domain = record.name.clone();
        self.dns.records.entry(domain).or_default().push(record);
    }

    pub fn get_records(&self, domain: &str) -> Option<&Vec<DNSRecord>> {
        self.dns.get_records(domain)
    }

    pub fn create_vx0_network_record(&mut self) -> Result<(), DNSError> {
        // Check if vx0.network record already exists
        if self.get_records("vx0.network").is_some() {
            tracing::debug!("vx0.network DNS record already exists");
            return Ok(());
        }

        let record = DNSRecord {
            name: "vx0.network".to_string(),
            record_type: RecordType::A,
            data: "10.0.1.1".to_string(),
            ttl: 300,
            timestamp: chrono::Utc::now(),
        };

        self.add_record(record);
        tracing::info!("Created vx0.network DNS record");
        Ok(())
    }

    pub fn create_node_records(&mut self, node_count: u8) -> Result<(), DNSError> {
        for i in 1..=node_count {
            let record = DNSRecord {
                name: format!("node{}.vx0", i),
                record_type: RecordType::A,
                data: format!("10.0.2.{}", i),
                ttl: 300,
                timestamp: chrono::Utc::now(),
            };

            self.add_record(record);
            tracing::info!("Created node{}.vx0 DNS record", i);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_dns_server_creation() {
        let addr = "127.0.0.1:53".parse().unwrap();
        let server = Vx0DNSServer::new(addr);
        assert_eq!(server.bind_addr, addr);
    }

    #[test]
    fn test_record_creation() {
        let mut server = Vx0DNSServer::new("127.0.0.1:53".parse().unwrap());

        let result = server.register_service(
            "test.vx0".to_string(),
            IpAddr::V4(Ipv4Addr::new(10, 0, 3, 1)),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_vx0_network_record() {
        let mut server = Vx0DNSServer::new("127.0.0.1:53".parse().unwrap());
        let result = server.create_vx0_network_record();
        assert!(result.is_ok());

        let records = server.get_records("vx0.network");
        assert!(records.is_some());

        if let Some(records) = records {
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].data, "10.0.1.1");
        }
    }
}
