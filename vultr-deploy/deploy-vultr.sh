#!/bin/bash

# VX0 Network - Vultr Deployment Script
# Automatically deploy Backbone and Regional nodes across Vultr regions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VULTR_API_URL="https://api.vultr.com/v2"
VX0_IMAGE="ghcr.io/vx0net/daemon:latest"

# Node configuration functions (compatible with Bash 3.2+)
get_backbone_region() {
    case "$1" in
        "us-east") echo "ewr" ;;      # New York
        "us-west") echo "lax" ;;      # Los Angeles  
        "eu-west") echo "lon" ;;      # London
        "eu-central") echo "fra" ;;   # Frankfurt
        "asia-pacific") echo "sgp" ;; # Singapore
        "asia-east") echo "nrt" ;;    # Tokyo
        *) echo "" ;;
    esac
}

get_regional_region() {
    case "$1" in
        "us-central") echo "ord" ;;   # Chicago
        "us-south") echo "dfw" ;;     # Dallas
        "eu-north") echo "sto" ;;     # Stockholm
        "asia-south") echo "bom" ;;   # Mumbai
        "oceania") echo "syd" ;;      # Sydney
        "canada") echo "tor" ;;       # Toronto
        *) echo "" ;;
    esac
}

get_backbone_asn() {
    case "$1" in
        "us-east") echo "65001" ;;
        "us-west") echo "65002" ;;
        "eu-west") echo "65003" ;;
        "eu-central") echo "65004" ;;
        "asia-pacific") echo "65005" ;;
        "asia-east") echo "65006" ;;
        *) echo "" ;;
    esac
}

get_regional_asn() {
    case "$1" in
        "us-central") echo "65101" ;;
        "us-south") echo "65102" ;;
        "eu-north") echo "65103" ;;
        "asia-south") echo "65104" ;;
        "oceania") echo "65105" ;;
        "canada") echo "65106" ;;
        *) echo "" ;;
    esac
}

# Location lists for iteration
BACKBONE_LOCATIONS="us-east us-west eu-west eu-central asia-pacific asia-east"
REGIONAL_LOCATIONS="us-central us-south eu-north asia-south oceania canada"

# Vultr instance configuration
VULTR_PLAN="vc2-1c-1gb"        # 1 vCPU, 1GB RAM - $6/month
VULTR_OS="387"                 # Ubuntu 22.04 LTS
VULTR_ENABLE_IPV6="true"
VULTR_ENABLE_PRIVATE_NETWORK="false"

print_header() {
    echo -e "${CYAN}"
    echo "██╗   ██╗██╗  ██╗ ██████╗     ██╗   ██╗██╗   ██╗██╗  ████████╗██████╗ "
    echo "██║   ██║╚██╗██╔╝██╔═████╗    ██║   ██║██║   ██║██║  ╚══██╔══╝██╔══██╗"
    echo "██║   ██║ ╚███╔╝ ██║██╔██║    ██║   ██║██║   ██║██║     ██║   ██████╔╝"
    echo "╚██╗ ██╔╝ ██╔██╗ ████╔╝██║    ╚██╗ ██╔╝██║   ██║██║     ██║   ██╔══██╗"
    echo " ╚████╔╝ ██╔╝ ██╗╚██████╔╝     ╚████╔╝ ╚██████╔╝███████╗██║   ██║  ██║"
    echo "  ╚═══╝  ╚═╝  ╚═╝ ╚═════╝       ╚═══╝   ╚═════╝ ╚══════╝╚═╝   ╚═╝  ╚═╝"
    echo ""
    echo "                    VX0 Network - Vultr Deployment"
    echo "                   Deploy Backbone & Regional Nodes"
    echo -e "${NC}"
}

print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_step() {
    echo -e "${PURPLE}🔄 $1${NC}"
}

# Check prerequisites
check_prerequisites() {
    print_step "Checking prerequisites..."
    
    # Check for required tools
    local missing_tools=()
    
    if ! command -v curl >/dev/null 2>&1; then
        missing_tools+=("curl")
    fi
    
    if ! command -v jq >/dev/null 2>&1; then
        missing_tools+=("jq")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        print_info "Please install missing tools and try again"
        exit 1
    fi
    
    # Check for Vultr API key
    if [ -z "$VULTR_API_KEY" ]; then
        print_error "VULTR_API_KEY environment variable not set"
        print_info "Get your API key from: https://my.vultr.com/settings/#settingsapi"
        print_info "Then export VULTR_API_KEY=your_api_key_here"
        exit 1
    fi
    
    print_status "Prerequisites satisfied"
}

# Make Vultr API call
vultr_api() {
    local method="$1"
    local endpoint="$2"
    local data="$3"
    
    local curl_args=(-s -H "Authorization: Bearer $VULTR_API_KEY" -H "Content-Type: application/json")
    
    if [ "$method" = "POST" ]; then
        curl_args+=(-X POST -d "$data")
    elif [ "$method" = "PUT" ]; then
        curl_args+=(-X PUT -d "$data")
    elif [ "$method" = "DELETE" ]; then
        curl_args+=(-X DELETE)
    fi
    
    curl "${curl_args[@]}" "$VULTR_API_URL$endpoint"
}

# Get available regions
get_vultr_regions() {
    print_step "Fetching available Vultr regions..."
    
    vultr_api "GET" "/regions" | jq -r '.regions[] | "\(.id): \(.city), \(.country)"' | sort
}

# Get available plans
get_vultr_plans() {
    print_step "Fetching available Vultr plans..."
    
    vultr_api "GET" "/plans" | jq -r '.plans[] | select(.type == "vc2") | "\(.id): \(.vcpu_count) vCPU, \(.ram)MB RAM, \(.disk)GB SSD - $\(.monthly_cost)"' | sort
}

# Create VPS instance
create_instance() {
    local label="$1"
    local region="$2"
    local node_type="$3"
    local location="$4"
    local asn="$5"
    
    print_step "Creating VPS instance: $label in $region..."
    
    # Generate startup script
    local startup_script
    startup_script=$(cat << EOF | base64 -w 0
#!/bin/bash
set -e

# Update system
apt-get update -y
apt-get upgrade -y

# Install Docker
curl -fsSL https://get.docker.com | sh
systemctl start docker
systemctl enable docker

# Create VX0 user
useradd -m -s /bin/bash vx0net
usermod -aG docker vx0net

# Create VX0 directory
mkdir -p /opt/vx0-network/{config,certs,data,logs}
chown -R vx0net:vx0net /opt/vx0-network

# Generate node configuration
cat > /opt/vx0-network/config/vx0net.toml << 'EOFCONFIG'
[node]
node_id = "$(openssl rand -hex 16)"
tier = "$node_type"
asn = $asn
location = "$location"
hostname = "$label.vx0.network"

[network.bgp]
listen_address = "0.0.0.0"
listen_port = 1179
router_id = "$(curl -s https://ipinfo.io/ip)"

[network.dns]
listen_address = "0.0.0.0"
listen_port = 5353
vx0_dns_servers = ["10.0.0.2:53", "10.0.0.3:53"]

[security.ike]
listen_port = 4500
dh_group = 14

[services]
enable_discovery = true
discovery_port = 8080

[monitoring]
enable_metrics = true
metrics_port = 9090
log_level = "info"

[auto_discovery]
enabled = true
discovery_interval_seconds = 300
methods = ["dns", "bootstrap_registry"]
bootstrap_registry_url = "https://registry.vx0.network/bootstrap-registry.json"
EOFCONFIG

# Generate certificates
cd /opt/vx0-network/certs
openssl req -x509 -newkey rsa:2048 -keyout ca.key -out ca.crt -days 365 -nodes -subj "/CN=VX0-Network-CA"
openssl req -newkey rsa:2048 -keyout node.key -out node.csr -nodes -subj "/CN=$label.vx0.network"
openssl x509 -req -in node.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out node.crt -days 365
rm node.csr
chmod 600 *.key
chown -R vx0net:vx0net /opt/vx0-network/certs

# Create Docker Compose file
cat > /opt/vx0-network/docker-compose.yml << 'EOFCOMPOSE'
version: '3.8'

services:
  vx0-node:
    image: $VX0_IMAGE
    container_name: vx0-$node_type-$location
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
    environment:
      - VX0NET_LOG_LEVEL=info
      - VX0NET_NODE_TIER=$node_type
      - VX0NET_NODE_ASN=$asn
      - VX0NET_LOCATION=$location
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

networks:
  vx0-network:
    driver: bridge
EOFCOMPOSE

# Create systemd service
cat > /etc/systemd/system/vx0-node.service << 'EOFSERVICE'
[Unit]
Description=VX0 Network Node ($node_type)
After=docker.service
Requires=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
User=root
WorkingDirectory=/opt/vx0-network
ExecStart=/usr/bin/docker-compose up -d
ExecStop=/usr/bin/docker-compose down
ExecReload=/usr/bin/docker-compose restart

[Install]
WantedBy=multi-user.target
EOFSERVICE

# Set permissions
chown -R vx0net:vx0net /opt/vx0-network

# Enable and start service
systemctl daemon-reload
systemctl enable vx0-node
systemctl start vx0-node

# Create management scripts
cat > /opt/vx0-network/status.sh << 'EOFSTATUS'
#!/bin/bash
cd /opt/vx0-network
echo "VX0 $node_type Node Status ($location):"
docker-compose ps
echo ""
echo "Node Info:"
docker-compose exec -T vx0-node vx0net info 2>/dev/null || echo "Node starting up..."
EOFSTATUS

cat > /opt/vx0-network/update.sh << 'EOFUPDATE'
#!/bin/bash
cd /opt/vx0-network
echo "Updating VX0 Node..."
docker-compose pull
docker-compose up -d
echo "Update complete!"
EOFUPDATE

chmod +x /opt/vx0-network/*.sh

echo "VX0 $node_type Node deployment completed!"
echo "Status: /opt/vx0-network/status.sh"
echo "Update: /opt/vx0-network/update.sh"
echo "Metrics: http://\$(curl -s https://ipinfo.io/ip):9090"
EOF
)
    
    # Create instance
    local instance_data
    instance_data=$(cat << EOF
{
    "region": "$region",
    "plan": "$VULTR_PLAN",
    "os_id": $VULTR_OS,
    "label": "$label",
    "tag": "vx0-network",
    "hostname": "$label",
    "enable_ipv6": $VULTR_ENABLE_IPV6,
    "enable_private_network": $VULTR_ENABLE_PRIVATE_NETWORK,
    "script_id": "",
    "user_data": "$startup_script"
}
EOF
)
    
    local response
    response=$(vultr_api "POST" "/instances" "$instance_data")
    
    if echo "$response" | jq -e '.instance.id' >/dev/null 2>&1; then
        local instance_id
        instance_id=$(echo "$response" | jq -r '.instance.id')
        print_status "Instance created: $instance_id"
        echo "$instance_id"
    else
        print_error "Failed to create instance: $response"
        return 1
    fi
}

# Wait for instance to be active
wait_for_instance() {
    local instance_id="$1"
    local label="$2"
    
    print_step "Waiting for instance $label to become active..."
    
    local status="pending"
    local attempts=0
    local max_attempts=60
    
    while [ "$status" != "active" ] && [ $attempts -lt $max_attempts ]; do
        sleep 10
        local response
        response=$(vultr_api "GET" "/instances/$instance_id")
        status=$(echo "$response" | jq -r '.instance.status // "unknown"')
        
        case "$status" in
            "active")
                print_status "Instance $label is now active"
                ;;
            "pending"|"installing")
                print_info "Instance $label status: $status (waiting...)"
                ;;
            *)
                print_warning "Instance $label status: $status"
                ;;
        esac
        
        ((attempts++))
    done
    
    if [ "$status" != "active" ]; then
        print_error "Instance $label failed to become active after $max_attempts attempts"
        return 1
    fi
    
    # Get instance IP
    local ip
    ip=$(echo "$response" | jq -r '.instance.main_ip // "unknown"')
    echo "$ip"
}

# Deploy backbone nodes
deploy_backbone_nodes() {
    local locations=("$@")
    
    if [ ${#locations[@]} -eq 0 ]; then
        locations=("us-east" "eu-west" "asia-pacific")
    fi
    
    print_info "Deploying Backbone nodes in locations: ${locations[*]}"
    echo ""
    
    # Use space-separated string instead of associative array for Bash 3.2 compatibility
    local backbone_instances=""
    
    for location in "${locations[@]}"; do
        local region
        region=$(get_backbone_region "$location")
        if [ -z "$region" ]; then
            print_warning "Unknown backbone location: $location (skipping)"
            continue
        fi
        
        local asn
        asn=$(get_backbone_asn "$location")
        local label="vx0-backbone-$location"
        
        print_step "Deploying Backbone node: $location"
        
        local instance_id
        instance_id=$(create_instance "$label" "$region" "Backbone" "$location" "$asn")
        
        if [ $? -eq 0 ]; then
            backbone_instances="$backbone_instances $location:$instance_id"
            print_status "Backbone node queued: $location ($instance_id)"
        else
            print_error "Failed to deploy Backbone node: $location"
        fi
        
        echo ""
    done
    
    # Wait for all instances to become active
    echo ""
    print_info "Waiting for all Backbone nodes to become active..."
    echo ""
    
    local backbone_ips=""
    
    for entry in $backbone_instances; do
        if [ -n "$entry" ]; then
            local location="${entry%:*}"
            local instance_id="${entry#*:}"
            local ip
            ip=$(wait_for_instance "$instance_id" "backbone-$location")
            
            if [ $? -eq 0 ]; then
                backbone_ips="$backbone_ips $location:$ip"
            fi
        fi
    done
    
    # Display summary
    echo ""
    print_status "Backbone nodes deployment completed!"
    echo ""
    echo "📊 Deployed Backbone Nodes:"
    for entry in $backbone_ips; do
        if [ -n "$entry" ]; then
            local location="${entry%:*}"
            local ip="${entry#*:}"
            local asn
            asn=$(get_backbone_asn "$location")
            echo "  🌐 $location: ASN $asn @ $ip"
            echo "      Dashboard: http://$ip:9090"
            echo "      Status: ssh root@$ip '/opt/vx0-network/status.sh'"
        fi
    done
    echo ""
}

# Deploy regional nodes
deploy_regional_nodes() {
    local locations=("$@")
    
    if [ ${#locations[@]} -eq 0 ]; then
        locations=("us-central" "eu-north" "asia-south")
    fi
    
    print_info "Deploying Regional nodes in locations: ${locations[*]}"
    echo ""
    
    local regional_instances=""
    
    for location in "${locations[@]}"; do
        local region
        region=$(get_regional_region "$location")
        if [ -z "$region" ]; then
            print_warning "Unknown regional location: $location (skipping)"
            continue
        fi
        
        local asn
        asn=$(get_regional_asn "$location")
        local label="vx0-regional-$location"
        
        print_step "Deploying Regional node: $location"
        
        local instance_id
        instance_id=$(create_instance "$label" "$region" "Regional" "$location" "$asn")
        
        if [ $? -eq 0 ]; then
            regional_instances="$regional_instances $location:$instance_id"
            print_status "Regional node queued: $location ($instance_id)"
        else
            print_error "Failed to deploy Regional node: $location"
        fi
        
        echo ""
    done
    
    # Wait for all instances to become active
    echo ""
    print_info "Waiting for all Regional nodes to become active..."
    echo ""
    
    local regional_ips=""
    
    for entry in $regional_instances; do
        if [ -n "$entry" ]; then
            local location="${entry%:*}"
            local instance_id="${entry#*:}"
            local ip
            ip=$(wait_for_instance "$instance_id" "regional-$location")
            
            if [ $? -eq 0 ]; then
                regional_ips="$regional_ips $location:$ip"
            fi
        fi
    done
    
    # Display summary
    echo ""
    print_status "Regional nodes deployment completed!"
    echo ""
    echo "📊 Deployed Regional Nodes:"
    for entry in $regional_ips; do
        if [ -n "$entry" ]; then
            local location="${entry%:*}"
            local ip="${entry#*:}"
            local asn
            asn=$(get_regional_asn "$location")
            echo "  🌐 $location: ASN $asn @ $ip"
            echo "      Dashboard: http://$ip:9090"
            echo "      Status: ssh root@$ip '/opt/vx0-network/status.sh'"
        fi
    done
    echo ""
}

# List existing instances
list_instances() {
    print_step "Fetching VX0 instances..."
    
    local response
    response=$(vultr_api "GET" "/instances")
    
    echo ""
    echo "📋 VX0 Network Instances:"
    echo ""
    
    if echo "$response" | jq -e '.instances' >/dev/null 2>&1; then
        echo "$response" | jq -r '.instances[] | select(.tag == "vx0-network") | "\(.label): \(.status) @ \(.main_ip) (Region: \(.region), Plan: \(.plan))"' | \
        while IFS= read -r line; do
            if [[ "$line" == *"active"* ]]; then
                echo -e "  ${GREEN}✅ $line${NC}"
            elif [[ "$line" == *"pending"* ]] || [[ "$line" == *"installing"* ]]; then
                echo -e "  ${YELLOW}🔄 $line${NC}"
            else
                echo -e "  ${RED}❌ $line${NC}"
            fi
        done
    else
        print_info "No VX0 instances found"
    fi
    
    echo ""
}

# Delete instances by pattern
delete_instances() {
    local pattern="$1"
    
    if [ -z "$pattern" ]; then
        print_error "No deletion pattern provided"
        print_info "Usage: $0 delete <pattern>"
        print_info "Example: $0 delete vx0-backbone"
        return 1
    fi
    
    print_step "Finding instances matching pattern: $pattern"
    
    local response
    response=$(vultr_api "GET" "/instances")
    
    local instances
    instances=$(echo "$response" | jq -r --arg pattern "$pattern" '.instances[] | select(.tag == "vx0-network" and (.label | contains($pattern))) | "\(.id):\(.label)"')
    
    if [ -z "$instances" ]; then
        print_info "No instances found matching pattern: $pattern"
        return 0
    fi
    
    echo ""
    echo "Found instances to delete:"
    echo "$instances" | while IFS=':' read -r id label; do
        echo "  - $label ($id)"
    done
    echo ""
    
    read -p "Are you sure you want to delete these instances? (y/N): " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Deletion cancelled"
        return 0
    fi
    
    echo "$instances" | while IFS=':' read -r id label; do
        print_step "Deleting instance: $label"
        local delete_response
        delete_response=$(vultr_api "DELETE" "/instances/$id")
        
        if [ $? -eq 0 ]; then
            print_status "Deleted: $label"
        else
            print_error "Failed to delete: $label"
        fi
    done
}

# Show help
show_help() {
    echo "VX0 Network - Vultr Deployment Script"
    echo ""
    echo "Prerequisites:"
    echo "  export VULTR_API_KEY=your_api_key_here"
    echo "  Get API key: https://my.vultr.com/settings/#settingsapi"
    echo ""
    echo "Commands:"
    echo "  deploy-backbone [locations...]   Deploy Backbone nodes"
    echo "  deploy-regional [locations...]   Deploy Regional nodes" 
    echo "  deploy-all                      Deploy both Backbone and Regional nodes"
    echo "  list                            List existing VX0 instances"
    echo "  delete <pattern>                Delete instances matching pattern"
    echo "  regions                         Show available Vultr regions"
    echo "  plans                           Show available Vultr plans"
    echo "  help                            Show this help"
    echo ""
        echo "Available Backbone Locations:"
    for location in $BACKBONE_LOCATIONS; do
        local region
        region=$(get_backbone_region "$location")
        echo "  $location ($region)"
    done
    echo ""
    echo "Available Regional Locations:"
    for location in $REGIONAL_LOCATIONS; do
        local region
        region=$(get_regional_region "$location")
        echo "  $location ($region)"
    done
    echo ""
    echo "Examples:"
    echo "  $0 deploy-backbone us-east eu-west"
    echo "  $0 deploy-regional us-central asia-south"
    echo "  $0 deploy-all"
    echo "  $0 list"
    echo "  $0 delete vx0-backbone"
}

# Main function
main() {
    print_header
    
    case "${1:-help}" in
        deploy-backbone)
            shift
            check_prerequisites
            deploy_backbone_nodes "$@"
            ;;
        deploy-regional)
            shift
            check_prerequisites
            deploy_regional_nodes "$@"
            ;;
        deploy-all)
            check_prerequisites
            deploy_backbone_nodes
            sleep 5
            deploy_regional_nodes
            ;;
        list)
            check_prerequisites
            list_instances
            ;;
        delete)
            check_prerequisites
            delete_instances "$2"
            ;;
        regions)
            check_prerequisites
            get_vultr_regions
            ;;
        plans)
            check_prerequisites
            get_vultr_plans
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "Unknown command: $1"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
