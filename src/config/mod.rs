use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Vx0Config {
    pub node: NodeConfig,
    pub network: NetworkConfig,
    pub security: SecurityConfig,
    pub services: ServicesConfig,
    pub monitoring: MonitoringConfig,
    #[serde(default)]
    pub bootstrap: Option<BootstrapConfig>,
    #[serde(default)]
    pub psk: Option<PSKConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NodeConfig {
    pub hostname: String,
    pub asn: u32,
    pub tier: String,
    pub location: String,
    pub ipv4_address: String,
    pub ipv6_address: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkConfig {
    pub bgp: BGPConfig,
    pub dns: DNSConfig,
    pub routing: RoutingConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BGPConfig {
    pub router_id: String,
    pub listen_port: u16,
    pub hold_time: u16,
    pub keepalive_time: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DNSConfig {
    pub listen_port: u16,
    pub vx0_dns_servers: Vec<String>, // Only VX0 internal DNS servers
    pub cache_size: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RoutingConfig {
    pub max_paths: u8,
    pub local_preference: u32,
    pub med: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecurityConfig {
    pub ike: IKEConfig,
    pub certificates: CertificateConfig,
    pub encryption: EncryptionConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IKEConfig {
    pub listen_port: u16,
    pub dh_group: u8,
    pub encryption_algorithm: String,
    pub hash_algorithm: String,
    pub prf_algorithm: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CertificateConfig {
    pub ca_cert_path: String,
    pub node_cert_path: String,
    pub node_key_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EncryptionConfig {
    pub cipher: String,
    pub key_size: u32,
    pub iv_size: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServicesConfig {
    pub enable_discovery: bool,
    pub discovery_port: u16,
    pub service_ttl: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub metrics_port: u16,
    pub log_level: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BootstrapConfig {
    pub nodes: Vec<BootstrapNode>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BootstrapNode {
    pub hostname: String,
    pub ip: String,
    pub asn: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PSKConfig {
    pub default: String,
}

impl Vx0Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("vx0net.toml").required(false))
            .add_source(File::with_name("/etc/vx0net/config.toml").required(false))
            .add_source(Environment::with_prefix("VX0NET"))
            .set_default("node.hostname", "vx0-node")?
            .set_default("node.asn", 65001)?
            .set_default("node.tier", "Edge")?
            .set_default("node.location", "Unknown")?
            .set_default("node.ipv4_address", "192.168.1.100")?
            .set_default("node.ipv6_address", "fe80::1")?
            .set_default("network.bgp.router_id", "192.168.1.100")?
            .set_default("network.bgp.listen_port", 179)?
            .set_default("network.bgp.hold_time", 90)?
            .set_default("network.bgp.keepalive_time", 30)?
            .set_default("network.dns.listen_port", 53)?
            .set_default(
                "network.dns.vx0_dns_servers",
                vec!["10.0.0.2:53", "10.0.0.3:53"],
            )?
            .set_default("network.dns.cache_size", 1000)?
            .set_default("network.routing.max_paths", 4)?
            .set_default("network.routing.local_preference", 100)?
            .set_default("network.routing.med", 0)?
            .set_default("security.ike.listen_port", 500)?
            .set_default("security.ike.dh_group", 14)?
            .set_default("security.ike.encryption_algorithm", "AES-256")?
            .set_default("security.ike.hash_algorithm", "SHA-256")?
            .set_default("security.ike.prf_algorithm", "HMAC-SHA256")?
            .set_default("security.certificates.ca_cert_path", "/etc/vx0net/ca.crt")?
            .set_default(
                "security.certificates.node_cert_path",
                "/etc/vx0net/node.crt",
            )?
            .set_default(
                "security.certificates.node_key_path",
                "/etc/vx0net/node.key",
            )?
            .set_default("security.encryption.cipher", "AES-256-GCM")?
            .set_default("security.encryption.key_size", 32)?
            .set_default("security.encryption.iv_size", 12)?
            .set_default("services.enable_discovery", true)?
            .set_default("services.discovery_port", 8080)?
            .set_default("services.service_ttl", 300)?
            .set_default("monitoring.enable_metrics", true)?
            .set_default("monitoring.metrics_port", 9090)?
            .set_default("monitoring.log_level", "info")?
            .build()?;

        config.try_deserialize()
    }

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let toml_content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, toml_content)?;
        Ok(())
    }

    pub fn get_ipv4_addr(&self) -> Result<Ipv4Addr, std::net::AddrParseError> {
        self.node.ipv4_address.parse()
    }

    pub fn get_ipv6_addr(&self) -> Result<Ipv6Addr, std::net::AddrParseError> {
        self.node.ipv6_address.parse()
    }
}
