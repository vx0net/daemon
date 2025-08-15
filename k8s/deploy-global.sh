#!/bin/bash

# VX0 Network Global Kubernetes Deployment Script
# Deploy backbone nodes across multiple VPS locations with auto-discovery

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Global configuration
declare -A LOCATIONS=(
    ["us-east"]="New York, USA"
    ["us-west"]="San Francisco, USA"
    ["eu-west"]="London, UK"
    ["eu-central"]="Frankfurt, Germany"
    ["asia-pacific"]="Singapore"
    ["asia-east"]="Tokyo, Japan"
)

declare -A BACKBONE_ASNS=(
    ["us-east"]="65001"
    ["us-west"]="65002"
    ["eu-west"]="65003"
    ["eu-central"]="65004"
    ["asia-pacific"]="65005"
    ["asia-east"]="65006"
)

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

# Check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."
    
    if ! command -v kubectl &> /dev/null; then
        print_error "kubectl is not installed"
        exit 1
    fi
    
    if ! command -v docker &> /dev/null; then
        print_error "docker is not installed"
        exit 1
    fi
    
    # Check if we can connect to Kubernetes
    if ! kubectl cluster-info &> /dev/null; then
        print_error "Cannot connect to Kubernetes cluster"
        exit 1
    fi
    
    print_status "Prerequisites satisfied"
}

# Create namespace and RBAC
setup_namespace() {
    print_info "Setting up Kubernetes namespace and RBAC..."
    
    kubectl apply -f - <<EOF
apiVersion: v1
kind: Namespace
metadata:
  name: vx0-network
  labels:
    name: vx0-network
    purpose: vx0-network-nodes

---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: vx0-backbone
  namespace: vx0-network

---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: vx0-discovery
  namespace: vx0-network

---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: vx0-network-reader
rules:
- apiGroups: [""]
  resources: ["services", "endpoints", "nodes"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["apps"]
  resources: ["deployments"]
  verbs: ["get", "list", "watch"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: vx0-network-reader
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: vx0-network-reader
subjects:
- kind: ServiceAccount
  name: vx0-backbone
  namespace: vx0-network
- kind: ServiceAccount
  name: vx0-discovery
  namespace: vx0-network
EOF
    
    print_status "Namespace and RBAC configured"
}

# Generate certificates for a location
generate_certificates() {
    local location=$1
    local cert_dir="$PROJECT_ROOT/k8s/certs/$location"
    
    print_info "Generating certificates for $location..."
    
    mkdir -p "$cert_dir"
    
    # Generate CA if it doesn't exist
    if [ ! -f "$PROJECT_ROOT/k8s/certs/ca.crt" ]; then
        mkdir -p "$PROJECT_ROOT/k8s/certs"
        openssl req -x509 -newkey rsa:4096 \
            -keyout "$PROJECT_ROOT/k8s/certs/ca.key" \
            -out "$PROJECT_ROOT/k8s/certs/ca.crt" \
            -days 3650 -nodes \
            -subj "/CN=VX0-Network-CA" 2>/dev/null
    fi
    
    # Generate backbone certificate for this location
    local hostname="backbone-${location}.vx0.network"
    openssl req -newkey rsa:4096 \
        -keyout "$cert_dir/backbone.key" \
        -out "$cert_dir/backbone.csr" -nodes \
        -subj "/CN=${hostname}" 2>/dev/null
    
    openssl x509 -req -in "$cert_dir/backbone.csr" \
        -CA "$PROJECT_ROOT/k8s/certs/ca.crt" \
        -CAkey "$PROJECT_ROOT/k8s/certs/ca.key" \
        -CAcreateserial \
        -out "$cert_dir/backbone.crt" -days 365 2>/dev/null
    
    rm "$cert_dir/backbone.csr"
    
    print_status "Certificates generated for $location"
}

# Deploy discovery service (global)
deploy_discovery_service() {
    print_info "Deploying global discovery service..."
    
    kubectl apply -f "$SCRIPT_DIR/discovery/discovery-service.yaml"
    
    # Wait for discovery service to be ready
    kubectl wait --for=condition=ready pod -l app=vx0-discovery -n vx0-network --timeout=120s
    
    print_status "Discovery service deployed"
}

# Deploy backbone node for a specific location
deploy_backbone_node() {
    local location=$1
    local public_ip=$2
    local asn=${BACKBONE_ASNS[$location]}
    local location_name=${LOCATIONS[$location]}
    
    print_info "Deploying backbone node for $location - $location_name..."
    
    # Generate certificates
    generate_certificates "$location"
    
    # Create secret for certificates
    kubectl create secret generic "vx0-backbone-certs-$location" \
        --from-file="$PROJECT_ROOT/k8s/certs/ca.crt" \
        --from-file="$PROJECT_ROOT/k8s/certs/$location/backbone.crt" \
        --from-file="$PROJECT_ROOT/k8s/certs/$location/backbone.key" \
        --namespace=vx0-network \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Create configmap with location-specific configuration
    local other_backbones=""
    for other_location in "${!BACKBONE_ASNS[@]}"; do
        if [ "$other_location" != "$location" ]; then
            local other_asn=${BACKBONE_ASNS[$other_location]}
            other_backbones="${other_backbones}        { hostname = \"backbone-${other_location}.vx0.network\", ip = \"AUTO_DISCOVER\", asn = ${other_asn} },\n"
        fi
    done
    
    kubectl apply -f - <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: vx0-backbone-config-$location
  namespace: vx0-network
  labels:
    app: vx0-backbone
    location: $location
data:
  asn: "$asn"
  location: "$location_name"
  public_ip: "$public_ip"
  config: |
    [node]
    hostname = "backbone-${location}.vx0.network"
    asn = $asn
    tier = "Backbone"
    location = "$location_name"
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

    # Auto-discovery of other backbone nodes
    [bootstrap]
    nodes = [
$other_backbones    ]

    [security.psk]
    default = "vx0-backbone-${location}-secure-key"
EOF
    
    # Create PVC for data persistence
    kubectl apply -f - <<EOF
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: vx0-backbone-data-$location
  namespace: vx0-network
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
  storageClassName: standard
EOF
    
    # Deploy backbone node
    sed "s/vx0-backbone/vx0-backbone-$location/g; s/vx0-backbone-config/vx0-backbone-config-$location/g; s/vx0-backbone-certs/vx0-backbone-certs-$location/g; s/vx0-backbone-data/vx0-backbone-data-$location/g" \
        "$SCRIPT_DIR/backbone/backbone-deployment.yaml" | \
    sed "s/backbone-k8s.vx0.network/backbone-${location}.vx0.network/g" | \
    kubectl apply -f -
    
    # Deploy service
    sed "s/vx0-backbone/vx0-backbone-$location/g" \
        "$SCRIPT_DIR/backbone/backbone-service.yaml" | \
    kubectl apply -f -
    
    print_status "Backbone node deployed for $location"
}

# Update discovery registry with deployed nodes
update_discovery_registry() {
    print_info "Updating discovery registry with deployed nodes..."
    
    # Get all backbone services and their external IPs
    local backbone_nodes="[]"
    for location in "${!BACKBONE_ASNS[@]}"; do
        local asn=${BACKBONE_ASNS[$location]}
        local location_name=${LOCATIONS[$location]}
        
        # Try to get external IP (may take a few minutes)
        local external_ip
        external_ip=$(kubectl get service "vx0-backbone-$location" -n vx0-network -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "PENDING")
        
        if [ "$external_ip" = "PENDING" ] || [ -z "$external_ip" ]; then
            external_ip="AUTO_DISCOVER"
        fi
        
        # Add to registry
        backbone_nodes=$(echo "$backbone_nodes" | jq ". += [{
            \"hostname\": \"backbone-${location}.vx0.network\",
            \"ip\": \"$external_ip\",
            \"asn\": $asn,
            \"location\": \"$location_name\",
            \"uptime\": \"99.9%\",
            \"operator\": \"VX0 Kubernetes\",
            \"contact\": \"admin@vx0.network\",
            \"features\": [\"bgp\", \"ike\", \"dns\", \"discovery\"],
            \"max_peers\": 50,
            \"current_peers\": 0,
            \"status\": \"active\",
            \"last_seen\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
        }]")
    done
    
    # Update discovery registry configmap
    local registry_data
    registry_data=$(cat << EOF | jq -c ".vx0_network_bootstrap_registry.backbone_nodes = $backbone_nodes"
{
  "vx0_network_bootstrap_registry": {
    "version": "1.0.0",
    "last_updated": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "description": "Kubernetes-managed VX0 network bootstrap registry",
    "auto_discovery": {
      "enabled": true,
      "update_interval_seconds": 300,
      "health_check_interval_seconds": 60,
      "methods": ["k8s-service-discovery", "dns-resolution", "bootstrap-nodes"]
    },
    "backbone_nodes": [],
    "regional_nodes": [],
    "edge_nodes": [],
    "network_stats": {
      "total_asns_allocated": ${#BACKBONE_ASNS[@]},
      "available_asns": {
        "backbone": $((100 - ${#BACKBONE_ASNS[@]})),
        "regional": 900,
        "edge": 4000
      },
      "network_health": "excellent",
      "average_latency_ms": 25
    }
  }
}
EOF
)
    
    kubectl patch configmap vx0-discovery-registry -n vx0-network --type='merge' -p="{
        \"data\": {
            \"bootstrap-registry.json\": \"$(echo "$registry_data" | sed 's/"/\\"/g')\"
        }
    }"
    
    print_status "Discovery registry updated"
}

# Wait for external IPs to be assigned
wait_for_external_ips() {
    print_info "Waiting for LoadBalancer external IPs to be assigned..."
    
    for location in "${!BACKBONE_ASNS[@]}"; do
        print_info "Waiting for external IP for $location..."
        kubectl wait --for=jsonpath='{.status.loadBalancer.ingress[0].ip}' \
            service "vx0-backbone-$location" -n vx0-network --timeout=300s || true
    done
    
    print_status "External IP assignment completed - some may still be pending"
}

# Display deployment status
show_deployment_status() {
    echo ""
    echo -e "${BLUE}🌐 VX0 Network Global Deployment Status${NC}"
    echo "========================================"
    echo ""
    
    echo "📊 Backbone Nodes:"
    for location in "${!BACKBONE_ASNS[@]}"; do
        local asn=${BACKBONE_ASNS[$location]}
        local location_name=${LOCATIONS[$location]}
        local external_ip
        external_ip=$(kubectl get service "vx0-backbone-$location" -n vx0-network -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "PENDING")
        
        echo "  $location - $location_name: ASN $asn @ $external_ip"
    done
    
    echo ""
    echo "🔍 Discovery Service:"
    local discovery_ip
    discovery_ip=$(kubectl get service vx0-discovery -n vx0-network -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "PENDING")
    echo "  Registry: $discovery_ip/registry"
    
    echo ""
    echo "📋 Quick Commands:"
    echo "  View all nodes:     kubectl get pods -n vx0-network"
    echo "  Check node logs:    kubectl logs -n vx0-network -l app=vx0-backbone-LOCATION"
    echo "  Access discovery:   curl http://$discovery_ip/registry"
    echo "  Node status:        kubectl exec -n vx0-network PODNAME -- vx0net info"
    
    echo ""
    echo "🌍 Network Auto-Discovery:"
    echo "  Backbone nodes will automatically discover each other using:"
    echo "  • Kubernetes service discovery"
    echo "  • Bootstrap node configuration"
    echo "  • BGP route announcements"
    echo "  • DNS-based discovery"
}

# Main deployment function
deploy_global_network() {
    local locations=("$@")
    
    if [ ${#locations[@]} -eq 0 ]; then
        locations=("us-east" "eu-west" "asia-pacific")
        print_info "No locations specified, deploying to default locations: ${locations[*]}"
    fi
    
    echo -e "${BLUE}🚀 VX0 Network Global Kubernetes Deployment${NC}"
    echo "============================================="
    echo ""
    
    check_prerequisites
    setup_namespace
    deploy_discovery_service
    
    # Deploy backbone nodes to specified locations
    for location in "${locations[@]}"; do
        if [[ ! ${BACKBONE_ASNS[$location]+_} ]]; then
            print_error "Unknown location: $location"
            continue
        fi
        
        # For this demo, we'll use placeholder IPs
        # In production, you'd get these from your VPS provider or load balancer
        local public_ip="203.0.113.$((${BACKBONE_ASNS[$location]} - 65000))"
        deploy_backbone_node "$location" "$public_ip"
    done
    
    wait_for_external_ips
    update_discovery_registry
    show_deployment_status
}

# Command line interface
case "${1:-deploy}" in
    deploy)
        shift
        deploy_global_network "$@"
        ;;
    status)
        show_deployment_status
        ;;
    cleanup)
        print_info "Cleaning up VX0 network deployment..."
        kubectl delete namespace vx0-network --ignore-not-found
        print_status "Cleanup completed"
        ;;
    add-location)
        if [ -z "$2" ] || [ -z "$3" ]; then
            print_error "Usage: $0 add-location LOCATION PUBLIC_IP"
            exit 1
        fi
        deploy_backbone_node "$2" "$3"
        update_discovery_registry
        ;;
    help)
        echo "Usage: $0 [command] [options]"
        echo ""
        echo "Commands:"
        echo "  deploy [locations...]   Deploy global VX0 network - default: us-east eu-west asia-pacific"
        echo "  status                  Show deployment status"
        echo "  cleanup                 Remove all VX0 network resources"
        echo "  add-location LOC IP     Add backbone node in specific location"
        echo "  help                    Show this help"
        echo ""
        echo "Available locations: ${!LOCATIONS[*]}"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac
