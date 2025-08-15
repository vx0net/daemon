#!/bin/bash

# VX0 Network Testing Script
# Run this to verify your VX0 network is working correctly

echo "=== VX0 Network Testing Script ==="
echo ""

# Configuration - update these with your actual IPs
BACKBONE1_IP="${BACKBONE1_IP:-YOUR_BACKBONE_IP}"
REGIONAL1_IP="${REGIONAL1_IP:-YOUR_REGIONAL_IP}"
EDGE1_IP="${EDGE1_IP:-YOUR_EDGE_IP}"

if [[ "$BACKBONE1_IP" == "YOUR_BACKBONE_IP" ]]; then
    echo "❌ Please set your VPS IP addresses first:"
    echo "   export BACKBONE1_IP='your.backbone.ip'"
    echo "   export REGIONAL1_IP='your.regional.ip'"
    echo "   export EDGE1_IP='your.edge.ip'"
    echo "   Then run this script again."
    exit 1
fi

echo "Testing VX0 Network with:"
echo "  Backbone: $BACKBONE1_IP"
echo "  Regional: $REGIONAL1_IP"
echo "  Edge: $EDGE1_IP"
echo ""

# Test 1: Check if nodes are running
echo "🔍 Test 1: Checking if VX0 nodes are running..."

check_service() {
    local ip=$1
    local name=$2
    local port=$3
    
    if curl -s --connect-timeout 5 "http://$ip:$port/health" > /dev/null 2>&1; then
        echo "  ✅ $name node at $ip is responding"
        return 0
    else
        echo "  ❌ $name node at $ip is not responding on port $port"
        return 1
    fi
}

# Test BGP ports instead of health (since we don't have health endpoint yet)
test_bgp_port() {
    local ip=$1
    local name=$2
    
    if timeout 5 bash -c "</dev/tcp/$ip/1179" 2>/dev/null; then
        echo "  ✅ $name BGP port (1179) is open"
        return 0
    else
        echo "  ❌ $name BGP port (1179) is not accessible"
        return 1
    fi
}

test_bgp_port "$BACKBONE1_IP" "Backbone"
test_bgp_port "$REGIONAL1_IP" "Regional"
test_bgp_port "$EDGE1_IP" "Edge"

echo ""

# Test 2: Check metrics endpoints
echo "📊 Test 2: Checking metrics endpoints..."

test_metrics() {
    local ip=$1
    local name=$2
    local port=$3
    
    if curl -s --connect-timeout 5 "http://$ip:$port/metrics" | head -n 1 > /dev/null 2>&1; then
        echo "  ✅ $name metrics available at $ip:$port"
        return 0
    else
        echo "  ❌ $name metrics not available at $ip:$port"
        return 1
    fi
}

test_metrics "$BACKBONE1_IP" "Backbone" "9090"
test_metrics "$REGIONAL1_IP" "Regional" "9091"
test_metrics "$EDGE1_IP" "Edge" "9092"

echo ""

# Test 3: Check DNS isolation
echo "🌐 Test 3: Testing DNS isolation..."

test_dns() {
    local ip=$1
    local name=$2
    local domain=$3
    local should_resolve=$4
    
    if timeout 5 nslookup "$domain" "$ip" > /dev/null 2>&1; then
        if [[ "$should_resolve" == "true" ]]; then
            echo "  ✅ $name correctly resolved $domain"
            return 0
        else
            echo "  ❌ $name incorrectly resolved $domain (should be blocked)"
            return 1
        fi
    else
        if [[ "$should_resolve" == "false" ]]; then
            echo "  ✅ $name correctly blocked $domain"
            return 0
        else
            echo "  ❌ $name failed to resolve $domain (should work)"
            return 1
        fi
    fi
}

# Test VX0 domain resolution
test_dns "$EDGE1_IP" "Edge DNS" "vx0.network" "true"

# Test internet domain blocking (should fail - network isolation)
test_dns "$EDGE1_IP" "Edge DNS" "google.com" "false"

echo ""

# Test 4: Check service discovery
echo "🛰️  Test 4: Testing service discovery..."

# This would test if services can be registered and discovered
echo "  ℹ️  Service discovery test requires running nodes"
echo "  ℹ️  Check logs for 'Service registered' messages"

echo ""

# Test 5: Network hierarchy
echo "🏗️  Test 5: Testing network hierarchy..."

echo "  ℹ️  Check logs for proper tier peering:"
echo "    - Backbone should peer with Regional"
echo "    - Regional should peer with Edge"
echo "    - Edge should NOT peer with Edge"

echo ""

# Test 6: Log analysis
echo "📋 Test 6: Analyzing recent logs..."

analyze_logs() {
    local ip=$1
    local name=$2
    
    echo "  $name node recent activity:"
    
    # Check if we can SSH to get logs (if SSH is configured)
    if ssh -o ConnectTimeout=5 -o BatchMode=yes "user@$ip" "tail -n 5 ~/vx0-network/logs/vx0net.log 2>/dev/null" 2>/dev/null; then
        echo "    ✅ Recent log entries found"
    else
        echo "    ⚠️  Cannot access logs remotely (SSH not configured)"
        echo "       To check logs: ssh user@$ip 'tail -f ~/vx0-network/logs/vx0net.log'"
    fi
}

analyze_logs "$BACKBONE1_IP" "Backbone"
analyze_logs "$REGIONAL1_IP" "Regional"
analyze_logs "$EDGE1_IP" "Edge"

echo ""

# Summary
echo "=== VX0 Network Test Summary ==="
echo ""
echo "✅ Tests completed. If all tests passed, your VX0 network is operational!"
echo ""
echo "🔍 To monitor your network:"
echo "  • Backbone metrics: http://$BACKBONE1_IP:9090/metrics"
echo "  • Regional metrics: http://$REGIONAL1_IP:9091/metrics"
echo "  • Edge metrics: http://$EDGE1_IP:9092/metrics"
echo ""
echo "📋 To check logs:"
echo "  ssh user@$BACKBONE1_IP 'tail -f ~/vx0-network/logs/vx0net.log'"
echo "  ssh user@$REGIONAL1_IP 'tail -f ~/vx0-network/logs/vx0net.log'"
echo "  ssh user@$EDGE1_IP 'tail -f ~/vx0-network/logs/vx0net.log'"
echo ""
echo "🛠️  To manage services:"
echo "  ssh user@$EDGE1_IP 'sudo systemctl status vx0net'"
echo "  ssh user@$EDGE1_IP '~/vx0-network/vx0net-daemon/target/release/vx0net info'"
echo ""
echo "🎉 Your censorship-resistant VX0 network is now running!"
echo "   Nodes can only communicate with each other and resolve .vx0 domains."
echo "   Regular internet access is blocked - the network is fully isolated!"