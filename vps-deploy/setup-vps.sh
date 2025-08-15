#!/bin/bash

# VX0 Network VPS Setup Script
# Sets up a VPS for hosting VX0 backbone nodes using Docker/Kubernetes

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
VX0_USER="vx0net"
VX0_DIR="/opt/vx0-network"
DOCKER_COMPOSE_VERSION="2.24.0"

print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Detect VPS location from IP geolocation
detect_location() {
    print_info "Detecting VPS location..."
    
    # Get public IP
    local public_ip=$(curl -s https://ipinfo.io/ip 2>/dev/null || curl -s https://api.ipify.org 2>/dev/null || echo "unknown")
    
    # Get location info
    local location_info
    if [ "$public_ip" != "unknown" ]; then
        location_info=$(curl -s "https://ipinfo.io/$public_ip/json" 2>/dev/null || echo '{"error":"failed"}')
        local city=$(echo "$location_info" | jq -r '.city // "Unknown"' 2>/dev/null || echo "Unknown")
        local country=$(echo "$location_info" | jq -r '.country // "Unknown"' 2>/dev/null || echo "Unknown")
        local region=$(echo "$location_info" | jq -r '.region // "Unknown"' 2>/dev/null || echo "Unknown")
        
        echo "Detected location: $city, $region, $country (IP: $public_ip)"
        
        # Map to VX0 regions
        case "$country" in
            US)
                case "$region" in
                    *California*|*Washington*|*Oregon*|*Nevada*) echo "us-west" ;;
                    *) echo "us-east" ;;
                esac
                ;;
            GB|UK) echo "eu-west" ;;
            DE|FR|NL|BE) echo "eu-central" ;;
            SG|MY|TH) echo "asia-pacific" ;;
            JP|KR) echo "asia-east" ;;
            *) echo "unknown" ;;
        esac
    else
        echo "unknown"
    fi
}

# Update system packages
update_system() {
    print_info "Updating system packages..."
    
    # Detect OS
    if [ -f /etc/debian_version ]; then
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -qq
        apt-get upgrade -y -qq
        apt-get install -y -qq curl wget jq unzip software-properties-common apt-transport-https ca-certificates gnupg lsb-release
    elif [ -f /etc/redhat-release ]; then
        yum update -y -q
        yum install -y -q curl wget jq unzip
    else
        print_error "Unsupported operating system"
        exit 1
    fi
    
    print_status "System packages updated"
}

# Install Docker
install_docker() {
    print_info "Installing Docker..."
    
    if command -v docker >/dev/null 2>&1; then
        print_status "Docker already installed"
        return 0
    fi
    
    # Install Docker using official script
    curl -fsSL https://get.docker.com | sh
    
    # Add user to docker group
    usermod -aG docker "$VX0_USER" 2>/dev/null || true
    
    # Start and enable Docker
    systemctl start docker
    systemctl enable docker
    
    print_status "Docker installed and configured"
}

# Install Docker Compose
install_docker_compose() {
    print_info "Installing Docker Compose..."
    
    if command -v docker-compose >/dev/null 2>&1; then
        print_status "Docker Compose already installed"
        return 0
    fi
    
    # Install Docker Compose
    curl -L "https://github.com/docker/compose/releases/download/v${DOCKER_COMPOSE_VERSION}/docker-compose-$(uname -s)-$(uname -m)" \
        -o /usr/local/bin/docker-compose
    chmod +x /usr/local/bin/docker-compose
    
    print_status "Docker Compose installed"
}

# Install kubectl (optional, for Kubernetes deployments)
install_kubectl() {
    print_info "Installing kubectl..."
    
    if command -v kubectl >/dev/null 2>&1; then
        print_status "kubectl already installed"
        return 0
    fi
    
    # Install kubectl
    curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
    install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
    rm kubectl
    
    print_status "kubectl installed"
}

# Create VX0 user and directories
setup_user() {
    print_info "Setting up VX0 user and directories..."
    
    # Create vx0net user if it doesn't exist
    if ! id "$VX0_USER" >/dev/null 2>&1; then
        useradd -r -m -s /bin/bash -d "$VX0_DIR" "$VX0_USER"
        print_status "Created user $VX0_USER"
    fi
    
    # Create directories
    mkdir -p "$VX0_DIR"/{config,certs,logs,data}
    chown -R "$VX0_USER:$VX0_USER" "$VX0_DIR"
    
    print_status "Directories created"
}

# Configure firewall
configure_firewall() {
    print_info "Configuring firewall..."
    
    # Install and configure UFW
    if command -v ufw >/dev/null 2>&1; then
        # VX0 network ports
        ufw allow 1179/tcp comment "VX0 BGP"
        ufw allow 4500/udp comment "VX0 IKE"
        ufw allow 8080/tcp comment "VX0 Discovery"
        ufw allow 9090/tcp comment "VX0 Metrics"
        ufw allow 5353/udp comment "VX0 DNS"
        
        # SSH (ensure we don't lock ourselves out)
        ufw allow ssh
        
        # Enable UFW if not already enabled
        ufw --force enable >/dev/null 2>&1 || true
        
        print_status "Firewall configured"
    else
        print_warning "UFW not available, firewall not configured"
    fi
}

# Generate auto-discovery configuration
generate_autodiscovery_config() {
    local location=$1
    local public_ip=$2
    
    print_info "Generating auto-discovery configuration..."
    
    # Map location to ASN
    local asn
    case "$location" in
        us-east) asn=65001 ;;
        us-west) asn=65002 ;;
        eu-west) asn=65003 ;;
        eu-central) asn=65004 ;;
        asia-pacific) asn=65005 ;;
        asia-east) asn=65006 ;;
        *) asn=65099 ;;  # Default/unknown
    esac
    
    # Create enhanced bootstrap registry with all known locations
    cat > "$VX0_DIR/config/bootstrap-registry.json" <<EOF
{
  "vx0_network_bootstrap_registry": {
    "version": "1.0.0",
    "last_updated": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "description": "Auto-generated VPS bootstrap registry",
    "auto_discovery": {
      "enabled": true,
      "methods": ["dns-resolution", "bootstrap-nodes", "ip-geolocation"],
      "dns_domains": ["vx0.network", "backbone.vx0.network"],
      "discovery_interval_seconds": 300,
      "health_check_interval_seconds": 60
    },
    "backbone_nodes": [
      {
        "hostname": "backbone-us-east.vx0.network",
        "ip": "AUTO_DISCOVER",
        "asn": 65001,
        "location": "US-East-1",
        "discovery_methods": ["dns", "bootstrap"]
      },
      {
        "hostname": "backbone-us-west.vx0.network", 
        "ip": "AUTO_DISCOVER",
        "asn": 65002,
        "location": "US-West-1",
        "discovery_methods": ["dns", "bootstrap"]
      },
      {
        "hostname": "backbone-eu-west.vx0.network",
        "ip": "AUTO_DISCOVER", 
        "asn": 65003,
        "location": "EU-West-1",
        "discovery_methods": ["dns", "bootstrap"]
      },
      {
        "hostname": "backbone-eu-central.vx0.network",
        "ip": "AUTO_DISCOVER",
        "asn": 65004,
        "location": "EU-Central-1", 
        "discovery_methods": ["dns", "bootstrap"]
      },
      {
        "hostname": "backbone-asia-pacific.vx0.network",
        "ip": "AUTO_DISCOVER",
        "asn": 65005,
        "location": "Asia-Pacific-1",
        "discovery_methods": ["dns", "bootstrap"]
      },
      {
        "hostname": "backbone-asia-east.vx0.network",
        "ip": "AUTO_DISCOVER",
        "asn": 65006,
        "location": "Asia-East-1",
        "discovery_methods": ["dns", "bootstrap"]
      }
    ],
    "network_stats": {
      "total_asns_allocated": 6,
      "available_asns": {
        "backbone": 94,
        "regional": 900,
        "edge": 4000
      },
      "network_health": "excellent"
    }
  }
}
EOF
    
    # Create VPS-specific configuration
    cat > "$VX0_DIR/config/vx0net.toml" <<EOF
# VX0 Network VPS Configuration - Auto-generated
# Location: $location, ASN: $asn

[node]
hostname = "backbone-${location}.vx0.network"
asn = $asn
tier = "Backbone"
location = "$location"
ipv4_address = "$public_ip"
ipv6_address = "2001:db8::$asn"

[network.bgp]
router_id = "$public_ip"
listen_port = 1179
hold_time = 90
keepalive_time = 30

[network.dns]
listen_port = 5353
vx0_dns_servers = ["10.0.0.2:53", "10.0.0.3:53"]
cache_size = 5000

[network.routing]
max_paths = 8
local_preference = 300
med = 0

[security.ike]
listen_port = 4500
dh_group = 14
encryption_algorithm = "AES-256"
hash_algorithm = "SHA-256"
prf_algorithm = "HMAC-SHA256"

[security.certificates]
ca_cert_path = "/app/certs/ca.crt"
node_cert_path = "/app/certs/backbone.crt"
node_key_path = "/app/certs/backbone.key"

[security.encryption]
cipher = "AES-256-GCM"
key_size = 32
iv_size = 12

[services]
enable_discovery = true
discovery_port = 8080
service_ttl = 300

[monitoring]
enable_metrics = true
metrics_port = 9090
log_level = "info"

# Auto-discovery: Will dynamically discover other backbone nodes
[bootstrap]
nodes = []  # Will be populated by auto-discovery

[auto_discovery]
enabled = true
discovery_interval_seconds = 300
methods = ["dns", "bootstrap_registry", "ping_sweep"]
bootstrap_registry_url = "https://registry.vx0.network/bootstrap-registry.json"
fallback_registry_path = "/app/config/bootstrap-registry.json"

[security.psk]
default = "vx0-backbone-${location}-secure-key"
EOF
    
    chown "$VX0_USER:$VX0_USER" "$VX0_DIR/config/"*
    
    print_status "Auto-discovery configuration generated"
}

# Create systemd service
create_systemd_service() {
    print_info "Creating systemd service..."
    
    cat > /etc/systemd/system/vx0net.service <<EOF
[Unit]
Description=VX0 Network Backbone Node
After=docker.service
Requires=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
User=root
WorkingDirectory=$VX0_DIR
ExecStart=/usr/local/bin/docker-compose up -d
ExecStop=/usr/local/bin/docker-compose down
ExecReload=/usr/local/bin/docker-compose restart

[Install]
WantedBy=multi-user.target
EOF
    
    systemctl daemon-reload
    systemctl enable vx0net
    
    print_status "Systemd service created"
}

# Create docker-compose file for VPS deployment
create_docker_compose() {
    local location=$1
    local public_ip=$2
    
    print_info "Creating Docker Compose configuration..."
    
    cat > "$VX0_DIR/docker-compose.yml" <<EOF
version: '3.8'

services:
  vx0-backbone:
    image: vx0net-daemon:latest
    build:
      context: .
      dockerfile: Dockerfile
    container_name: vx0-backbone-${location}
    restart: unless-stopped
    ports:
      - "1179:1179/tcp"    # BGP
      - "4500:4500/udp"    # IKE
      - "5353:5353/udp"    # DNS
      - "8080:8080/tcp"    # Discovery
      - "9090:9090/tcp"    # Metrics
    volumes:
      - ./config/vx0net.toml:/app/vx0net.toml:ro
      - ./certs:/app/certs:ro
      - ./data:/app/data
      - ./logs:/app/logs
      - ./config/bootstrap-registry.json:/app/bootstrap-registry.json:ro
    environment:
      - VX0NET_LOG_LEVEL=info
      - VX0NET_NODE_TIER=Backbone
      - VX0NET_NODE_ASN=\${VX0NET_NODE_ASN}
      - VX0NET_LOCATION=${location}
      - VX0NET_PUBLIC_IP=${public_ip}
      - VX0NET_AUTO_DISCOVERY=true
    networks:
      - vx0-network
    sysctls:
      - net.ipv4.ip_forward=1
    cap_add:
      - NET_ADMIN
      - NET_RAW
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9090/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

  # Auto-discovery service (optional)
  vx0-discovery:
    image: nginx:alpine
    container_name: vx0-discovery-${location}
    restart: unless-stopped
    ports:
      - "8081:80/tcp"
    volumes:
      - ./config/bootstrap-registry.json:/usr/share/nginx/html/bootstrap-registry.json:ro
    networks:
      - vx0-network

networks:
  vx0-network:
    driver: bridge
EOF
    
    # Create .env file
    cat > "$VX0_DIR/.env" <<EOF
VX0NET_NODE_ASN=$(grep '^asn =' "$VX0_DIR/config/vx0net.toml" | cut -d= -f2 | tr -d ' ')
VX0NET_LOCATION=${location}
VX0NET_PUBLIC_IP=${public_ip}
EOF
    
    chown "$VX0_USER:$VX0_USER" "$VX0_DIR/docker-compose.yml" "$VX0_DIR/.env"
    
    print_status "Docker Compose configuration created"
}

# Generate certificates
generate_certificates() {
    local location=$1
    
    print_info "Generating SSL certificates..."
    
    cd "$VX0_DIR/certs"
    
    # Generate CA if not exists
    if [ ! -f ca.crt ]; then
        openssl req -x509 -newkey rsa:4096 -keyout ca.key -out ca.crt \
            -days 3650 -nodes -subj "/CN=VX0-Network-CA" 2>/dev/null
    fi
    
    # Generate backbone certificate
    local hostname="backbone-${location}.vx0.network"
    openssl req -newkey rsa:4096 -keyout backbone.key -out backbone.csr \
        -nodes -subj "/CN=${hostname}" 2>/dev/null
    
    openssl x509 -req -in backbone.csr -CA ca.crt -CAkey ca.key \
        -CAcreateserial -out backbone.crt -days 365 2>/dev/null
    
    rm backbone.csr
    
    # Set permissions
    chmod 600 *.key
    chmod 644 *.crt
    chown "$VX0_USER:$VX0_USER" *
    
    print_status "Certificates generated"
}

# Main setup function
main() {
    echo -e "${BLUE}🚀 VX0 Network VPS Setup${NC}"
    echo "========================"
    echo ""
    
    # Check if running as root
    if [ "$EUID" -ne 0 ]; then
        print_error "This script must be run as root"
        exit 1
    fi
    
    # Detect location and IP
    local public_ip=$(curl -s https://ipinfo.io/ip 2>/dev/null || curl -s https://api.ipify.org 2>/dev/null)
    local location=$(detect_location)
    
    if [ "$location" = "unknown" ]; then
        print_warning "Could not auto-detect location. Please specify manually."
        echo "Available locations: us-east, us-west, eu-west, eu-central, asia-pacific, asia-east"
        read -p "Enter location: " location
    fi
    
    echo "Setting up VPS for location: $location (IP: $public_ip)"
    echo ""
    
    # Run setup steps
    update_system
    install_docker
    install_docker_compose
    install_kubectl
    setup_user
    configure_firewall
    generate_autodiscovery_config "$location" "$public_ip"
    create_docker_compose "$location" "$public_ip"
    generate_certificates "$location"
    create_systemd_service
    
    echo ""
    print_status "VPS setup completed successfully!"
    echo ""
    echo -e "${BLUE}📋 Next Steps:${NC}"
    echo "1. Copy VX0 daemon source code to $VX0_DIR/"
    echo "2. Build Docker image: cd $VX0_DIR && docker build -t vx0net-daemon:latest ."
    echo "3. Start services: systemctl start vx0net"
    echo "4. Check status: systemctl status vx0net"
    echo "5. View logs: docker-compose -f $VX0_DIR/docker-compose.yml logs -f"
    echo ""
    echo -e "${BLUE}🌐 Auto-Discovery:${NC}"
    echo "This node will automatically discover and connect to other backbone nodes"
    echo "Registry URL: http://$public_ip:8081/bootstrap-registry.json"
    echo "Metrics: http://$public_ip:9090/metrics"
}

# Run main function
main "$@"
