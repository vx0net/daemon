use std::collections::HashMap;
use std::net::IpAddr;
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;

pub mod resolver;
pub mod server;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vx0DNS {
    pub zones: HashMap<String, DNSZone>,
    pub records: HashMap<String, Vec<DNSRecord>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DNSZone {
    pub name: String,
    pub soa: SOARecord,
    pub ns_records: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SOARecord {
    pub primary: String,
    pub email: String,
    pub serial: u32,
    pub refresh: u32,
    pub retry: u32,
    pub expire: u32,
    pub minimum: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DNSRecord {
    pub name: String,
    pub record_type: RecordType,
    pub data: String,
    pub ttl: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordType {
    A,
    AAAA,
    CNAME,
    MX,
    TXT,
    SRV,
    PTR,
}

#[derive(Debug, thiserror::Error)]
pub enum DNSError {
    #[error("Invalid domain: {0}")]
    InvalidDomain(String),
    #[error("Record not found: {0}")]
    RecordNotFound(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}

impl Vx0DNS {
    pub fn new() -> Self {
        let mut dns = Vx0DNS {
            zones: HashMap::new(),
            records: HashMap::new(),
        };

        // Create the root VX0 zone
        dns.create_vx0_zone();
        dns
    }

    fn create_vx0_zone(&mut self) {
        let vx0_zone = DNSZone {
            name: "vx0".to_string(),
            soa: SOARecord {
                primary: "ns1.vx0".to_string(),
                email: "admin.vx0".to_string(),
                serial: 1,
                refresh: 3600,
                retry: 1800,
                expire: 604800,
                minimum: 86400,
            },
            ns_records: vec![
                "ns1.vx0".to_string(),
                "ns2.vx0".to_string(),
            ],
        };

        self.zones.insert("vx0".to_string(), vx0_zone);

        // Add some default records
        self.add_record(DNSRecord {
            name: "gateway.vx0".to_string(),
            record_type: RecordType::A,
            data: "10.0.0.1".to_string(),
            ttl: 300,
            timestamp: chrono::Utc::now(),
        });

        self.add_record(DNSRecord {
            name: "ns1.vx0".to_string(),
            record_type: RecordType::A,
            data: "10.0.0.2".to_string(),
            ttl: 300,
            timestamp: chrono::Utc::now(),
        });

        self.add_record(DNSRecord {
            name: "ns2.vx0".to_string(),
            record_type: RecordType::A,
            data: "10.0.0.3".to_string(),
            ttl: 300,
            timestamp: chrono::Utc::now(),
        });

        // Add vx0.network record
        self.add_record(DNSRecord {
            name: "vx0.network".to_string(),
            record_type: RecordType::A,
            data: "10.0.1.1".to_string(),
            ttl: 300,
            timestamp: chrono::Utc::now(),
        });
    }

    pub async fn resolve_vx0_domain(&self, domain: &str) -> Option<IpAddr> {
        tracing::debug!("Resolving VX0 domain: {}", domain);

        if !domain.ends_with(".vx0") && domain != "vx0.network" {
            return None;
        }

        // Query internal DNS records
        if let Some(records) = self.records.get(domain) {
            for record in records {
                if matches!(record.record_type, RecordType::A) {
                    if let Ok(ip) = record.data.parse::<IpAddr>() {
                        tracing::info!("Resolved {} to {}", domain, ip);
                        return Some(ip);
                    }
                }
            }
        }

        // Query distributed DNS network
        self.query_distributed_dns(domain).await
    }

    async fn query_distributed_dns(&self, domain: &str) -> Option<IpAddr> {
        tracing::debug!("Querying distributed DNS for {}", domain);
        
        // For now, return a placeholder IP for vx0.network
        if domain == "vx0.network" {
            return Some("10.0.1.1".parse().unwrap());
        }

        // In a real implementation, we would query other VX0 nodes
        None
    }

    pub fn register_service(&mut self, domain: String, ip: IpAddr) -> Result<(), DNSError> {
        if !domain.ends_with(".vx0") && domain != "vx0.network" {
            return Err(DNSError::InvalidDomain(domain));
        }

        let record = DNSRecord {
            name: domain.clone(),
            record_type: RecordType::A,
            data: ip.to_string(),
            ttl: 300,
            timestamp: chrono::Utc::now(),
        };

        self.add_record(record);
        tracing::info!("Registered service {} -> {}", domain, ip);

        Ok(())
    }

    fn add_record(&mut self, record: DNSRecord) {
        let domain = record.name.clone();
        self.records
            .entry(domain)
            .or_insert_with(Vec::new)
            .push(record);
    }

    pub fn get_records(&self, domain: &str) -> Option<&Vec<DNSRecord>> {
        self.records.get(domain)
    }

    pub async fn start_server(&self, bind_addr: &str) -> Result<(), DNSError> {
        let socket = UdpSocket::bind(bind_addr).await?;
        tracing::info!("VX0 DNS server listening on {}", bind_addr);

        let mut buf = [0; 512];
        
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((size, addr)) => {
                    tracing::debug!("DNS query from {} ({} bytes)", addr, size);
                    
                    // In a real implementation, we would parse the DNS query
                    // and respond with appropriate DNS records
                    
                    // For now, just log the query
                }
                Err(e) => {
                    tracing::error!("DNS server error: {}", e);
                }
            }
        }
    }
}

impl Default for Vx0DNS {
    fn default() -> Self {
        Self::new()
    }
}