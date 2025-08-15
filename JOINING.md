# 🌐 Joining the VX0 Network

**Anyone can join the VX0 network!** The network is designed to be completely open and permissionless. Here's everything you need to know.

## 🚀 **Quick Start - Join in 5 Minutes**

### **Option 1: Easy Join Script (Recommended)**

```bash
# Clone the VX0 daemon
git clone <this-repo> && cd vx0net-daemon

# Build the daemon  
cargo build --release

# Run the easy join script
./scripts/join-network.sh
```

The script will:
- ✅ Auto-configure your node based on your preferences
- ✅ Detect your external IP automatically  
- ✅ Generate SSL certificates
- ✅ Connect to the network automatically
- ✅ Start serving the network immediately

### **Option 2: Manual Configuration**

1. **Choose Your Node Type**:
   - **Edge Node** (easiest): Personal/home nodes, connects to regional nodes
   - **Regional Node** (intermediate): Serves local areas, connects edge to backbone  
   - **Backbone Node** (advanced): Core infrastructure, high availability required

2. **Get Your ASN Range**:
   - Edge: `66000-69999` (4,000 available ASNs)
   - Regional: `65100-65999` (900 available ASNs)
   - Backbone: `65000-65099` (100 available ASNs)

3. **Create Configuration** (see templates in `deploy/` folder)

4. **Start Your Node**:
   ```bash
   ./target/release/vx0net start --join-network
   ```

## 🔗 **Current Network Entry Points**

### **Public Bootstrap Nodes** (Updated: 2025-01-15)

These nodes accept new connections and help bootstrap new nodes:

#### **Backbone Nodes (Tier 1)**
```
backbone1.vx0.network - ASN 65001 - Location: US-East
backbone2.vx0.network - ASN 65002 - Location: EU-West  
backbone3.vx0.network - ASN 65003 - Location: Asia-Pacific
```

#### **Regional Nodes (Tier 2)**  
```
regional1.vx0.network - ASN 65101 - Location: US-East
regional2.vx0.network - ASN 65102 - Location: US-West
regional3.vx0.network - ASN 65103 - Location: EU-West
regional4.vx0.network - ASN 65104 - Location: EU-East
regional5.vx0.network - ASN 65105 - Location: Asia-Pacific
```

### **How to Add Your Node to the Bootstrap List**

Once your node is stable and has been running for 30+ days:

1. Open an issue or PR on this repository
2. Provide your node details:
   ```
   Hostname: your-node.vx0.network
   IP: your.public.ip.address
   ASN: your-asn
   Tier: your-tier
   Location: your-location
   Uptime: 99.5%
   Contact: your-contact-info (optional)
   ```
3. Community members will verify and add your node

## 📖 **How Network Joining Works**

### **1. Automatic ASN Assignment**
- Your node automatically finds an unused ASN in your tier's range
- No central authority needed - fully decentralized
- ASN conflicts are resolved automatically

### **2. Peer Discovery**
The network uses multiple discovery methods:
- **Bootstrap Nodes**: Connect to known public entry points
- **Local Discovery**: Find nearby nodes on the same network
- **DNS Discovery**: Resolve vx0.network domains
- **Peer Exchange**: Learn about new nodes from existing peers

### **3. Tier-Based Connections**  
- **Edge nodes** connect to Regional nodes only
- **Regional nodes** connect to Backbone and Edge nodes
- **Backbone nodes** connect to other Backbone and Regional nodes
- This creates a scalable hierarchical structure

### **4. Secure Tunnels**
- All connections use IKE/IPSec encryption
- Each peer relationship has its own secure tunnel
- Traffic is encrypted end-to-end

### **5. Network Isolation**
- Your node will only resolve `.vx0` domains
- No traffic leaks to the regular internet
- Complete isolation from surveillance and censorship

## 🔧 **Requirements**

### **Minimum Hardware**
- **Edge Node**: 1 CPU, 512MB RAM, 10GB disk
- **Regional Node**: 1 CPU, 1GB RAM, 20GB disk  
- **Backbone Node**: 2 CPU, 2GB RAM, 40GB disk

### **Network Requirements**
- **Public IP address** (static preferred)
- **Open ports**: 1179 (BGP), 4500 (IKE), 8080 (Discovery), 9090+ (Monitoring)
- **Bandwidth**: Minimum 1 Mbps, more for backbone nodes

### **Operating System**
- Linux (Ubuntu 22.04+ recommended)
- macOS (development/testing)
- Windows (via WSL2)

## 🌟 **Network Behavior After Joining**

### **What Your Node Does**
- ✅ **Routes traffic** for the VX0 network
- ✅ **Provides redundancy** and resilience  
- ✅ **Resolves .vx0 domains** for local clients
- ✅ **Discovers and shares** network services
- ✅ **Maintains secure tunnels** with peers
- ✅ **Blocks external internet** access (isolation mode)

### **What You Get**
- 🔒 **Complete privacy** - no surveillance possible
- 🚫 **Censorship resistance** - no single point of failure
- 🌐 **Access to VX0 services** - decentralized apps and services
- 📡 **Peer-to-peer communication** - direct encrypted messaging
- 🛡️ **Network security** - all traffic encrypted
- 🤝 **Community membership** - part of a free internet

## 🆘 **Troubleshooting**

### **Common Issues**

**"No entry points discovered"**
```bash
# Check internet connectivity
ping 8.8.8.8

# Check if ports are blocked
telnet backbone1.vx0.network 1179

# Try manual bootstrap
./target/release/vx0net connect --peer backbone1.vx0.network:1179
```

**"ASN assignment failed"**
```bash
# Manually assign an ASN
./target/release/vx0net config set-asn 66123  # Use any available ASN

# Check ASN conflicts
./target/release/vx0net network scan-asns
```

**"Tunnel establishment failed"**
```bash
# Check firewall
sudo ufw status
sudo ufw allow 1179/tcp
sudo ufw allow 4500/udp

# Regenerate certificates  
rm -rf certs/ && deploy/generate_certs.sh $(hostname)
```

**"No peers connecting"**
```bash
# Check if your IP is reachable
./target/release/vx0net network test-connectivity

# Verify configuration
./target/release/vx0net config validate

# Check logs
tail -f logs/vx0net.log
```

### **Getting Help**

- 📖 **Documentation**: See README.md and DEPLOYMENT.md
- 🐛 **Issues**: Open an issue on this repository  
- 💬 **Community**: Join the VX0 network and find us on vx0.network domains
- 📧 **Contact**: Post in discussions or issues

## 🌍 **Growing the Network**

### **Help Others Join**
- Share your node details so others can bootstrap from you
- Run a stable node that others can rely on
- Document your setup and share guides
- Contribute to the codebase

### **Network Health**
The VX0 network gets stronger with each new node:
- **More nodes** = more redundancy and resilience
- **Geographic distribution** = better global coverage
- **Diverse operators** = resistance to coordinated attacks
- **Community growth** = more services and applications

### **Recommended Network Growth**
- **Phase 1**: 10-50 nodes (current)
- **Phase 2**: 100-500 nodes (short term)  
- **Phase 3**: 1,000-10,000 nodes (medium term)
- **Phase 4**: 100,000+ nodes (long term vision)

## 🎯 **What Makes VX0 Different**

### **Truly Open**
- ✅ No registration required
- ✅ No approval process
- ✅ No central authority
- ✅ No fees or payments
- ✅ No identity verification

### **Censorship Resistant**  
- ✅ No single point of failure
- ✅ Distributed routing
- ✅ Encrypted communication
- ✅ No DNS censorship possible
- ✅ Traffic analysis resistant

### **Scalable Architecture**
- ✅ Three-tier hierarchy handles growth
- ✅ BGP routing provides efficiency
- ✅ Local services reduce latency
- ✅ Peer limits prevent overload

### **Community Controlled**
- ✅ Open source code
- ✅ Community governance
- ✅ Transparent operations
- ✅ Collaborative development

---

## 🚀 **Ready to Join?**

```bash
# One command to join the revolution
./scripts/join-network.sh
```

**Welcome to the free internet! 🌐**

The VX0 network is powered by people like you who believe in digital freedom, privacy, and censorship resistance. Every node makes the network stronger and the internet more free.

**Join us. Build the future. Stay free.** ✊