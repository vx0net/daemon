# 🌐 VX0 Network Daemon

**A decentralized, censorship-resistant network infrastructure built with Rust**

[![Build Status](https://github.com/vx0net/daemon/workflows/CI/badge.svg)](https://github.com/vx0net/daemon/actions)
[![Docker](https://img.shields.io/docker/v/ghcr.io/vx0net/daemon?label=docker)](https://github.com/vx0net/daemon/pkgs/container/daemon)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **Join the decentralized internet in under 30 seconds**

## 🚀 Quick Start

### One-Command Installation
```bash
curl -fsSL https://raw.githubusercontent.com/vx0net/daemon/main/install-vx0.sh | bash
```

That's it! Your Edge Node will be running and connected to the VX0 network automatically.

### What You Get
- 🌐 **Instant Network Access** - Connect to the decentralized VX0 network
- 🔒 **Built-in Security** - Encrypted tunnels with automatic certificate management
- 📊 **Web Dashboard** - Beautiful interface at `http://localhost:8090`
- 🔄 **Zero Maintenance** - Automatic updates and self-healing
- 🆘 **24/7 Support** - Built-in help and community support

## 🏗️ What is VX0 Network?

VX0 Network is a **decentralized internet infrastructure** that provides:

- **Censorship Resistance** - No central authority can block or monitor traffic
- **Global Connectivity** - Peer-to-peer connections across the world
- **Privacy by Design** - End-to-end encryption for all communications
- **Self-Organizing** - Automatic peer discovery and network healing
- **Open Source** - Fully transparent and community-driven

### Network Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Backbone      │────│    Regional     │────│     Edge        │
│   Nodes         │    │     Nodes       │    │    Nodes        │
│                 │    │                 │    │                 │
│ ASN: 65000-65099│    │ ASN: 65100-65999│    │ ASN: 66000-69999│
│ Global Routing  │    │ Regional Hubs   │    │ End Users       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

- **Backbone Nodes** - High-capacity routing infrastructure
- **Regional Nodes** - Geographic distribution points  
- **Edge Nodes** - User endpoints and local services

## 📦 Installation Methods

### 🌟 Ultra-Simple (Recommended)
Perfect for non-technical users:
```bash
curl -fsSL https://raw.githubusercontent.com/vx0net/daemon/main/install-vx0.sh | bash
```

### 🐳 Docker
For developers and advanced users:
```bash
git clone https://github.com/vx0net/daemon.git
cd daemon
docker-compose up -d
```

### 🌍 Web Installer
Visit: [vx0net.github.io/daemon/web-installer](https://vx0net.github.io/daemon/web-installer/)

### 🖥️ Desktop GUI
Download the desktop installer from [Releases](https://github.com/vx0net/daemon/releases)

## 🛠️ Technical Details

### Core Features
- **BGP Routing** - Dynamic path selection and route management
- **IKE/IPSec Security** - Encrypted tunnel establishment
- **DNS Resolution** - Internal `.vx0` domain handling
- **Service Discovery** - Automatic service registration
- **Multi-Platform** - Linux, macOS, Windows (WSL2)

### Network Protocols
| Protocol | Port | Purpose |
|----------|------|---------|
| BGP | 1179/tcp | Dynamic routing |
| IKE | 4500/udp | Security negotiation |
| DNS | 5353/tcp | Name resolution |
| Discovery | 8080/tcp | Peer discovery |
| Metrics | 9090/tcp | Monitoring |

### System Requirements
- **CPU** - 1 core minimum, 2+ cores recommended
- **RAM** - 512MB minimum, 1GB+ recommended  
- **Storage** - 1GB available space
- **Network** - Internet connection with ports 1179, 4500, 5353, 8080, 9090

## 🌍 Global Deployment

### For Organizations
Deploy backbone and regional nodes across multiple locations:
- **Kubernetes** - Orchestrated deployment across regions
- **VPS Automation** - Automated setup on cloud providers
- **Monitoring** - Prometheus and Grafana integration

See [GLOBAL-DEPLOYMENT.md](GLOBAL-DEPLOYMENT.md) for details.

### For Developers
Docker-based development environment:
```bash
# Clone repository
git clone https://github.com/vx0net/daemon.git
cd daemon

# Start development environment
docker-compose up -d

# View logs
docker-compose logs -f
```

See [DOCKER.md](DOCKER.md) for details.

## 📊 Monitoring & Management

### Web Dashboard
Access your node dashboard at `http://localhost:8090`:
- Real-time network status
- Peer connections
- Traffic statistics
- Configuration management

### Command Line
```bash
# Check status
vx0net status

# View peers
vx0net peers list

# Monitor traffic
vx0net metrics

# Update node
vx0net update
```

### Logs
```bash
# View live logs
tail -f ~/vx0-network/logs/vx0net.log

# Auto-update logs
tail -f ~/vx0-network/logs/auto-update.log
```

## 🔧 Configuration

### Automatic Configuration
The installer automatically:
- Detects your network environment
- Generates secure certificates
- Configures firewall rules
- Selects optimal ASN
- Connects to bootstrap nodes

### Manual Configuration
Edit `~/vx0-network/config/vx0net.toml`:
```toml
[node]
name = "my-edge-node"
tier = "Edge"
asn = 66001

[network]
listen_addr = "0.0.0.0:1179"
bootstrap_nodes = ["bootstrap1.vx0.network:1179"]

[security]
cert_path = "/etc/vx0net/certs/node.crt"
key_path = "/etc/vx0net/certs/node.key"
```

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- How to set up development environment
- Code style guidelines
- Pull request process
- Community guidelines

### Development Setup
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/vx0net/daemon.git
cd daemon
cargo build

# Run tests
cargo test

# Start development node
cargo run -- start --config config/edge-node.toml
```

## 📚 Documentation

- **[SIMPLE-SETUP.md](SIMPLE-SETUP.md)** - Beginner-friendly guide
- **[DOCKER.md](DOCKER.md)** - Docker deployment
- **[GLOBAL-DEPLOYMENT.md](GLOBAL-DEPLOYMENT.md)** - Multi-location deployment
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Developer guide
- **[CHANGELOG.md](CHANGELOG.md)** - Version history

## 🆘 Support

### Community
- **Discord** - [discord.gg/vx0network](https://discord.gg/vx0network)
- **GitHub Discussions** - [Ask questions and share ideas](https://github.com/vx0net/daemon/discussions)
- **Issues** - [Report bugs and request features](https://github.com/vx0net/daemon/issues)

### Troubleshooting
```bash
# Check node health
vx0net health

# Test connectivity
vx0net test

# Reset configuration
vx0net reset

# Get help
vx0net --help
```

Common solutions:
- **Can't connect to peers** - Check firewall settings
- **Dashboard not loading** - Verify port 8090 is accessible
- **Updates failing** - Run `vx0net update --force`

## 🔒 Security

VX0 Network prioritizes security:
- **End-to-end encryption** for all communications
- **Automatic certificate rotation** 
- **Regular security audits**
- **Zero-trust architecture**

Report security issues: security@vx0.network

## 📄 License

This project is licensed under the [MIT License](LICENSE).

## 🌟 Why VX0 Network?

> *"The internet was designed to be decentralized, but it became centralized. VX0 Network brings it back to its roots."*

- **Freedom** - No censorship or surveillance
- **Privacy** - Your data stays yours
- **Resilience** - No single point of failure
- **Community** - Built by users, for users
- **Future-Proof** - Designed for the next generation of internet

Join thousands of users building the decentralized internet. Start with one command:

```bash
curl -fsSL https://raw.githubusercontent.com/vx0net/daemon/main/install-vx0.sh | bash
```

---

**🌐 Welcome to the free internet. Welcome to VX0.**