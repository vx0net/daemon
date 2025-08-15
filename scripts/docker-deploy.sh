#!/bin/bash

# VX0 Network Daemon - Docker Deployment Script
# This script helps deploy VX0 nodes using Docker

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}🐳 VX0 Network Daemon - Docker Deployment${NC}"
echo "=========================================="
echo ""

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if Docker is installed
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        echo "  Visit: https://docs.docker.com/get-docker/"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed. Please install Docker Compose first."
        echo "  Visit: https://docs.docker.com/compose/install/"
        exit 1
    fi
    
    print_status "Docker and Docker Compose are installed"
}

# Generate development certificates
generate_certs() {
    local cert_dir="$PROJECT_ROOT/certs"
    
    if [ -f "$cert_dir/ca.crt" ] && [ -f "$cert_dir/node.crt" ]; then
        print_status "Certificates already exist"
        return 0
    fi
    
    echo -e "${BLUE}🔐 Generating development certificates...${NC}"
    mkdir -p "$cert_dir"
    
    # Generate CA
    openssl req -x509 -newkey rsa:4096 -keyout "$cert_dir/ca.key" -out "$cert_dir/ca.crt" \
        -days 365 -nodes -subj "/CN=VX0-Development-CA" 2>/dev/null
    
    # Generate node certificates for each type
    for node_type in edge regional backbone; do
        openssl req -newkey rsa:4096 -keyout "$cert_dir/${node_type}.key" \
            -out "$cert_dir/${node_type}.csr" -nodes \
            -subj "/CN=${node_type}-docker.vx0" 2>/dev/null
        
        openssl x509 -req -in "$cert_dir/${node_type}.csr" \
            -CA "$cert_dir/ca.crt" -CAkey "$cert_dir/ca.key" \
            -CAcreateserial -out "$cert_dir/${node_type}.crt" -days 365 2>/dev/null
        
        rm "$cert_dir/${node_type}.csr"
    done
    
    # Set permissions
    chmod 600 "$cert_dir"/*.key 2>/dev/null || true
    chmod 644 "$cert_dir"/*.crt 2>/dev/null || true
    
    print_status "Development certificates generated"
}

# Show deployment options
show_options() {
    echo -e "${BLUE}📋 Available Deployment Options:${NC}"
    echo ""
    echo "1. Edge Node (Recommended for personal use)"
    echo "   - Low resource usage"
    echo "   - Easy to run at home"
    echo "   - ASN range: 66000-69999"
    echo ""
    echo "2. Regional Node (Community networks)"
    echo "   - Medium resource usage"
    echo "   - Serves local community"
    echo "   - ASN range: 65100-65999"
    echo ""
    echo "3. Backbone Node (Infrastructure providers)"
    echo "   - High resource usage"
    echo "   - Core network infrastructure"
    echo "   - ASN range: 65000-65099"
    echo ""
    echo "4. Development Stack (All nodes + monitoring)"
    echo "   - Full testing environment"
    echo "   - Includes Prometheus + Grafana"
    echo "   - Good for development"
    echo ""
}

# Deploy based on user choice
deploy_node() {
    local choice=$1
    
    cd "$PROJECT_ROOT"
    
    case $choice in
        1|edge)
            echo -e "${BLUE}🚀 Deploying Edge Node...${NC}"
            docker-compose up -d vx0-edge
            print_status "Edge node deployed successfully"
            echo ""
            echo "Node info:"
            echo "  - Web interface: http://localhost:9090"
            echo "  - BGP port: 1179"
            echo "  - IKE port: 4500"
            echo "  - Logs: docker-compose logs -f vx0-edge"
            ;;
        2|regional)
            echo -e "${BLUE}🚀 Deploying Regional Node...${NC}"
            docker-compose --profile regional up -d vx0-regional
            print_status "Regional node deployed successfully"
            echo ""
            echo "Node info:"
            echo "  - Web interface: http://localhost:9091"
            echo "  - BGP port: 1180"
            echo "  - IKE port: 4501"
            echo "  - Logs: docker-compose logs -f vx0-regional"
            ;;
        3|backbone)
            echo -e "${BLUE}🚀 Deploying Backbone Node...${NC}"
            docker-compose --profile backbone up -d vx0-backbone
            print_status "Backbone node deployed successfully"
            echo ""
            echo "Node info:"
            echo "  - Web interface: http://localhost:9092"
            echo "  - BGP port: 1181"
            echo "  - IKE port: 4502"
            echo "  - Logs: docker-compose logs -f vx0-backbone"
            ;;
        4|dev|development)
            echo -e "${BLUE}🚀 Deploying Development Stack...${NC}"
            docker-compose --profile monitoring up -d vx0-edge prometheus grafana
            print_status "Development stack deployed successfully"
            echo ""
            echo "Services available:"
            echo "  - VX0 Edge Node: http://localhost:9090"
            echo "  - Prometheus: http://localhost:9093"
            echo "  - Grafana: http://localhost:3000 (admin/vx0-admin-change-me)"
            echo "  - Logs: docker-compose logs -f"
            ;;
        *)
            print_error "Invalid choice: $choice"
            return 1
            ;;
    esac
}

# Show status of running containers
show_status() {
    echo -e "${BLUE}📊 Container Status:${NC}"
    docker-compose ps
    echo ""
    
    echo -e "${BLUE}📋 Quick Commands:${NC}"
    echo "  View logs:     docker-compose logs -f [service-name]"
    echo "  Stop nodes:    docker-compose down"
    echo "  Node info:     docker-compose exec vx0-edge vx0net info"
    echo "  Network status: docker-compose exec vx0-edge vx0net network-status"
    echo "  Join network:  docker-compose exec vx0-edge vx0net join"
}

# Main script logic
main() {
    check_docker
    
    cd "$PROJECT_ROOT"
    
    # Handle command line arguments
    if [ $# -gt 0 ]; then
        case $1 in
            status)
                show_status
                exit 0
                ;;
            stop)
                echo -e "${BLUE}🛑 Stopping VX0 nodes...${NC}"
                docker-compose down
                print_status "VX0 nodes stopped"
                exit 0
                ;;
            clean)
                echo -e "${BLUE}🧹 Cleaning up VX0 deployment...${NC}"
                docker-compose down -v
                docker system prune -f
                print_status "Cleanup complete"
                exit 0
                ;;
            logs)
                docker-compose logs -f ${2:-}
                exit 0
                ;;
            edge|regional|backbone|dev|development)
                generate_certs
                deploy_node $1
                echo ""
                show_status
                exit 0
                ;;
            help|--help|-h)
                echo "Usage: $0 [option]"
                echo ""
                echo "Options:"
                echo "  edge         Deploy edge node"
                echo "  regional     Deploy regional node" 
                echo "  backbone     Deploy backbone node"
                echo "  dev          Deploy development stack"
                echo "  status       Show container status"
                echo "  stop         Stop all containers"
                echo "  clean        Clean up everything"
                echo "  logs [svc]   Show logs for service"
                echo "  help         Show this help"
                exit 0
                ;;
        esac
    fi
    
    # Interactive mode
    generate_certs
    show_options
    
    echo -e "${BLUE}Choose deployment option (1-4):${NC} "
    read -r choice
    
    deploy_node "$choice"
    echo ""
    show_status
}

# Run main function
main "$@"
