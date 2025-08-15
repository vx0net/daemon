# VX0 Network - Production Deployment Guide

## 🚀 **Deploy Your Own Censorship-Resistant Network**

This guide will help you deploy the VX0 Network across multiple VPS instances to create a real, working, isolated network.

## 📋 **Prerequisites**

- **3+ VPS instances** (DigitalOcean, Linode, AWS, Vultr, etc.)
- **Ubuntu 22.04** or similar Linux distribution
- **Root/sudo access** on each VPS
- **Basic networking knowledge**

## 🏗️ **Network Architecture**

```
🌐 INTERNET (for initial setup only)
│
├── VPS 1: Backbone Node (ASN 65001)
│   └── Public IP: backbone1.vx0.network
│
├── VPS 2: Regional Node (ASN 65101) 
│   └── Public IP: regional1.vx0.network
│
└── VPS 3: Edge Node (ASN 66001)
    └── Public IP: edge1.vx0.network
```

## 📝 **Step 1: Prepare Your VPS Instances**

### Get 3 VPS instances with these minimum specs:
- **Backbone**: 2 CPU, 4GB RAM, 80GB disk
- **Regional**: 1 CPU, 2GB RAM, 40GB disk  
- **Edge**: 1 CPU, 1GB RAM, 20GB disk

### Note down the public IP addresses:
```bash
BACKBONE1_IP="YOUR_BACKBONE_VPS_IP"
REGIONAL1_IP="YOUR_REGIONAL_VPS_IP" 
EDGE1_IP="YOUR_EDGE_VPS_IP"
```

## 🔧 **Step 2: Install VX0 on Each VPS**

### On each VPS, run:

```bash
# Download and run the installation script
curl -sSL https://raw.githubusercontent.com/your-repo/vx0net-daemon/main/deploy/install.sh | bash

# Or manually copy and run the script
scp deploy/install.sh user@vps-ip:~/
ssh user@vps-ip
bash install.sh
```

## 📁 **Step 3: Copy Code to VPS**

From your development machine:

```bash
# Copy the entire vx0net-daemon directory to each VPS
scp -r ./vx0net-daemon user@$BACKBONE1_IP:~/vx0-network/
scp -r ./vx0net-daemon user@$REGIONAL1_IP:~/vx0-network/
scp -r ./vx0net-daemon user@$EDGE1_IP:~/vx0-network/
```

## 🔐 **Step 4: Generate Certificates**

### On the backbone node (VPS 1):
```bash
ssh user@$BACKBONE1_IP
cd ~/vx0-network/certs
../vx0net-daemon/deploy/generate_certs.sh backbone1.vx0.network
```

### Copy the CA certificate to other nodes:
```bash
# From backbone node, copy CA cert to other nodes
scp ~/vx0-network/certs/ca.crt user@$REGIONAL1_IP:~/vx0-network/certs/
scp ~/vx0-network/certs/ca.crt user@$EDGE1_IP:~/vx0-network/certs/
```

### On regional node (VPS 2):
```bash
ssh user@$REGIONAL1_IP
cd ~/vx0-network/certs
../vx0net-daemon/deploy/generate_certs.sh regional1.vx0.network
```

### On edge node (VPS 3):
```bash
ssh user@$EDGE1_IP
cd ~/vx0-network/certs
../vx0net-daemon/deploy/generate_certs.sh edge1.vx0.network
```

## ⚙️ **Step 5: Configure Each Node**

### Backbone Node Configuration:

```bash
ssh user@$BACKBONE1_IP
cd ~/vx0-network/config

# Edit the configuration
cat > vx0net.toml << EOF
[node]
hostname = "backbone1.vx0.network"
asn = 65001
tier = "Backbone"
location = "US-East-1"
ipv4_address = "$BACKBONE1_IP"
ipv6_address = "fe80::1"

[network.bgp]
router_id = "$BACKBONE1_IP"
listen_port = 1179
hold_time = 90
keepalive_time = 30

[network.dns]
listen_port = 5353
vx0_dns_servers = ["10.0.0.2:53", "10.0.0.3:53"]
cache_size = 1000

[security.ike]
listen_port = 4500
dh_group = 14
encryption_algorithm = "AES-256"

[services]
enable_discovery = true
discovery_port = 8080

[monitoring]
enable_metrics = true
metrics_port = 9090
log_level = "info"

[[bootstrap.nodes]]
hostname = "regional1.vx0.network"
ip = "$REGIONAL1_IP"
asn = 65101

[security.psk]
default = "your-secure-pre-shared-key-change-this"
EOF
```

### Regional Node Configuration:

```bash
ssh user@$REGIONAL1_IP
cd ~/vx0-network/config

cat > vx0net.toml << EOF
[node]
hostname = "regional1.vx0.network"
asn = 65101
tier = "Regional"
location = "US-West-1"
ipv4_address = "$REGIONAL1_IP"
ipv6_address = "fe80::2"

[network.bgp]
router_id = "$REGIONAL1_IP"
listen_port = 1179
hold_time = 90
keepalive_time = 30

[network.dns]
listen_port = 5353
vx0_dns_servers = ["10.0.0.2:53", "10.0.0.3:53"]
cache_size = 1000

[security.ike]
listen_port = 4500
dh_group = 14
encryption_algorithm = "AES-256"

[services]
enable_discovery = true
discovery_port = 8081

[monitoring]
enable_metrics = true
metrics_port = 9091
log_level = "info"

[[bootstrap.nodes]]
hostname = "backbone1.vx0.network"
ip = "$BACKBONE1_IP"
asn = 65001

[security.psk]
default = "your-secure-pre-shared-key-change-this"
EOF
```

### Edge Node Configuration:

```bash
ssh user@$EDGE1_IP
cd ~/vx0-network/config

cat > vx0net.toml << EOF
[node]
hostname = "edge1.vx0.network"
asn = 66001
tier = "Edge"
location = "Home-Lab"
ipv4_address = "$EDGE1_IP"
ipv6_address = "fe80::3"

[network.bgp]
router_id = "$EDGE1_IP"
listen_port = 1179
hold_time = 90
keepalive_time = 30

[network.dns]
listen_port = 5353
vx0_dns_servers = ["10.0.0.2:53", "10.0.0.3:53"]
cache_size = 1000

[security.ike]
listen_port = 4500
dh_group = 14
encryption_algorithm = "AES-256"

[services]
enable_discovery = true
discovery_port = 8082

[monitoring]
enable_metrics = true
metrics_port = 9092
log_level = "info"

[[bootstrap.nodes]]
hostname = "regional1.vx0.network"
ip = "$REGIONAL1_IP"
asn = 65101

[security.psk]
default = "your-secure-pre-shared-key-change-this"
EOF
```

## 🔨 **Step 6: Build and Deploy**

### On each VPS:

```bash
# Build the daemon
cd ~/vx0-network/vx0net-daemon
cargo build --release

# Enable and start the service
sudo systemctl enable vx0net
sudo systemctl start vx0net

# Check status
sudo systemctl status vx0net
```

## 🔍 **Step 7: Verify Network Operation**

### Check logs on each node:
```bash
# View real-time logs
tail -f ~/vx0-network/logs/vx0net.log

# Check for successful connections
grep "BGP session established" ~/vx0-network/logs/vx0net.log
grep "Added.*peer" ~/vx0-network/logs/vx0net.log
```

### Test connectivity between nodes:
```bash
# On backbone node
curl http://$REGIONAL1_IP:9091/metrics  # Should connect to regional metrics
curl http://$EDGE1_IP:9092/metrics      # Should connect to edge metrics

# Check BGP connections
ss -tulpn | grep 1179
```

### Verify the hierarchy is working:
```bash
# Check peer counts (should respect tier limits)
# Backbone: up to 50 peers
# Regional: up to 20 peers  
# Edge: up to 5 peers

# Look for tier enforcement in logs
grep "tier" ~/vx0-network/logs/vx0net.log
```

## 🧪 **Step 8: Test the Network**

### Test service registration:
```bash
# On edge node, register a service
cd ~/vx0-network/vx0net-daemon
./target/release/vx0net register-service chat chat.community1.vx0 6667
```

### Test DNS resolution (VX0 domains only):
```bash
# Should resolve VX0 domains
nslookup vx0.network $EDGE1_IP:5353
nslookup chat.community1.vx0 $EDGE1_IP:5353

# Should NOT resolve internet domains (network isolation)
nslookup google.com $EDGE1_IP:5353  # Should fail/timeout
```

### Monitor network metrics:
```bash
# Check Prometheus metrics on each node
curl http://$BACKBONE1_IP:9090/metrics
curl http://$REGIONAL1_IP:9091/metrics
curl http://$EDGE1_IP:9092/metrics
```

## 🎉 **Success Indicators**

You know it's working when you see:

✅ **BGP sessions established** between appropriate tiers
✅ **Route advertisements** propagating through the hierarchy
✅ **Service registration** working on edge nodes
✅ **DNS resolution** for .vx0 domains only
✅ **Internet isolation** - external domains blocked
✅ **Tier enforcement** - edge nodes can't connect to each other
✅ **Peer limits** respected by each tier

## 🔧 **Troubleshooting**

### Common issues:

1. **Firewall blocking connections**:
   ```bash
   sudo ufw status
   sudo ufw allow 1179/tcp
   sudo ufw allow 4500/udp
   ```

2. **DNS not resolving**:
   ```bash
   # Check if DNS server is running
   ss -tulpn | grep 5353
   ```

3. **BGP connections failing**:
   ```bash
   # Check if BGP port is open
   telnet $BACKBONE1_IP 1179
   ```

4. **Service not starting**:
   ```bash
   sudo journalctl -u vx0net -f
   ```

## 🌟 **Scaling the Network**

To add more nodes:

1. **More Backbone nodes**: ASNs 65002-65099
2. **More Regional nodes**: ASNs 65102-65999
3. **More Edge nodes**: ASNs 66002-69999

Each new node follows the same process but with:
- Different ASN (within tier range)
- Unique hostname
- Different IP address
- Bootstrap configuration pointing to existing nodes

## 🔒 **Security Notes**

- **Change default PSK** in production
- **Use proper certificates** for production
- **Monitor logs** for suspicious activity
- **Keep software updated**
- **Backup configuration and certificates**

## 📊 **Monitoring**

Access monitoring dashboards:
- Backbone: `http://$BACKBONE1_IP:9090/metrics`
- Regional: `http://$REGIONAL1_IP:9091/metrics`
- Edge: `http://$EDGE1_IP:9092/metrics`

**You now have a working, censorship-resistant, isolated VX0 network!** 🎉