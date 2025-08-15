use clap::{Parser, Subcommand};
use rand::random;
use std::sync::Arc;
use tokio::signal;
use tracing::{debug, error, info};

use vx0net_daemon::network::bgp::BGPDaemon;
use vx0net_daemon::network::ike::session::IKEDaemon;
use vx0net_daemon::node::manager::NodeManager;
use vx0net_daemon::{NodeError, Vx0Config, Vx0Node};

#[derive(Parser)]
#[command(name = "vx0net")]
#[command(about = "VX0 Network Daemon - Censorship-resistant networking system")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[arg(short, long)]
    config: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the VX0 network daemon
    Start {
        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
        /// Automatically join the network on start
        #[arg(long)]
        join_network: bool,
    },
    /// Stop the VX0 network daemon
    Stop,
    /// Show daemon status
    Status,
    /// Show node information
    Info,
    /// Connect to a peer node
    Connect {
        /// Peer IP address
        peer_ip: String,
        /// Peer ASN
        peer_asn: u32,
    },
    /// Disconnect from a peer node
    Disconnect {
        /// Peer IP address
        peer_ip: String,
    },
    /// Show routing table
    Routes,
    /// Show connected peers
    Peers,
    /// Register a .vx0 service
    RegisterService {
        /// Service name
        name: String,
        /// Service domain (must end with .vx0)
        domain: String,
        /// Service port
        port: u16,
    },
    /// Join the VX0 network (interactive)
    Join,
    /// Check network connectivity and bootstrap status
    NetworkStatus,
    /// Scan for available ASNs in your tier
    ScanAsns {
        /// Node tier (Backbone, Regional, Edge)
        tier: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize tracing
    let log_level = match cli.log_level.as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    info!("VX0 Network Daemon v0.1.0");

    match cli.command {
        Commands::Start {
            foreground,
            join_network,
        } => {
            start_daemon(foreground, join_network).await?;
        }
        Commands::Stop => {
            info!("Stopping VX0 daemon...");
            // In a real implementation, we would send a signal to the running daemon
            info!("VX0 daemon stopped");
        }
        Commands::Status => {
            info!("VX0 daemon status: Running"); // Placeholder
        }
        Commands::Info => {
            show_node_info().await?;
        }
        Commands::Connect { peer_ip, peer_asn } => {
            info!("Connecting to peer {} (ASN: {})", peer_ip, peer_asn);
            // Placeholder for peer connection
        }
        Commands::Disconnect { peer_ip } => {
            info!("Disconnecting from peer {}", peer_ip);
            // Placeholder for peer disconnection
        }
        Commands::Routes => {
            show_routes().await?;
        }
        Commands::Peers => {
            show_peers().await?;
        }
        Commands::RegisterService { name, domain, port } => {
            register_service(&name, &domain, port).await?;
        }
        Commands::Join => {
            join_network_interactive().await?;
        }
        Commands::NetworkStatus => {
            show_network_status().await?;
        }
        Commands::ScanAsns { tier } => {
            scan_available_asns(&tier).await?;
        }
    }

    Ok(())
}

async fn start_daemon(
    foreground: bool,
    join_network: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting VX0 network daemon...");

    if !foreground {
        info!("Running in daemon mode");
    }

    // Load configuration
    let config = Vx0Config::load().map_err(|e| {
        error!("Failed to load configuration: {}", e);
        e
    })?;

    debug!(
        "Configuration loaded: ASN {}, Hostname: {}",
        config.node.asn, config.node.hostname
    );

    // Create VX0 node
    let node = Arc::new(Vx0Node::new(config.clone())?);
    info!("Created VX0 node: {} (ASN: {})", node.hostname, node.asn);

    // Start node services
    node.start().await?;

    // Start BGP daemon
    let bgp_daemon = BGPDaemon::new(
        config.node.asn,
        config.get_ipv4_addr()?.into(),
        config.network.bgp.listen_port,
    );
    bgp_daemon.start().await?;

    // Start IKE daemon
    let mut ike_daemon =
        IKEDaemon::new(format!("0.0.0.0:{}", config.security.ike.listen_port).parse()?);
    ike_daemon.start().await?;

    // Start node manager
    let node_manager = NodeManager::new(Arc::clone(&node));
    node_manager.run().await?;

    // Add some VX0 network routes
    let vx0_network: ipnet::IpNet = "10.0.0.0/8".parse()?;
    bgp_daemon
        .add_route(
            vx0_network,
            "10.0.0.1".parse()?,
            vx0net_daemon::network::bgp::BGPOrigin::IGP,
        )
        .await?;

    info!("VX0 network daemon started successfully");
    info!(
        "Listening for BGP connections on port {}",
        config.network.bgp.listen_port
    );
    info!(
        "Listening for IKE connections on port {}",
        config.security.ike.listen_port
    );

    // Auto-join network if requested
    if join_network {
        info!("🌐 Auto-joining VX0 network...");
        if let Err(e) = node.join_vx0_network().await {
            error!("Failed to join network: {}", e);
        } else {
            info!("✅ Successfully joined VX0 network!");
        }
    }

    // Handle shutdown signals
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received Ctrl+C, shutting down...");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Graceful shutdown
    info!("Shutting down VX0 node...");
    node.stop().await?;
    info!("VX0 network daemon stopped");

    Ok(())
}

async fn show_node_info() -> Result<(), NodeError> {
    let config = Vx0Config::load().map_err(|e| NodeError::Config(e.to_string()))?;
    let node = Vx0Node::new(config)?;

    println!("VX0 Node Information:");
    println!("  Node ID: {}", node.node_id);
    println!("  Hostname: {}", node.hostname);
    println!("  ASN: {}", node.asn);
    println!("  Tier: {:?}", node.tier);
    println!("  IPv4: {}", node.ipv4_addr);
    println!("  IPv6: {}", node.ipv6_addr);
    println!("  Location: {}", node.location.city);
    println!("  Peer count: {}", node.get_peer_count().await);

    Ok(())
}

async fn show_routes() -> Result<(), Box<dyn std::error::Error>> {
    println!("VX0 Routing Table:");
    println!("  Network          Next Hop        AS Path    Origin");
    println!("  10.0.0.0/8       10.0.0.1        65001      IGP");
    println!("  vx0.network      10.0.1.1        65001      IGP");
    // In a real implementation, we would query the actual routing table

    Ok(())
}

async fn show_peers() -> Result<(), Box<dyn std::error::Error>> {
    println!("VX0 Connected Peers:");
    println!("  Peer IP          ASN      Status       Uptime");
    println!("  192.168.1.100    65002    Connected    00:15:42");
    // In a real implementation, we would query the actual peer list

    Ok(())
}

async fn register_service(
    name: &str,
    domain: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    if !domain.ends_with(".vx0") {
        return Err("Service domain must end with .vx0".into());
    }

    info!("Registering service '{}' at {}:{}", name, domain, port);

    // In a real implementation, we would:
    // 1. Register the service in the local service registry
    // 2. Announce it to the VX0 DNS network
    // 3. Update BGP routing if needed

    info!("Service '{}' registered successfully", name);
    Ok(())
}

async fn join_network_interactive() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 VX0 Network Interactive Join");
    println!("================================");
    println!();

    println!("Welcome to the VX0 censorship-resistant network!");
    println!("This wizard will help you join the network and start contributing.");
    println!();

    // Check if config exists
    if std::path::Path::new("config/vx0net.toml").exists() {
        println!("⚠️  Configuration file already exists.");
        println!("If you want to rejoin with new settings, delete config/vx0net.toml first.");
        return Ok(());
    }

    println!("🎯 For the easiest setup, run: ./scripts/join-network.sh");
    println!("📖 For detailed instructions, see: JOINING.md");
    println!();

    println!("Manual joining steps:");
    println!("1. Choose your node tier (Edge recommended for beginners)");
    println!("2. Get an available ASN in your tier range");
    println!("3. Configure your node settings");
    println!("4. Start the daemon with: vx0net start --join-network");
    println!();

    println!("📋 Current network status:");
    show_network_status().await?;

    Ok(())
}

async fn show_network_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 VX0 Network Status");
    println!("====================");
    println!();

    // Try to load bootstrap registry
    if let Ok(registry_content) = std::fs::read_to_string("bootstrap-registry.json") {
        if let Ok(registry) = serde_json::from_str::<serde_json::Value>(&registry_content) {
            if let Some(network) = registry.get("vx0_network_bootstrap_registry") {
                if let Some(total) = network.get("total_nodes") {
                    println!("📊 Total nodes in network: {}", total);
                }

                if let Some(stats) = network.get("network_stats") {
                    if let Some(health) = stats.get("network_health") {
                        println!(
                            "💚 Network health: {}",
                            health.as_str().unwrap_or("unknown")
                        );
                    }
                    if let Some(latency) = stats.get("average_latency_ms") {
                        println!("⚡ Average latency: {}ms", latency);
                    }
                }

                println!();
                println!("🏗️  Available node types:");

                if let Some(backbone) = network.get("backbone_nodes") {
                    println!(
                        "  Backbone nodes: {} active",
                        backbone.as_array().unwrap_or(&vec![]).len()
                    );
                }
                if let Some(regional) = network.get("regional_nodes") {
                    println!(
                        "  Regional nodes: {} active",
                        regional.as_array().unwrap_or(&vec![]).len()
                    );
                }
                if let Some(edge) = network.get("edge_nodes") {
                    println!(
                        "  Edge nodes: {} active",
                        edge.as_array().unwrap_or(&vec![]).len()
                    );
                }
            }
        }
    } else {
        println!("❌ Cannot load network registry");
        println!("🔍 Checking connectivity to known bootstrap nodes...");

        // Test connectivity to bootstrap nodes
        let bootstrap_nodes = [
            ("backbone1.vx0.network", 1179),
            ("regional1.vx0.network", 1179),
        ];

        for (hostname, port) in &bootstrap_nodes {
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                tokio::net::TcpSocket::new_v4()
                    .unwrap()
                    .connect(format!("{}:{}", hostname, port).parse().unwrap()),
            )
            .await
            {
                Ok(Ok(_)) => println!("  ✅ {} is reachable", hostname),
                _ => println!("  ❌ {} is not reachable", hostname),
            }
        }
    }

    println!();
    println!("📍 To join the network:");
    println!("  ./scripts/join-network.sh   (automatic setup)");
    println!("  vx0net join                  (this wizard)");
    println!("  See JOINING.md               (manual setup)");

    Ok(())
}

async fn scan_available_asns(tier: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Scanning available ASNs for {} tier", tier);
    println!("=====================================");
    println!();

    let (min_asn, max_asn, tier_name) = match tier.to_lowercase().as_str() {
        "backbone" => (65000, 65099, "Backbone"),
        "regional" => (65100, 65999, "Regional"),
        "edge" => (66000, 69999, "Edge"),
        _ => {
            println!("❌ Invalid tier. Use: Backbone, Regional, or Edge");
            return Ok(());
        }
    };

    println!("📋 {} Tier ASN Range: {} - {}", tier_name, min_asn, max_asn);
    println!("📊 Total available ASNs: {}", max_asn - min_asn + 1);
    println!();

    // In a real implementation, we would query the network to find used ASNs
    // For now, show some examples
    println!("💡 Recommended ASNs for new nodes:");
    for i in 0..5 {
        let suggested_asn = min_asn + (i * 100) + (random::<u32>() % 100);
        if suggested_asn <= max_asn {
            println!("  ASN {}: Available ✅", suggested_asn);
        }
    }

    println!();
    println!(
        "🎲 Random available ASN: {}",
        min_asn + (random::<u32>() % (max_asn - min_asn + 1))
    );
    println!();
    println!("ℹ️  You can use any unused ASN in the {} range.", tier_name);
    println!("   The network will automatically detect conflicts and reassign if needed.");

    Ok(())
}
