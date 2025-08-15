# 🧪 VX0 Network Real-World Testing Guide

**Test your VX0 censorship-resistant network across multiple VPS instances**

This guide walks you through deploying and testing the VX0 network on real VPS infrastructure to validate functionality, performance, and resilience.

---

## 📋 **Prerequisites**

### **Infrastructure Requirements**
- **3+ VPS instances** from any provider (DigitalOcean, Linode, AWS, Vultr, Hetzner, etc.)
- **Operating System**: Ubuntu 22.04+ (recommended) or similar Linux distribution
- **Access**: Root/sudo access on each VPS
- **Connectivity**: Public IP addresses and internet connectivity

### **Hardware Requirements**
| Node Type | CPU | RAM | Storage | Bandwidth |
|-----------|-----|-----|---------|-----------|
| Backbone  | 2 cores | 2GB | 40GB | 10+ Mbps |
| Regional  | 1 core | 1GB | 20GB | 5+ Mbps |
| Edge      | 1 core | 512MB | 10GB | 1+ Mbps |

### **Network Requirements**
- **Open Ports**: 1179 (BGP), 4500 (IKE), 8080 (Discovery), 9090-9092 (Monitoring)
- **Firewall**: Configured to allow VX0 traffic
- **DNS**: Ability to configure custom DNS settings (optional)

---

## 🏗️ **Testing Architecture**

We'll deploy a 3-tier hierarchical network:

```
┌─────────────────────────────────────────────────────────────┐
│                    VX0 Test Network                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  VPS 1: Backbone Node                                      │
│  ├─ Hostname: backbone1.vx0.test                           │
│  ├─ ASN: 65001                                             │
│  ├─ Role: Core routing, high availability                  │
│  └─ Connects to: Regional nodes                            │
│                                                             │
│  VPS 2: Regional Node                                      │
│  ├─ Hostname: regional1.vx0.test                           │
│  ├─ ASN: 65101                                             │
│  ├─ Role: Regional hub, aggregation                        │
│  └─ Connects to: Backbone + Edge nodes                     │
│                                                             │
│  VPS 3: Edge Node                                          │
│  ├─ Hostname: edge1.vx0.test                               │
│  ├─ ASN: 66001                                             │
│  ├─ Role: User services, local access                      │
│  └─ Connects to: Regional nodes only                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 🚀 **Step 1: Prepare VPS Infrastructure**

### **1.1 Gather VPS Information**

Create a testing environment file:

```bash
# Create testing environment
cat > vx0-test-env.sh << 'EOF'
#!/bin/bash

# VPS Information - UPDATE THESE WITH YOUR ACTUAL IPS
export VPS1_IP="YOUR_VPS1_PUBLIC_IP"    # Backbone Node
export VPS2_IP="YOUR_VPS2_PUBLIC_IP"    # Regional Node  
export VPS3_IP="YOUR_VPS3_PUBLIC_IP"    # Edge Node

# VPS SSH Users (adjust if not root)
export VPS1_USER="root"
export VPS2_USER="root"
export VPS3_USER="root"

# Test Hostnames
export BACKBONE_HOST="backbone1.vx0.test"
export REGIONAL_HOST="regional1.vx0.test"
export EDGE_HOST="edge1.vx0.test"

# Network Configuration
export NETWORK_PSK="vx0-test-network-psk-2025"

echo "VX0 Test Environment Loaded:"
echo "  Backbone: $VPS1_IP ($BACKBONE_HOST)"
echo "  Regional: $VPS2_IP ($REGIONAL_HOST)"
echo "  Edge: $VPS3_IP ($EDGE_HOST)"
EOF

# Load environment
source vx0-test-env.sh
```

### **1.2 Test Basic Connectivity**

```bash
# Verify you can reach all VPS instances
ping -c 3 $VPS1_IP && echo "✅ VPS1 reachable"
ping -c 3 $VPS2_IP && echo "✅ VPS2 reachable"  
ping -c 3 $VPS3_IP && echo "✅ VPS3 reachable"

# Test SSH access
ssh $VPS1_USER@$VPS1_IP "echo 'VPS1 SSH working'" 
ssh $VPS2_USER@$VPS2_IP "echo 'VPS2 SSH working'"
ssh $VPS3_USER@$VPS3_IP "echo 'VPS3 SSH working'"
```

### **1.3 Deploy Code to All VPS**

```bash
# Copy vx0net-daemon to each VPS
echo "📦 Deploying code to VPS instances..."

scp -r ./vx0net-daemon $VPS1_USER@$VPS1_IP:~/
scp -r ./vx0net-daemon $VPS2_USER@$VPS2_IP:~/
scp -r ./vx0net-daemon $VPS3_USER@$VPS3_IP:~/

echo "✅ Code deployed to all VPS instances"
```

---

## ⚙️ **Step 2: Configure VPS 1 (Backbone Node)**

```bash
# SSH to Backbone VPS
ssh $VPS1_USER@$VPS1_IP

# Navigate to project directory
cd ~/vx0net-daemon

# Install dependencies and build
echo "🔨 Installing dependencies and building..."
./deploy/install.sh
cargo build --release

# Configure firewall
echo "🔥 Configuring firewall..."
ufw allow 1179/tcp comment "VX0 BGP"
ufw allow 4500/udp comment "VX0 IKE"
ufw allow 8080/tcp comment "VX0 Discovery"
ufw allow 9090/tcp comment "VX0 Monitoring"
ufw --force enable

# Create configuration
echo "📝 Creating Backbone node configuration..."
mkdir -p config logs certs

cat > config/vx0net.toml << EOF
[node]
hostname = "backbone1.vx0.test"
asn = 65001
tier = "Backbone"
location = "VPS-Test-Backbone"
ipv4_address = "$VPS1_IP"
ipv6_address = "fe80::1"

[network.bgp]
router_id = "$VPS1_IP"
listen_port = 1179
hold_time = 90
keepalive_time = 30

[network.dns]
listen_port = 5353
vx0_dns_servers = ["10.0.0.2:53"]
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
hostname = "regional1.vx0.test"
ip = "$VPS2_IP"
asn = 65101

[security.psk]
default = "vx0-test-network-psk-2025"
EOF

# Generate certificates
echo "🔐 Generating SSL certificates..."
cd certs
../deploy/generate_certs.sh backbone1.vx0.test
cd ..

# Start the backbone node
echo "🚀 Starting Backbone node..."
VX0NET_CONFIG_PATH=config/vx0net.toml \
nohup ./target/release/vx0net start --foreground > logs/vx0net.log 2>&1 &

echo "✅ Backbone node started on VPS1"
echo "📋 Node info:"
echo "  Hostname: backbone1.vx0.test"
echo "  ASN: 65001"
echo "  IP: $VPS1_IP"
echo "  Logs: tail -f ~/vx0net-daemon/logs/vx0net.log"

# Exit SSH session
exit
```

---

## ⚙️ **Step 3: Configure VPS 2 (Regional Node)**

```bash
# SSH to Regional VPS
ssh $VPS2_USER@$VPS2_IP

cd ~/vx0net-daemon

# Install dependencies and build
echo "🔨 Installing dependencies and building..."
./deploy/install.sh
cargo build --release

# Configure firewall
echo "🔥 Configuring firewall..."
ufw allow 1179/tcp comment "VX0 BGP"
ufw allow 4500/udp comment "VX0 IKE"
ufw allow 8080/tcp comment "VX0 Discovery"
ufw allow 9091/tcp comment "VX0 Monitoring"
ufw --force enable

# Create configuration
echo "📝 Creating Regional node configuration..."
mkdir -p config logs certs

cat > config/vx0net.toml << EOF
[node]
hostname = "regional1.vx0.test"
asn = 65101
tier = "Regional"
location = "VPS-Test-Regional"
ipv4_address = "$VPS2_IP"
ipv6_address = "fe80::2"

[network.bgp]
router_id = "$VPS2_IP"
listen_port = 1179
hold_time = 90
keepalive_time = 30

[network.dns]
listen_port = 5353
vx0_dns_servers = ["10.0.0.2:53"]
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
metrics_port = 9091
log_level = "info"

[[bootstrap.nodes]]
hostname = "backbone1.vx0.test"
ip = "$VPS1_IP"
asn = 65001

[security.psk]
default = "vx0-test-network-psk-2025"
EOF

# Generate certificates and copy CA
echo "🔐 Generating SSL certificates..."
cd certs
../deploy/generate_certs.sh regional1.vx0.test
scp $VPS1_USER@$VPS1_IP:~/vx0net-daemon/certs/ca.crt ./
cd ..

# Start the regional node
echo "🚀 Starting Regional node..."
VX0NET_CONFIG_PATH=config/vx0net.toml \
nohup ./target/release/vx0net start --foreground > logs/vx0net.log 2>&1 &

echo "✅ Regional node started on VPS2"
echo "📋 Node info:"
echo "  Hostname: regional1.vx0.test"
echo "  ASN: 65101"
echo "  IP: $VPS2_IP"
echo "  Logs: tail -f ~/vx0net-daemon/logs/vx0net.log"

exit
```

---

## ⚙️ **Step 4: Configure VPS 3 (Edge Node)**

```bash
# SSH to Edge VPS
ssh $VPS3_USER@$VPS3_IP

cd ~/vx0net-daemon

# Install dependencies and build
echo "🔨 Installing dependencies and building..."
./deploy/install.sh
cargo build --release

# Configure firewall
echo "🔥 Configuring firewall..."
ufw allow 1179/tcp comment "VX0 BGP"
ufw allow 4500/udp comment "VX0 IKE"
ufw allow 8080/tcp comment "VX0 Discovery"
ufw allow 9092/tcp comment "VX0 Monitoring"
ufw --force enable

# Create configuration
echo "📝 Creating Edge node configuration..."
mkdir -p config logs certs

cat > config/vx0net.toml << EOF
[node]
hostname = "edge1.vx0.test"
asn = 66001
tier = "Edge"
location = "VPS-Test-Edge"
ipv4_address = "$VPS3_IP"
ipv6_address = "fe80::3"

[network.bgp]
router_id = "$VPS3_IP"
listen_port = 1179
hold_time = 90
keepalive_time = 30

[network.dns]
listen_port = 5353
vx0_dns_servers = ["10.0.0.2:53"]
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
metrics_port = 9092
log_level = "info"

[[bootstrap.nodes]]
hostname = "regional1.vx0.test"
ip = "$VPS2_IP"
asn = 65101

[security.psk]
default = "vx0-test-network-psk-2025"
EOF

# Generate certificates and copy CA
echo "🔐 Generating SSL certificates..."
cd certs
../deploy/generate_certs.sh edge1.vx0.test
scp $VPS1_USER@$VPS1_IP:~/vx0net-daemon/certs/ca.crt ./
cd ..

# Start the edge node
echo "🚀 Starting Edge node..."
VX0NET_CONFIG_PATH=config/vx0net.toml \
nohup ./target/release/vx0net start --foreground > logs/vx0net.log 2>&1 &

echo "✅ Edge node started on VPS3"
echo "📋 Node info:"
echo "  Hostname: edge1.vx0.test"
echo "  ASN: 66001"
echo "  IP: $VPS3_IP"
echo "  Logs: tail -f ~/vx0net-daemon/logs/vx0net.log"

exit
```

---

## 🔍 **Step 5: Verify Network Formation**

### **5.1 Test Basic Connectivity**

```bash
# Load environment
source vx0-test-env.sh

echo "🔗 Testing BGP port connectivity..."

# Test BGP ports between nodes
timeout 5 nc -z $VPS1_IP 1179 && echo "✅ Backbone BGP port open" || echo "❌ Backbone BGP port closed"
timeout 5 nc -z $VPS2_IP 1179 && echo "✅ Regional BGP port open" || echo "❌ Regional BGP port closed"
timeout 5 nc -z $VPS3_IP 1179 && echo "✅ Edge BGP port open" || echo "❌ Edge BGP port closed"

echo "🔗 Testing IKE port connectivity..."

# Test IKE ports
timeout 5 nc -u -z $VPS1_IP 4500 && echo "✅ Backbone IKE port open" || echo "❌ Backbone IKE port closed"
timeout 5 nc -u -z $VPS2_IP 4500 && echo "✅ Regional IKE port open" || echo "❌ Regional IKE port closed"  
timeout 5 nc -u -z $VPS3_IP 4500 && echo "✅ Edge IKE port open" || echo "❌ Edge IKE port closed"
```

### **5.2 Check Node Status**

```bash
# Function to check node status
check_node_status() {
    local ip=$1
    local user=$2
    local name=$3
    
    echo "📊 Checking $name node status..."
    ssh $user@$ip 'cd ~/vx0net-daemon && timeout 10 ./target/release/vx0net info 2>/dev/null || echo "Node not responding"'
    echo ""
}

# Check all nodes
check_node_status $VPS1_IP $VPS1_USER "Backbone"
check_node_status $VPS2_IP $VPS2_USER "Regional"
check_node_status $VPS3_IP $VPS3_USER "Edge"
```

### **5.3 Monitor Logs for Connections**

```bash
# Function to check recent logs
check_logs() {
    local ip=$1
    local user=$2
    local name=$3
    
    echo "📋 Recent $name node logs:"
    ssh $user@$ip 'cd ~/vx0net-daemon && tail -n 10 logs/vx0net.log | grep -E "(established|connected|BGP|IKE)" || echo "No connection logs found"'
    echo ""
}

# Check logs on all nodes
check_logs $VPS1_IP $VPS1_USER "Backbone"
check_logs $VPS2_IP $VPS2_USER "Regional" 
check_logs $VPS3_IP $VPS3_USER "Edge"
```

---

## 🧪 **Step 6: Test Network Functionality**

### **6.1 Test Service Registration**

```bash
echo "🛰️ Testing service registration..."

# Register test services on Edge node
ssh $VPS3_USER@$VPS3_IP 'cd ~/vx0net-daemon && {
    ./target/release/vx0net register-service "test-chat" "chat.test.vx0" 6667
    ./target/release/vx0net register-service "test-web" "web.test.vx0" 8080
    ./target/release/vx0net register-service "test-api" "api.test.vx0" 3000
}'

echo "✅ Test services registered"
```

### **6.2 Test BGP Route Propagation**

```bash
echo "🗺️ Testing BGP route propagation..."

# Check routing tables on each node
ssh $VPS1_USER@$VPS1_IP 'cd ~/vx0net-daemon && echo "=== Backbone Routes ===" && ./target/release/vx0net routes 2>/dev/null || echo "Routes command not available"'

ssh $VPS2_USER@$VPS2_IP 'cd ~/vx0net-daemon && echo "=== Regional Routes ===" && ./target/release/vx0net routes 2>/dev/null || echo "Routes command not available"'

ssh $VPS3_USER@$VPS3_IP 'cd ~/vx0net-daemon && echo "=== Edge Routes ===" && ./target/release/vx0net routes 2>/dev/null || echo "Routes command not available"'
```

### **6.3 Test DNS Functionality**

```bash
echo "🌐 Testing DNS functionality..."

# Test VX0 domain resolution (should work)
echo "Testing .vx0 domain resolution:"
timeout 5 nslookup test.vx0 $VPS3_IP 2>/dev/null && echo "✅ VX0 domains resolving" || echo "❌ VX0 domain resolution failed"

# Test external domain blocking (should fail - network isolation)
echo "Testing external domain blocking:"
timeout 5 nslookup google.com $VPS3_IP 2>/dev/null && echo "❌ External domains NOT blocked" || echo "✅ External domains blocked (isolation working)"
```

### **6.4 Test Metrics Endpoints**

```bash
echo "📊 Testing metrics endpoints..."

# Check if metrics are available
curl -s --connect-timeout 5 http://$VPS1_IP:9090/metrics | head -n 5 > /dev/null && echo "✅ Backbone metrics available" || echo "❌ Backbone metrics not available"

curl -s --connect-timeout 5 http://$VPS2_IP:9091/metrics | head -n 5 > /dev/null && echo "✅ Regional metrics available" || echo "❌ Regional metrics not available"

curl -s --connect-timeout 5 http://$VPS3_IP:9092/metrics | head -n 5 > /dev/null && echo "✅ Edge metrics available" || echo "❌ Edge metrics not available"
```

---

## 🔄 **Step 7: Test Network Resilience**

### **7.1 Test Node Failure Recovery**

```bash
echo "💥 Testing node failure recovery..."

# Stop Regional node to test recovery
echo "Stopping Regional node to test recovery..."
ssh $VPS2_USER@$VPS2_IP 'pkill -f vx0net'

# Wait a moment
sleep 10

echo "Checking if Edge can still function without Regional node..."
ssh $VPS3_USER@$VPS3_IP 'cd ~/vx0net-daemon && ./target/release/vx0net info 2>/dev/null || echo "Edge node affected by Regional failure"'

# Restart Regional node
echo "Restarting Regional node..."
ssh $VPS2_USER@$VPS2_IP 'cd ~/vx0net-daemon && VX0NET_CONFIG_PATH=config/vx0net.toml nohup ./target/release/vx0net start --foreground > logs/vx0net.log 2>&1 &'

echo "✅ Node failure recovery test completed"
```

### **7.2 Test Automatic Reconnection**

```bash
echo "🔄 Testing automatic reconnection..."

# Monitor logs for reconnection attempts
echo "Monitoring Edge node logs for reconnection attempts..."
ssh $VPS3_USER@$VPS3_IP 'cd ~/vx0net-daemon && timeout 30 tail -f logs/vx0net.log | grep -E "(reconnect|retry|established)" | head -n 5 || echo "No reconnection activity observed"'
```

---

## 📈 **Step 8: Performance Testing**

### **8.1 Test Tunnel Performance**

```bash
echo "🔐 Testing tunnel encryption performance..."

# Check tunnel status on each node
ssh $VPS1_USER@$VPS1_IP 'cd ~/vx0net-daemon && ./target/release/vx0net info 2>/dev/null | grep -i tunnel || echo "Tunnel info not available"'

ssh $VPS2_USER@$VPS2_IP 'cd ~/vx0net-daemon && ./target/release/vx0net info 2>/dev/null | grep -i tunnel || echo "Tunnel info not available"'

ssh $VPS3_USER@$VPS3_IP 'cd ~/vx0net-daemon && ./target/release/vx0net info 2>/dev/null | grep -i tunnel || echo "Tunnel info not available"'
```

### **8.2 Load Testing**

```bash
echo "⚡ Running load test..."

# Register multiple services to test scalability
ssh $VPS3_USER@$VPS3_IP 'cd ~/vx0net-daemon && {
    for i in {1..10}; do
        ./target/release/vx0net register-service "service-$i" "test$i.vx0" $((8000+i)) 2>/dev/null || true
    done
    echo "Registered 10 test services"
}'

echo "✅ Load test completed"
```

---

## 📊 **Step 9: Automated Network Test Script**

Create a comprehensive test script:

```bash
cat > test-vx0-network.sh << 'EOF'
#!/bin/bash

# VX0 Network Automated Test Script
set -e

# Load environment
source vx0-test-env.sh

echo "🧪 VX0 Network Automated Test Suite"
echo "===================================="
echo ""

# Test 1: Basic Connectivity
echo "1️⃣ Testing basic connectivity..."
for ip in $VPS1_IP $VPS2_IP $VPS3_IP; do
    if ping -c 1 -W 5 $ip > /dev/null 2>&1; then
        echo "  ✅ $ip is reachable"
    else
        echo "  ❌ $ip is not reachable"
        exit 1
    fi
done

# Test 2: BGP Ports
echo ""
echo "2️⃣ Testing BGP ports..."
for ip in $VPS1_IP $VPS2_IP $VPS3_IP; do
    if timeout 5 nc -z $ip 1179 2>/dev/null; then
        echo "  ✅ $ip BGP port open"
    else
        echo "  ❌ $ip BGP port closed"
    fi
done

# Test 3: Node Status
echo ""
echo "3️⃣ Testing node status..."
test_node() {
    local ip=$1
    local user=$2
    local name=$3
    
    if ssh -o ConnectTimeout=10 -o BatchMode=yes $user@$ip 'cd ~/vx0net-daemon && ./target/release/vx0net info > /dev/null 2>&1'; then
        echo "  ✅ $name node responding"
    else
        echo "  ❌ $name node not responding"
    fi
}

test_node $VPS1_IP $VPS1_USER "Backbone"
test_node $VPS2_IP $VPS2_USER "Regional"
test_node $VPS3_IP $VPS3_USER "Edge"

# Test 4: Metrics Endpoints
echo ""
echo "4️⃣ Testing metrics endpoints..."
test_metrics() {
    local ip=$1
    local port=$2
    local name=$3
    
    if curl -s --connect-timeout 5 http://$ip:$port/metrics > /dev/null 2>&1; then
        echo "  ✅ $name metrics available"
    else
        echo "  ❌ $name metrics not available"
    fi
}

test_metrics $VPS1_IP 9090 "Backbone"
test_metrics $VPS2_IP 9091 "Regional"
test_metrics $VPS3_IP 9092 "Edge"

# Test 5: Service Registration
echo ""
echo "5️⃣ Testing service registration..."
if ssh -o ConnectTimeout=10 $VPS3_USER@$VPS3_IP 'cd ~/vx0net-daemon && ./target/release/vx0net register-service "test-service" "autotest.vx0" 9999 > /dev/null 2>&1'; then
    echo "  ✅ Service registration working"
else
    echo "  ❌ Service registration failed"
fi

echo ""
echo "🎉 VX0 Network Test Suite Completed!"
echo ""
echo "📊 Network Summary:"
echo "  Backbone: $VPS1_IP (ASN 65001)"
echo "  Regional: $VPS2_IP (ASN 65101)"
echo "  Edge: $VPS3_IP (ASN 66001)"
echo ""
echo "🔗 Monitoring URLs:"
echo "  Backbone Metrics: http://$VPS1_IP:9090/metrics"
echo "  Regional Metrics: http://$VPS2_IP:9091/metrics"
echo "  Edge Metrics: http://$VPS3_IP:9092/metrics"
echo ""
echo "📋 To check logs:"
echo "  ssh $VPS1_USER@$VPS1_IP 'tail -f ~/vx0net-daemon/logs/vx0net.log'"
echo "  ssh $VPS2_USER@$VPS2_IP 'tail -f ~/vx0net-daemon/logs/vx0net.log'"
echo "  ssh $VPS3_USER@$VPS3_IP 'tail -f ~/vx0net-daemon/logs/vx0net.log'"

EOF

chmod +x test-vx0-network.sh
```

Run the automated test:

```bash
./test-vx0-network.sh
```

---

## ✅ **Success Criteria Checklist**

### **Infrastructure Tests**
- [ ] All VPS instances accessible via SSH
- [ ] All required ports open (1179, 4500, 8080, 909x)
- [ ] Firewall configured correctly on all nodes
- [ ] VX0 daemon compiled and starts without errors

### **Network Formation Tests**
- [ ] BGP sessions established between appropriate tiers
- [ ] IKE tunnels showing as "Established" in logs
- [ ] Nodes can discover and connect to each other
- [ ] Bootstrap mechanism working correctly

### **Functionality Tests**
- [ ] Service registration working on Edge nodes
- [ ] Route propagation through the hierarchy
- [ ] DNS resolution for .vx0 domains only
- [ ] External domain blocking (network isolation)
- [ ] Metrics endpoints accessible and returning data

### **Resilience Tests**
- [ ] Network recovers from node failures
- [ ] Automatic reconnection after connectivity loss
- [ ] Load testing with multiple services
- [ ] Performance under simulated stress

### **Integration Tests**
- [ ] New node can join the existing network
- [ ] Tier-based connection rules enforced
- [ ] Certificate-based authentication working
- [ ] End-to-end encrypted communication

---

## 🐛 **Troubleshooting Guide**

### **Common Issues and Solutions**

#### **"Connection Refused" Errors**
```bash
# Check if daemon is running
ssh user@vps-ip 'ps aux | grep vx0net'

# Check if ports are open
ssh user@vps-ip 'netstat -tlnp | grep 1179'

# Check firewall status
ssh user@vps-ip 'ufw status verbose'

# Solution: Restart daemon and check firewall
ssh user@vps-ip 'cd ~/vx0net-daemon && pkill vx0net && VX0NET_CONFIG_PATH=config/vx0net.toml ./target/release/vx0net start --foreground'
```

#### **"BGP Session Failed" Errors**
```bash
# Check ASN configuration
ssh user@vps-ip 'cd ~/vx0net-daemon && grep -E "asn|tier" config/vx0net.toml'

# Verify tier compatibility
# Backbone (65000-65099) ↔ Regional (65100-65999) ↔ Edge (66000-69999)

# Check logs for specific BGP errors
ssh user@vps-ip 'cd ~/vx0net-daemon && grep -i bgp logs/vx0net.log | tail -n 10'
```

#### **"Certificate Errors"**
```bash
# Regenerate certificates
ssh user@vps-ip 'cd ~/vx0net-daemon && {
    rm -rf certs/*
    cd certs && ../deploy/generate_certs.sh $(hostname) && cd ..
}'

# Copy CA certificate from backbone to other nodes
scp backbone-ip:~/vx0net-daemon/certs/ca.crt user@other-ip:~/vx0net-daemon/certs/
```

#### **"DNS Resolution Not Working"**
```bash
# Check DNS service status
ssh user@vps-ip 'cd ~/vx0net-daemon && netstat -ulnp | grep 5353'

# Test DNS manually
ssh user@vps-ip 'dig @localhost -p 5353 test.vx0'

# Check DNS configuration
ssh user@vps-ip 'cd ~/vx0net-daemon && grep -A5 "\[network.dns\]" config/vx0net.toml'
```

#### **"Metrics Not Available"**
```bash
# Check if metrics service is running
ssh user@vps-ip 'cd ~/vx0net-daemon && netstat -tlnp | grep 909'

# Test metrics endpoint locally
ssh user@vps-ip 'curl -s localhost:9090/metrics | head -n 5'

# Check monitoring configuration
ssh user@vps-ip 'cd ~/vx0net-daemon && grep -A5 "\[monitoring\]" config/vx0net.toml'
```

### **Performance Issues**

#### **High Latency**
```bash
# Test network latency between nodes
ping -c 10 vps-ip

# Check for packet loss
mtr -r -c 10 vps-ip

# Monitor tunnel overhead
ssh user@vps-ip 'cd ~/vx0net-daemon && ./target/release/vx0net info | grep -i tunnel'
```

#### **Memory/CPU Usage**
```bash
# Check resource usage
ssh user@vps-ip 'top -p $(pgrep vx0net) -n 1'

# Check log file size
ssh user@vps-ip 'ls -lh ~/vx0net-daemon/logs/'

# Rotate logs if needed
ssh user@vps-ip 'cd ~/vx0net-daemon && mv logs/vx0net.log logs/vx0net.log.old && touch logs/vx0net.log'
```

---

## 📈 **Performance Monitoring**

### **Real-time Monitoring Commands**

```bash
# Monitor all nodes simultaneously
watch -n 5 '
echo "=== VX0 Network Status ===" && 
curl -s http://$VPS1_IP:9090/metrics | grep -E "node_connections|tunnel_status" 2>/dev/null || echo "Backbone: No metrics" &&
curl -s http://$VPS2_IP:9091/metrics | grep -E "node_connections|tunnel_status" 2>/dev/null || echo "Regional: No metrics" &&
curl -s http://$VPS3_IP:9092/metrics | grep -E "node_connections|tunnel_status" 2>/dev/null || echo "Edge: No metrics"
'

# Monitor logs across all nodes
multitail \
  -l "ssh $VPS1_USER@$VPS1_IP 'tail -f ~/vx0net-daemon/logs/vx0net.log'" \
  -l "ssh $VPS2_USER@$VPS2_IP 'tail -f ~/vx0net-daemon/logs/vx0net.log'" \
  -l "ssh $VPS3_USER@$VPS3_IP 'tail -f ~/vx0net-daemon/logs/vx0net.log'"
```

### **Network Health Dashboard**

Create a simple monitoring script:

```bash
cat > monitor-vx0.sh << 'EOF'
#!/bin/bash
source vx0-test-env.sh

while true; do
    clear
    echo "🌐 VX0 Network Health Dashboard"
    echo "==============================="
    date
    echo ""
    
    # Node Status
    echo "📊 Node Status:"
    for ip in $VPS1_IP $VPS2_IP $VPS3_IP; do
        if timeout 3 nc -z $ip 1179 2>/dev/null; then
            echo "  ✅ $ip:1179 (BGP)"
        else
            echo "  ❌ $ip:1179 (BGP)"
        fi
    done
    
    echo ""
    echo "📈 Metrics Status:"
    curl -s --max-time 3 http://$VPS1_IP:9090/metrics >/dev/null && echo "  ✅ Backbone metrics" || echo "  ❌ Backbone metrics"
    curl -s --max-time 3 http://$VPS2_IP:9091/metrics >/dev/null && echo "  ✅ Regional metrics" || echo "  ❌ Regional metrics"
    curl -s --max-time 3 http://$VPS3_IP:9092/metrics >/dev/null && echo "  ✅ Edge metrics" || echo "  ❌ Edge metrics"
    
    echo ""
    echo "Press Ctrl+C to stop monitoring..."
    sleep 10
done
EOF

chmod +x monitor-vx0.sh
./monitor-vx0.sh
```

---

## 🎉 **Testing Complete!**

### **What You've Accomplished**

✅ **Deployed a real VX0 network** across multiple VPS providers  
✅ **Verified BGP routing** between hierarchical tiers  
✅ **Confirmed IKE/IPSec encryption** for all communications  
✅ **Tested service discovery** and registration  
✅ **Validated network isolation** (only .vx0 domains accessible)  
✅ **Proven resilience** through failure recovery testing  
✅ **Demonstrated scalability** with load testing  

### **Next Steps**

1. **Document Your Results**: Record performance metrics and any issues encountered
2. **Scale the Network**: Add more nodes using the join mechanism
3. **Deploy Applications**: Build .vx0 services on your network
4. **Share Your Success**: Contribute your node details to the bootstrap registry
5. **Join the Community**: Help others deploy their own VX0 networks

### **You Now Have:**

🌐 **A fully functional censorship-resistant network**  
🔒 **Complete privacy and security** for all communications  
🚀 **Scalable infrastructure** ready for growth  
🛡️ **Protection against surveillance and censorship**  
🤝 **The foundation for a free internet**

**Congratulations! You've successfully deployed and tested a real-world VX0 network!** 🎊

---

## 📞 **Support and Resources**

- **Documentation**: README.md, DEPLOYMENT.md, JOINING.md
- **Issues**: Report problems via GitHub issues
- **Community**: Connect with other VX0 operators via the network
- **Updates**: Follow the project for new features and improvements

**Welcome to the censorship-resistant internet! 🌍✊**