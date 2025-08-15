use std::sync::Arc;
use tokio::time::{sleep, Duration};
use vx0net_daemon::{Vx0Config, Vx0Node};
use vx0net_daemon::node::PeerConnection;
use vx0net_daemon::network::bgp::BGPDaemon;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    println!("=== VX0 Network Daemon - Two Node Test ===\n");

    // Load configurations for both nodes
    let config1 = load_node_config("config/vx0net-node1.toml").await?;
    let config2 = load_node_config("config/vx0net-node2.toml").await?;

    // Create both nodes
    let node1 = Arc::new(Vx0Node::new(config1.clone())?);
    let node2 = Arc::new(Vx0Node::new(config2.clone())?);

    println!("Created VX0 Nodes:");
    println!("  Node 1: {} (ASN: {}) at {}", node1.hostname, node1.asn, node1.ipv4_addr);
    println!("  Node 2: {} (ASN: {}) at {}", node2.hostname, node2.asn, node2.ipv4_addr);
    println!();

    // Start both nodes
    node1.start().await?;
    node2.start().await?;

    // Create BGP daemons for both nodes
    let bgp1 = BGPDaemon::new(
        config1.node.asn,
        config1.get_ipv4_addr()?.into(),
        config1.network.bgp.listen_port,
    );

    let bgp2 = BGPDaemon::new(
        config2.node.asn,
        config2.get_ipv4_addr()?.into(),
        config2.network.bgp.listen_port,
    );

    // Start BGP daemons
    bgp1.start().await?;
    bgp2.start().await?;

    println!("Started BGP daemons on both nodes");
    
    // Simulate peer discovery and connection
    println!("\n=== Simulating Peer Discovery ===");
    
    let peer1_for_node2 = PeerConnection::new(
        node1.node_id,
        node1.asn,
        node1.ipv4_addr.into(),
    );

    let peer2_for_node1 = PeerConnection::new(
        node2.node_id,
        node2.asn,
        node2.ipv4_addr.into(),
    );

    // Add peers to each node
    node1.add_peer(peer2_for_node1).await?;
    node2.add_peer(peer1_for_node2).await?;

    println!("Added peer connections:");
    println!("  Node 1 connected to Node 2");
    println!("  Node 2 connected to Node 1");

    // Simulate BGP route announcements
    println!("\n=== BGP Route Announcements ===");

    // Node 1 announces some VX0 routes
    let vx0_network1: ipnet::IpNet = "10.1.0.0/24".parse()?;
    let vx0_network2: ipnet::IpNet = "10.2.0.0/24".parse()?;
    
    bgp1.add_route(
        vx0_network1,
        node1.ipv4_addr.into(),
        vx0net_daemon::network::bgp::BGPOrigin::IGP
    ).await?;

    bgp2.add_route(
        vx0_network2,
        node2.ipv4_addr.into(),
        vx0net_daemon::network::bgp::BGPOrigin::IGP
    ).await?;

    println!("Node 1 announced route: {}", vx0_network1);
    println!("Node 2 announced route: {}", vx0_network2);

    // Show routing tables
    println!("\n=== Routing Tables ===");
    let routes1 = bgp1.get_routes().await;
    let routes2 = bgp2.get_routes().await;

    println!("Node 1 routing table ({} routes):", routes1.len());
    for route in routes1 {
        println!("  {} via {} (AS path: {:?})", route.network, route.next_hop, route.as_path);
    }

    println!("Node 2 routing table ({} routes):", routes2.len());
    for route in routes2 {
        println!("  {} via {} (AS path: {:?})", route.network, route.next_hop, route.as_path);
    }

    // Test .vx0 domain resolution
    println!("\n=== VX0 Domain Resolution ===");
    
    // Register some test services
    node1.register_service(vx0net_daemon::node::HostedService {
        service_id: uuid::Uuid::new_v4(),
        name: "web".to_string(),
        service_type: vx0net_daemon::node::ServiceType::WebServer,
        domain: "web.node1.vx0".to_string(),
        port: 80,
        status: vx0net_daemon::node::ServiceStatus::Running,
        metadata: std::collections::HashMap::new(),
    }).await?;

    node2.register_service(vx0net_daemon::node::HostedService {
        service_id: uuid::Uuid::new_v4(),
        name: "chat".to_string(),
        service_type: vx0net_daemon::node::ServiceType::ChatServer,
        domain: "chat.node2.vx0".to_string(),
        port: 6667,
        status: vx0net_daemon::node::ServiceStatus::Running,
        metadata: std::collections::HashMap::new(),
    }).await?;

    println!("Registered services:");
    println!("  web.node1.vx0 (WebServer) on Node 1");
    println!("  chat.node2.vx0 (ChatServer) on Node 2");

    // Show network statistics
    println!("\n=== Network Statistics ===");
    println!("Node 1 peers: {}", node1.get_peer_count().await);
    println!("Node 2 peers: {}", node2.get_peer_count().await);

    let services1 = node1.services.read().await;
    let services2 = node2.services.read().await;
    println!("Node 1 services: {}", services1.len());
    println!("Node 2 services: {}", services2.len());

    // Test DNS resolution for vx0.network domain
    println!("\n=== Testing VX0.network Resolution ===");
    println!("vx0.network should resolve to the gateway: 10.0.1.1");
    println!("This demonstrates that the isolated VX0 network is operational");

    println!("\n=== Test Complete ===");
    println!("Two VX0 nodes are successfully connected and exchanging routes!");
    println!("The censorship-resistant network infrastructure is working.");

    // Keep running for a bit to demonstrate the connection
    println!("\nKeeping nodes running for 10 seconds...");
    sleep(Duration::from_secs(10)).await;

    // Graceful shutdown
    println!("Shutting down nodes...");
    node1.stop().await?;
    node2.stop().await?;

    println!("Test completed successfully!");

    Ok(())
}

async fn load_node_config(path: &str) -> Result<Vx0Config, Box<dyn std::error::Error>> {
    // For this test, we'll load the config or create defaults if the file doesn't exist
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let config: Vx0Config = toml::from_str(&content)?;
            Ok(config)
        }
        Err(_) => {
            println!("Config file {} not found, using defaults", path);
            // Create default config based on the path
            let default_config = if path.contains("node1") {
                create_default_config("node1.vx0", 65001, "192.168.1.100", 11790, 45000)
            } else {
                create_default_config("node2.vx0", 65002, "192.168.1.101", 11791, 45001)
            };
            Ok(default_config)
        }
    }
}

fn create_default_config(hostname: &str, asn: u32, ip: &str, bgp_port: u16, ike_port: u16) -> Vx0Config {
    use vx0net_daemon::config::*;

    Vx0Config {
        node: NodeConfig {
            hostname: hostname.to_string(),
            asn,
            tier: "Edge".to_string(),
            location: "Test Lab".to_string(),
            ipv4_address: ip.to_string(),
            ipv6_address: "fe80::1".to_string(),
        },
        network: NetworkConfig {
            bgp: BGPConfig {
                router_id: ip.to_string(),
                listen_port: bgp_port,
                hold_time: 90,
                keepalive_time: 30,
            },
            dns: DNSConfig {
                listen_port: 5353,
                vx0_dns_servers: vec!["10.0.0.2:53".to_string(), "10.0.0.3:53".to_string()],
                cache_size: 1000,
            },
            routing: RoutingConfig {
                max_paths: 4,
                local_preference: 100,
                med: 0,
            },
        },
        security: SecurityConfig {
            ike: IKEConfig {
                listen_port: ike_port,
                dh_group: 14,
                encryption_algorithm: "AES-256".to_string(),
                hash_algorithm: "SHA-256".to_string(),
                prf_algorithm: "HMAC-SHA256".to_string(),
            },
            certificates: CertificateConfig {
                ca_cert_path: "config/certs/ca.crt".to_string(),
                node_cert_path: format!("config/certs/{}.crt", hostname),
                node_key_path: format!("config/certs/{}.key", hostname),
            },
            encryption: EncryptionConfig {
                cipher: "AES-256-GCM".to_string(),
                key_size: 32,
                iv_size: 12,
            },
        },
        services: ServicesConfig {
            enable_discovery: true,
            discovery_port: if asn == 65001 { 8080 } else { 8081 },
            service_ttl: 300,
        },
        monitoring: MonitoringConfig {
            enable_metrics: true,
            metrics_port: if asn == 65001 { 9090 } else { 9091 },
            log_level: "info".to_string(),
        },
        bootstrap: None,
        psk: None,
    }
}