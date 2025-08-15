#!/bin/bash

# VX0 Network Global Deployment Orchestrator
# Master script to deploy VX0 network across multiple VPS and Kubernetes clusters

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

print_status() { echo -e "${GREEN}✅ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
print_error() { echo -e "${RED}❌ $1${NC}"; }
print_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }

show_banner() {
    echo -e "${BLUE}"
    cat << "EOF"
    ╔══════════════════════════════════════════════════════════════════╗
    ║                    VX0 Network Global Deployment                 ║
    ║              Censorship-Resistant Network Infrastructure         ║
    ╚══════════════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

show_help() {
    echo "VX0 Network Global Deployment Orchestrator"
    echo ""
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  k8s-global         Deploy to Kubernetes globally"
    echo "  vps-setup          Setup individual VPS"
    echo "  docker-local       Local Docker development"
    echo "  status             Show global network status"
    echo "  monitor            Start monitoring dashboard"
    echo "  cleanup            Clean up all deployments"
    echo ""
    echo "Examples:"
    echo "  $0 k8s-global us-east eu-west asia-pacific"
    echo "  $0 vps-setup 203.0.113.1"
    echo "  $0 docker-local edge"
    echo "  $0 status"
    echo ""
}

# Check prerequisites
check_prerequisites() {
    local missing_tools=()
    
    command -v docker >/dev/null 2>&1 || missing_tools+=("docker")
    command -v kubectl >/dev/null 2>&1 || missing_tools+=("kubectl")
    command -v jq >/dev/null 2>&1 || missing_tools+=("jq")
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        echo "Please install them and try again."
        exit 1
    fi
    
    print_status "Prerequisites check passed"
}

# Build Docker image if needed
build_image() {
    if ! docker images | grep -q "vx0net-daemon"; then
        print_info "Building VX0 Network Daemon Docker image..."
        docker build -t vx0net-daemon:latest "$SCRIPT_DIR"
        print_status "Docker image built"
    else
        print_status "Docker image already exists"
    fi
}

# Deploy to Kubernetes globally
deploy_k8s_global() {
    local locations=("$@")
    
    print_info "Deploying VX0 network to Kubernetes globally..."
    
    check_prerequisites
    build_image
    
    # Check if kubectl can connect
    if ! kubectl cluster-info >/dev/null 2>&1; then
        print_error "Cannot connect to Kubernetes cluster"
        print_info "Please ensure kubectl is configured correctly"
        exit 1
    fi
    
    # Run K8s deployment script
    if [ ${#locations[@]} -eq 0 ]; then
        "$SCRIPT_DIR/k8s/deploy-global.sh" deploy
    else
        "$SCRIPT_DIR/k8s/deploy-global.sh" deploy "${locations[@]}"
    fi
    
    print_status "Kubernetes global deployment completed"
}

# Setup individual VPS
setup_vps() {
    local vps_ip=$1
    
    if [ -z "$vps_ip" ]; then
        print_error "VPS IP address required"
        echo "Usage: $0 vps-setup <VPS_IP>"
        exit 1
    fi
    
    print_info "Setting up VPS at $vps_ip..."
    
    # Copy setup script to VPS and execute
    echo "You'll need to run this command on the VPS (as root):"
    echo ""
    echo -e "${YELLOW}curl -fsSL https://raw.githubusercontent.com/vx0net/daemon/main/vps-deploy/setup-vps.sh | bash${NC}"
    echo ""
    echo "Or manually:"
    echo -e "${YELLOW}scp vps-deploy/setup-vps.sh root@$vps_ip:/tmp/${NC}"
    echo -e "${YELLOW}ssh root@$vps_ip 'chmod +x /tmp/setup-vps.sh && /tmp/setup-vps.sh'${NC}"
    
    print_status "VPS setup instructions provided"
}

# Local Docker development
deploy_docker_local() {
    local node_type=${1:-edge}
    
    print_info "Starting local Docker development environment..."
    
    build_image
    
    case $node_type in
        edge|regional|backbone|dev)
            "$SCRIPT_DIR/scripts/docker-deploy.sh" "$node_type"
            ;;
        *)
            print_error "Invalid node type: $node_type"
            echo "Valid types: edge, regional, backbone, dev"
            exit 1
            ;;
    esac
    
    print_status "Local Docker environment started"
}

# Show global network status
show_status() {
    print_info "Checking global VX0 network status..."
    
    echo ""
    echo -e "${BLUE}🐳 Local Docker Status:${NC}"
    if command -v docker-compose >/dev/null 2>&1; then
        docker-compose ps 2>/dev/null || echo "No local Docker deployment found"
    fi
    
    echo ""
    echo -e "${BLUE}☸️  Kubernetes Status:${NC}"
    if kubectl cluster-info >/dev/null 2>&1; then
        if kubectl get namespace vx0-network >/dev/null 2>&1; then
            "$SCRIPT_DIR/k8s/deploy-global.sh" status
        else
            echo "No Kubernetes deployment found"
        fi
    else
        echo "Kubernetes not accessible"
    fi
    
    echo ""
    echo -e "${BLUE}🌐 Global Registry Status:${NC}"
    if command -v curl >/dev/null 2>&1; then
        local registry_urls=(
            "https://registry.vx0.network/bootstrap-registry.json"
            "http://backbone-us-east.vx0.network:8080/bootstrap-registry.json"
            "http://backbone-eu-west.vx0.network:8080/bootstrap-registry.json"
            "http://backbone-asia-pacific.vx0.network:8080/bootstrap-registry.json"
        )
        
        for url in "${registry_urls[@]}"; do
            if curl -s --max-time 5 "$url" >/dev/null 2>&1; then
                print_status "Registry accessible: $url"
                break
            else
                echo "Registry not accessible: $url"
            fi
        done
    fi
}

# Start monitoring dashboard
start_monitoring() {
    print_info "Starting monitoring dashboard..."
    
    # Try local Docker monitoring first
    if docker-compose ps | grep -q "grafana"; then
        print_status "Local Grafana dashboard: http://localhost:3000"
        print_info "Default login: admin / vx0-admin-change-me"
    fi
    
    # Check Kubernetes monitoring
    if kubectl get namespace vx0-network >/dev/null 2>&1; then
        if kubectl get service grafana -n vx0-network >/dev/null 2>&1; then
            local grafana_ip
            grafana_ip=$(kubectl get service grafana -n vx0-network -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null)
            if [ -n "$grafana_ip" ]; then
                print_status "Kubernetes Grafana dashboard: http://$grafana_ip:3000"
            else
                print_info "Kubernetes Grafana service exists but no external IP yet"
            fi
        fi
    fi
    
    print_status "Monitoring dashboard information provided"
}

# Clean up all deployments
cleanup_all() {
    print_warning "This will remove ALL VX0 network deployments!"
    read -p "Are you sure? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Cleaning up VX0 deployments..."
        
        # Clean up Docker
        if command -v docker-compose >/dev/null 2>&1; then
            docker-compose down -v 2>/dev/null || true
            print_status "Local Docker cleaned up"
        fi
        
        # Clean up Kubernetes
        if kubectl cluster-info >/dev/null 2>&1; then
            "$SCRIPT_DIR/k8s/deploy-global.sh" cleanup 2>/dev/null || true
            print_status "Kubernetes cleaned up"
        fi
        
        print_status "Cleanup completed"
    else
        print_info "Cleanup cancelled"
    fi
}

# Main function
main() {
    show_banner
    
    case "${1:-help}" in
        k8s-global)
            shift
            deploy_k8s_global "$@"
            ;;
        vps-setup)
            setup_vps "$2"
            ;;
        docker-local)
            deploy_docker_local "$2"
            ;;
        status)
            show_status
            ;;
        monitor)
            start_monitoring
            ;;
        cleanup)
            cleanup_all
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
