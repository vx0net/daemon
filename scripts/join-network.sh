#!/bin/bash

# VX0 Network Easy Join Script
# This script helps anyone join the VX0 network with minimal configuration

set -e

echo "🌐 VX0 Network Easy Join Script"
echo "==============================="
echo ""

# Check if vx0net daemon exists
if [ ! -f "target/release/vx0net" ]; then
    echo "❌ VX0 daemon not found. Please build first:"
    echo "   cargo build --release"
    exit 1
fi

# Get user's desired tier
echo "What type of node do you want to run?"
echo "1) Edge Node (easiest, connects to regional nodes)"
echo "2) Regional Node (intermediate, serves edge nodes)"
echo "3) Backbone Node (advanced, core network infrastructure)"
echo ""
read -p "Enter choice (1-3): " tier_choice

case $tier_choice in
    1)
        TIER="Edge"
        ASN_RANGE="66000-69999"
        ;;
    2)
        TIER="Regional"
        ASN_RANGE="65100-65999"
        ;;
    3)
        TIER="Backbone"
        ASN_RANGE="65000-65099"
        ;;
    *)
        echo "❌ Invalid choice. Defaulting to Edge node."
        TIER="Edge"
        ASN_RANGE="66000-69999"
        ;;
esac

echo ""
echo "Selected: $TIER node (ASN range: $ASN_RANGE)"

# Get basic node information
read -p "Enter your node hostname (e.g., mynode.vx0.network): " HOSTNAME
if [ -z "$HOSTNAME" ]; then
    HOSTNAME="node-$(date +%s).vx0.network"
    echo "Using auto-generated hostname: $HOSTNAME"
fi

read -p "Enter your location (e.g., 'New York, NY'): " LOCATION
if [ -z "$LOCATION" ]; then
    LOCATION="Unknown"
fi

# Get external IP
echo ""
echo "🔍 Detecting your external IP address..."
EXTERNAL_IP=$(curl -s ifconfig.me || curl -s ipinfo.io/ip || echo "127.0.0.1")
echo "Detected IP: $EXTERNAL_IP"
read -p "Is this correct? (y/n): " ip_confirm

if [ "$ip_confirm" != "y" ]; then
    read -p "Enter your external IP address: " EXTERNAL_IP
fi

# Generate a random ASN in the appropriate range
case $TIER in
    "Edge")
        ASN=$((66000 + RANDOM % 4000))
        ;;
    "Regional")
        ASN=$((65100 + RANDOM % 900))
        ;;
    "Backbone")
        ASN=$((65000 + RANDOM % 100))
        ;;
esac

echo ""
echo "📋 Node Configuration:"
echo "  Hostname: $HOSTNAME"
echo "  Tier: $TIER"
echo "  ASN: $ASN (auto-assigned)"
echo "  IP: $EXTERNAL_IP"
echo "  Location: $LOCATION"
echo ""

# Create config directory
mkdir -p config

# Generate configuration file
cat > config/vx0net.toml << EOF
[node]
hostname = "$HOSTNAME"
asn = $ASN
tier = "$TIER"
location = "$LOCATION"
ipv4_address = "$EXTERNAL_IP"
ipv6_address = "fe80::1"

[network.bgp]
router_id = "$EXTERNAL_IP"
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

[security.psk]
default = "vx0-network-default-psk-change-in-production"
EOF

echo "✅ Configuration file created: config/vx0net.toml"
echo ""

# Create certificates directory
mkdir -p certs

# Generate certificates if they don't exist
if [ ! -f "certs/$HOSTNAME.crt" ]; then
    echo "🔐 Generating SSL certificates..."
    if [ -f "deploy/generate_certs.sh" ]; then
        cd certs && ../deploy/generate_certs.sh "$HOSTNAME" && cd ..
    else
        echo "⚠️  Certificate generation script not found. Using self-signed certificates."
        openssl req -x509 -newkey rsa:2048 -keyout "certs/$HOSTNAME.key" -out "certs/$HOSTNAME.crt" -days 365 -nodes -subj "/CN=$HOSTNAME"
        chmod 600 "certs/$HOSTNAME.key"
    fi
    echo "✅ Certificates generated"
fi

echo ""
echo "🚀 Starting VX0 node and joining network..."
echo ""

# Create logs directory
mkdir -p logs

# Start the node with auto-join
echo "Starting node with configuration:"
cat config/vx0net.toml
echo ""

# Run the daemon with join network flag
VX0NET_CONFIG_PATH=config/vx0net.toml ./target/release/vx0net start --join-network --foreground

echo ""
echo "🎉 Welcome to the VX0 Network!"
echo ""
echo "Your node is now part of the decentralized, censorship-resistant VX0 network."
echo ""
echo "Next steps:"
echo "1. Check node status: ./target/release/vx0net status"
echo "2. View connected peers: ./target/release/vx0net peers"
echo "3. Register services: ./target/release/vx0net register-service <name> <domain> <port>"
echo "4. Monitor logs: tail -f logs/vx0net.log"
echo ""
echo "To help others join, share these bootstrap details:"
echo "  Hostname: $HOSTNAME"
echo "  IP: $EXTERNAL_IP"  
echo "  ASN: $ASN"
echo ""
echo "The network grows stronger with each new node! 🌐"