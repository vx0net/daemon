# 🌍 VX0 Network Global Deployment Guide

Deploy VX0 backbone nodes across multiple VPS locations with automatic discovery and Kubernetes orchestration.

## 🎯 Overview

This guide covers deploying a globally distributed VX0 network with:
- **Backbone nodes** in major geographic locations
- **Automatic peer discovery** between locations
- **Kubernetes orchestration** for scalability
- **VPS deployment** for geographic distribution
- **Load balancing** and high availability

## 🏗️ Architecture

```
VX0 Global Network Architecture
                                    
    ┌─────────────────────────────────────────────────────────┐
    │                 Global Discovery                        │
    │           registry.vx0.network                          │
    └─────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
   ┌────▼────┐          ┌────▼────┐          ┌────▼────┐
   │US-East  │◄────────►│EU-West  │◄────────►│Asia-Pac │
   │Backbone │          │Backbone │          │Backbone │
   │ASN 65001│          │ASN 65003│          │ASN 65005│
   └─────────┘          └─────────┘          └─────────┘
        │                     │                     │
   ┌────▼────┐          ┌────▼────┐          ┌────▼────┐
   │Regional │          │Regional │          │Regional │
   │Nodes    │          │Nodes    │          │Nodes    │
   └─────────┘          └─────────┘          └─────────┘
        │                     │                     │
   ┌────▼────┐          ┌────▼────┐          ┌────▼────┐
   │Edge     │          │Edge     │          │Edge     │
   │Nodes    │          │Nodes    │          │Nodes    │
   └─────────┘          └─────────┘          └─────────┘
```

## 🚀 Quick Start

### **Option 1: Kubernetes Multi-Location Deployment**

```bash
# Deploy backbone nodes globally
./k8s/deploy-global.sh deploy us-east eu-west asia-pacific

# Check status
./k8s/deploy-global.sh status

# Add more locations
./k8s/deploy-global.sh add-location us-west 203.0.113.4
```

### **Option 2: VPS Direct Deployment**

```bash
# On each VPS (run as root)
curl -fsSL https://raw.githubusercontent.com/vx0net/daemon/main/vps-deploy/setup-vps.sh | bash

# Or manually:
wget https://github.com/vx0net/daemon/archive/main.zip
unzip main.zip && cd vx0net-daemon-main
./vps-deploy/setup-vps.sh
```

## 🌍 Backbone Node Locations

### **Predefined Locations with Auto-Assignment**

| Location | ASN | Geographic Coverage | Recommended VPS |
|----------|-----|-------------------|-----------------|
| **us-east** | 65001 | US East Coast, Canada | DigitalOcean NYC |
| **us-west** | 65002 | US West Coast, Mexico | AWS us-west-2 |
| **eu-west** | 65003 | UK, Ireland, Western EU | Hetzner Finland |
| **eu-central** | 65004 | Germany, Central EU | Vultr Frankfurt |
| **asia-pacific** | 65005 | Singapore, Australia | Linode Singapore |
| **asia-east** | 65006 | Japan, Korea, China | AWS ap-northeast-1 |

## 🔍 Auto-Discovery Mechanisms

The VX0 network uses multiple auto-discovery methods for backbone nodes:

### **1. DNS-Based Discovery**
```bash
# Backbone nodes register with predictable hostnames
backbone-us-east.vx0.network     -> 203.0.113.1
backbone-eu-west.vx0.network     -> 203.0.113.3  
backbone-asia-pacific.vx0.network -> 203.0.113.5
```

### **2. Bootstrap Registry**
- Central registry at `registry.vx0.network/bootstrap-registry.json`
- Auto-updated when nodes come online
- Fallback to local registry files

### **3. Kubernetes Service Discovery**
- Uses Kubernetes DNS for container-to-container discovery
- Service mesh integration for automatic routing
- Health checks and automatic failover

### **4. BGP Route Announcements**
- Nodes announce themselves via BGP
- Automatic peer relationship establishment
- Route optimization based on AS-path

## 📋 Deployment Methods

### **Method 1: Kubernetes Global Deployment**

#### **Prerequisites**
```bash
# Install required tools
kubectl version --client
docker --version
jq --version
```

#### **Deploy All Locations**
```bash
# Clone repository
git clone https://github.com/vx0net/daemon.git
cd vx0net-daemon

# Build Docker image
docker build -t vx0net-daemon:latest .

# Deploy globally (default: us-east, eu-west, asia-pacific)
./k8s/deploy-global.sh deploy

# Or specify custom locations
./k8s/deploy-global.sh deploy us-east us-west eu-west eu-central asia-pacific asia-east
```

#### **Monitor Deployment**
```bash
# Check deployment status
./k8s/deploy-global.sh status

# View all pods
kubectl get pods -n vx0-network

# Check specific node logs
kubectl logs -n vx0-network -l app=vx0-backbone-us-east

# View services and external IPs
kubectl get services -n vx0-network
```

### **Method 2: VPS Direct Deployment**

#### **Setup Each VPS**
```bash
# SSH to each VPS and run as root:
curl -fsSL https://setup.vx0.network/vps | bash

# Or download and inspect first:
wget https://raw.githubusercontent.com/vx0net/daemon/main/vps-deploy/setup-vps.sh
chmod +x setup-vps.sh
./setup-vps.sh
```

#### **Manual VPS Setup**
```bash
# 1. Update system
apt update && apt upgrade -y

# 2. Install Docker
curl -fsSL https://get.docker.com | sh

# 3. Clone VX0 daemon
git clone https://github.com/vx0net/daemon.git /opt/vx0-network
cd /opt/vx0-network

# 4. Run VPS setup script
./vps-deploy/setup-vps.sh

# 5. Start services
systemctl start vx0net
systemctl enable vx0net
```

## 🔧 Configuration

### **Environment Variables**

Set these on each VPS or in Kubernetes ConfigMaps:

```bash
# Node identification
VX0NET_NODE_ASN=65001
VX0NET_LOCATION="us-east"
VX0NET_PUBLIC_IP="203.0.113.1"
VX0NET_NODE_HOSTNAME="backbone-us-east.vx0.network"

# Auto-discovery
VX0NET_AUTO_DISCOVERY=true
VX0NET_DISCOVERY_REGISTRY="https://registry.vx0.network/bootstrap-registry.json"
VX0NET_DISCOVERY_INTERVAL=300

# Network configuration
VX0NET_BGP_LISTEN_PORT=1179
VX0NET_IKE_LISTEN_PORT=4500
VX0NET_DISCOVERY_PORT=8080
VX0NET_METRICS_PORT=9090
```

### **DNS Configuration**

For automatic discovery, configure these DNS records:

```dns
# A records for backbone nodes
backbone-us-east.vx0.network.     IN A    203.0.113.1
backbone-us-west.vx0.network.     IN A    203.0.113.2
backbone-eu-west.vx0.network.     IN A    203.0.113.3
backbone-eu-central.vx0.network.  IN A    203.0.113.4
backbone-asia-pacific.vx0.network. IN A   203.0.113.5
backbone-asia-east.vx0.network.   IN A    203.0.113.6

# SRV records for service discovery
_vx0-bgp._tcp.vx0.network.        IN SRV  10 5 1179 backbone-us-east.vx0.network.
_vx0-bgp._tcp.vx0.network.        IN SRV  10 5 1179 backbone-eu-west.vx0.network.
_vx0-bgp._tcp.vx0.network.        IN SRV  10 5 1179 backbone-asia-pacific.vx0.network.

# Registry endpoint
registry.vx0.network.             IN A    203.0.113.1
```

## 🚦 Network Topology and Peering

### **Automatic Peering Rules**

Backbone nodes automatically establish peering relationships:

1. **Full Mesh**: All backbone nodes peer with each other
2. **Geographic Optimization**: Prefer closer nodes for better latency
3. **Redundancy**: Maintain at least 3 active peer connections
4. **Load Balancing**: Distribute traffic across available paths

### **ASN Allocation Strategy**

```
Backbone (Core Infrastructure):   65000-65099  (100 ASNs)
├── US Regions:                   65001-65020
├── EU Regions:                   65021-65040  
├── Asia Regions:                 65041-65060
└── Other Regions:                65061-65099

Regional (Community Networks):    65100-65999  (900 ASNs)
├── Per backbone region:          ~150 ASNs each

Edge (Personal/Home):            66000-69999  (4000 ASNs)
├── Auto-assigned by tier
└── Geographic clustering
```

## 📊 Monitoring and Observability

### **Built-in Metrics**

Each node exposes Prometheus metrics:

```bash
# Node health and status
curl http://backbone-us-east.vx0.network:9090/metrics

# BGP session status
vx0_bgp_sessions_total{state="established"} 3
vx0_bgp_routes_received_total 1234

# Network discovery metrics  
vx0_discovery_peers_found_total 5
vx0_discovery_last_update_timestamp 1642291200
```

### **Global Monitoring Stack**

Deploy monitoring across all locations:

```bash
# Deploy Prometheus + Grafana to each region
kubectl apply -f k8s/monitoring/

# Access regional dashboards
US-East:  http://backbone-us-east.vx0.network:3000
EU-West:  http://backbone-eu-west.vx0.network:3000
Asia-Pac: http://backbone-asia-pacific.vx0.network:3000
```

## 🔐 Security

### **Certificate Management**

Automatic certificate generation and distribution:

```bash
# Certificates are auto-generated per location
/app/certs/
├── ca.crt              # Shared CA
├── backbone.crt        # Location-specific certificate
└── backbone.key        # Private key

# Automatic renewal via Let's Encrypt (production)
certbot certonly --standalone -d backbone-us-east.vx0.network
```

### **Network Security**

```bash
# Firewall rules (applied automatically)
ufw allow 1179/tcp      # BGP
ufw allow 4500/udp      # IKE/IPSec
ufw allow 8080/tcp      # Discovery
ufw allow 9090/tcp      # Metrics (internal)

# IPSec tunnels between backbone nodes
# Automatic key exchange via IKE v2
# AES-256-GCM encryption
```

## 🚨 Troubleshooting

### **Common Issues**

#### **Nodes Can't Find Each Other**
```bash
# Check DNS resolution
nslookup backbone-us-east.vx0.network

# Check connectivity
telnet backbone-us-east.vx0.network 1179

# Check discovery registry
curl https://registry.vx0.network/bootstrap-registry.json

# Check local discovery
kubectl logs -n vx0-network -l app=vx0-discovery
```

#### **BGP Sessions Not Establishing**
```bash
# Check BGP configuration
kubectl exec -n vx0-network PODNAME -- vx0net info

# Check peer status
kubectl exec -n vx0-network PODNAME -- vx0net peers

# Check firewall
kubectl exec -n vx0-network PODNAME -- netstat -tlnp | grep 1179
```

#### **Auto-Discovery Not Working**
```bash
# Check discovery logs
kubectl logs -n vx0-network -l app=vx0-backbone -c vx0-backbone | grep discovery

# Verify registry accessibility
curl -v http://BACKBONE_IP:8080/bootstrap-registry.json

# Manual peer addition
kubectl exec -n vx0-network PODNAME -- vx0net connect PEER_IP PEER_ASN
```

## 📈 Scaling

### **Adding New Locations**

```bash
# Kubernetes deployment
./k8s/deploy-global.sh add-location LOCATION PUBLIC_IP

# VPS deployment
# SSH to new VPS and run setup script
./vps-deploy/setup-vps.sh

# The new node will automatically:
# 1. Detect its location via IP geolocation
# 2. Get assigned an appropriate ASN
# 3. Discover and connect to existing backbone nodes
# 4. Register itself in the global registry
```

### **Regional and Edge Nodes**

```bash
# Deploy regional nodes (connect to nearest backbone)
docker-compose --profile regional up -d

# Deploy edge nodes (for end users)
docker-compose up -d vx0-edge

# Automatic tier-based peering
# Edge -> Regional -> Backbone
```

## 🎉 Success Verification

After deployment, verify the global network:

```bash
# Check all backbone nodes are running
./k8s/deploy-global.sh status

# Verify inter-node connectivity
for node in us-east eu-west asia-pacific; do
  kubectl exec -n vx0-network -l app=vx0-backbone-$node -- vx0net network-status
done

# Check global registry
curl https://registry.vx0.network/bootstrap-registry.json | jq '.vx0_network_bootstrap_registry.backbone_nodes'

# Test end-to-end connectivity
kubectl exec -n vx0-network PODNAME -- ping backbone-eu-west.vx0.network
```

## 📚 Additional Resources

- [README.md](README.md) - Project overview
- [DOCKER.md](DOCKER.md) - Docker deployment guide  
- [JOINING.md](JOINING.md) - How users join the network
- [k8s/](k8s/) - Kubernetes manifests
- [vps-deploy/](vps-deploy/) - VPS deployment scripts

---

**🌍 The VX0 network will automatically discover and connect backbone nodes across all major geographic regions, creating a truly global, censorship-resistant network infrastructure!**
