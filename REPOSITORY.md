# 📁 VX0 Network Repository Structure

This document explains the organization of the VX0 Network Daemon repository.

## 🗂️ Directory Structure

```
vx0net-daemon/
├── 📄 README.md                    # Main project documentation
├── 📄 LICENSE                      # MIT License
├── 📄 CONTRIBUTING.md               # Contribution guidelines
├── 📄 CHANGELOG.md                  # Version history
├── 📄 Cargo.toml                    # Rust project configuration
├── 📄 Dockerfile                    # Docker image build
├── 📄 docker-compose.yml            # Local Docker orchestration
├── 📄 .gitignore                    # Git ignore rules
│
├── 📂 src/                          # Rust source code
│   ├── 📄 main.rs                   # CLI application entry point
│   ├── 📄 lib.rs                    # Library exports
│   ├── 📂 config/                   # Configuration management
│   ├── 📂 node/                     # Node management & peering
│   ├── 📂 network/                  # Network protocols (BGP, IKE, DNS)
│   ├── 📂 services/                 # Service discovery & monitoring
│   └── 📂 bin/                      # Additional binaries
│
├── 📂 config/                       # Configuration templates
│   ├── 📄 edge-node.toml            # Edge node configuration
│   ├── 📄 regional-node.toml        # Regional node configuration
│   ├── 📄 backbone-node.toml        # Backbone node configuration
│   └── 📄 vx0net-node1.toml         # Example configurations
│
├── 📂 scripts/                      # Management scripts
│   ├── 📄 docker-deploy.sh          # Docker deployment automation
│   └── 📄 join-network.sh           # Network joining script
│
├── 📂 k8s/                          # Kubernetes deployment
│   ├── 📄 deploy-global.sh          # Global K8s deployment script
│   ├── 📂 backbone/                 # Backbone node manifests
│   ├── 📂 regional/                 # Regional node manifests
│   ├── 📂 edge/                     # Edge node manifests
│   ├── 📂 discovery/                # Discovery service manifests
│   └── 📂 monitoring/               # Monitoring stack
│
├── 📂 vps-deploy/                   # VPS deployment automation
│   └── 📄 setup-vps.sh              # Automated VPS setup script
│
├── 📂 web-installer/                # Web-based installer
│   └── 📄 index.html                # Interactive web installer
│
├── 📂 desktop-installer/            # Desktop GUI installer
│   └── 📄 vx0-desktop-installer.py  # Cross-platform GUI installer
│
├── 📂 deploy/                       # Deployment templates and scripts
│   ├── 📄 install.sh                # Traditional installation script
│   ├── 📄 generate_certs.sh         # Certificate generation
│   └── 📄 test_network.sh           # Network testing
│
├── 📂 monitoring/                   # Monitoring configuration
│   ├── 📄 prometheus.yml            # Prometheus configuration
│   └── 📂 grafana/                  # Grafana dashboards
│
├── 📂 certs/                        # Certificate templates
│   └── 📄 .gitkeep                  # Keep directory in git
│
├── 📂 .github/                      # GitHub-specific files
│   ├── 📂 workflows/                # GitHub Actions
│   │   └── 📄 ci.yml                # CI/CD pipeline
│   └── 📂 ISSUE_TEMPLATE/           # Issue templates
│       └── 📄 bug_report.md         # Bug report template
│
├── 📄 install-vx0.sh               # 🌟 MAIN INSTALLER SCRIPT
├── 📄 auto-update.sh               # Automatic update system
├── 📄 deploy-vx0-global.sh         # Global deployment orchestrator
├── 📄 SIMPLE-SETUP.md              # Beginner's guide
├── 📄 DOCKER.md                    # Docker deployment guide
├── 📄 GLOBAL-DEPLOYMENT.md         # Multi-location deployment guide
└── 📄 bootstrap-registry.json      # Network bootstrap registry
```

## 🎯 Key Files for GitHub Upload

### **Essential Files** (Must upload)
1. **`install-vx0.sh`** - The main one-command installer
2. **`auto-update.sh`** - Self-healing and update system
3. **`src/`** - Complete Rust source code
4. **`Cargo.toml`** - Rust project configuration
5. **`Dockerfile`** - Docker image build instructions
6. **`docker-compose.yml`** - Container orchestration
7. **`README.md`** - Project overview and quick start
8. **`LICENSE`** - MIT license file

### **Installation & Deployment**
- **`SIMPLE-SETUP.md`** - Beginner-friendly guide
- **`scripts/docker-deploy.sh`** - Docker deployment automation
- **`k8s/`** - Complete Kubernetes deployment system
- **`vps-deploy/setup-vps.sh`** - VPS automation
- **`web-installer/`** - Web-based installer interface
- **`desktop-installer/`** - GUI installer for desktop

### **Configuration & Templates**
- **`config/`** - Node configuration templates
- **`deploy/`** - Deployment scripts and templates
- **`monitoring/`** - Monitoring configurations
- **`bootstrap-registry.json`** - Network discovery registry

### **Documentation**
- **`DOCKER.md`** - Docker deployment guide
- **`GLOBAL-DEPLOYMENT.md`** - Multi-location deployment
- **`CONTRIBUTING.md`** - Contribution guidelines
- **`CHANGELOG.md`** - Version history

### **GitHub Integration**
- **`.github/workflows/ci.yml`** - Automated CI/CD
- **`.github/ISSUE_TEMPLATE/`** - Issue templates
- **`.gitignore`** - Git ignore rules

## 🌐 URLs After GitHub Upload

Once uploaded to GitHub (e.g., `github.com/vx0net/daemon`), these URLs will work:

### **One-Command Installation**
```bash
curl -fsSL https://raw.githubusercontent.com/vx0net/daemon/main/install-vx0.sh | bash
```

### **Manual Download**
```bash
# Download installer
wget https://github.com/vx0net/daemon/raw/main/install-vx0.sh

# Download auto-update
wget https://github.com/vx0net/daemon/raw/main/auto-update.sh

# Clone entire repository
git clone https://github.com/vx0net/daemon.git
```

### **Web Installer**
- `https://vx0net.github.io/daemon/web-installer/` (via GitHub Pages)

### **Container Images**
- `ghcr.io/vx0net/daemon:latest` (via GitHub Container Registry)

## 📦 Release Assets

GitHub Releases will include:
- **Compiled binaries** for multiple platforms
- **Installation scripts** (`install-vx0.sh`, `auto-update.sh`)
- **Docker images** (automatically built and published)
- **Configuration templates**
- **Documentation PDFs**

## 🚀 Setup Instructions for Repository

1. **Create GitHub repository**: `vx0net/daemon`
2. **Upload all files** from this directory
3. **Configure GitHub Pages** to serve `web-installer/`
4. **Set up GitHub Actions** for automated CI/CD
5. **Configure container registry** for Docker images
6. **Create initial release** with v1.0.0 tag

## 🔗 Integration Points

### **Installation URLs**
The installer scripts reference these GitHub URLs:
- Raw file downloads from `raw.githubusercontent.com`
- Release downloads from `github.com/vx0net/daemon/releases`
- Container images from `ghcr.io/vx0net/daemon`

### **Auto-Update System**
- Checks GitHub releases for new versions
- Downloads updates from GitHub releases
- Falls back to raw file downloads

### **Discovery Registry**
- Bootstrap registry hosted on GitHub
- Auto-discovery points to GitHub-hosted registry
- Fallback to CDN-hosted registries

This structure provides a complete, production-ready repository that enables the ultra-simple one-command installation experience! 🌟
