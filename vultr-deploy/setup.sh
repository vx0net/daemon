#!/bin/bash

# VX0 Network - Vultr Setup Script
# First-time setup for Vultr deployment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_header() {
    echo -e "${BLUE}"
    echo "🚀 VX0 Network - Vultr Setup"
    echo "==============================="
    echo -e "${NC}"
}

print_step() {
    echo -e "${BLUE}🔄 $1${NC}"
}

print_success() {
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

# Check prerequisites
check_prerequisites() {
    print_step "Checking prerequisites..."
    
    local missing_tools=()
    
    if ! command -v curl >/dev/null 2>&1; then
        missing_tools+=("curl")
    fi
    
    if ! command -v jq >/dev/null 2>&1; then
        missing_tools+=("jq")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        echo ""
        print_info "Install missing tools:"
        
        # Detect OS and provide install commands
        if command -v apt-get >/dev/null 2>&1; then
            echo "  sudo apt-get update && sudo apt-get install -y ${missing_tools[*]}"
        elif command -v yum >/dev/null 2>&1; then
            echo "  sudo yum install -y ${missing_tools[*]}"
        elif command -v brew >/dev/null 2>&1; then
            echo "  brew install ${missing_tools[*]}"
        else
            echo "  Please install: ${missing_tools[*]}"
        fi
        
        exit 1
    fi
    
    print_success "All prerequisites satisfied"
}

# Get API key from user
get_api_key() {
    echo ""
    print_info "Setting up Vultr API access..."
    echo ""
    
    if [ -n "$VULTR_API_KEY" ]; then
        print_success "API key already set in environment"
        return 0
    fi
    
    echo "To deploy VX0 nodes on Vultr, you need an API key:"
    echo ""
    echo "1. Sign up at: https://www.vultr.com/"
    echo "2. Go to: https://my.vultr.com/settings/#settingsapi"  
    echo "3. Generate a new API key"
    echo ""
    
    while true; do
        read -p "Enter your Vultr API key: " -r api_key
        
        if [ -z "$api_key" ]; then
            print_warning "API key cannot be empty"
            continue
        fi
        
        # Test API key
        print_step "Testing API key..."
        
        if curl -s -H "Authorization: Bearer $api_key" https://api.vultr.com/v2/account | jq -e '.account' >/dev/null 2>&1; then
            print_success "API key is valid"
            
            # Save to config file
            if [ -f "$SCRIPT_DIR/vultr-config.env" ]; then
                sed -i.bak "s/VULTR_API_KEY=\".*\"/VULTR_API_KEY=\"$api_key\"/" "$SCRIPT_DIR/vultr-config.env"
            else
                echo "VULTR_API_KEY=\"$api_key\"" > "$SCRIPT_DIR/vultr-config.env"
            fi
            
            export VULTR_API_KEY="$api_key"
            print_success "API key saved to vultr-config.env"
            break
        else
            print_error "Invalid API key or connection failed"
            print_info "Please check your API key and try again"
        fi
    done
}

# Show deployment options
show_deployment_options() {
    echo ""
    print_info "Ready to deploy! Here are your options:"
    echo ""
    echo "🌐 Quick Deployment Options:"
    echo ""
    echo "1. 💰 Cost-Effective Setup (\$30/month):"
    echo "   ./deploy.sh deploy-backbone us-east eu-west asia-pacific"
    echo ""
    echo "2. 🌍 Full Global Network (\$72/month):"
    echo "   ./deploy.sh deploy-all"
    echo ""
    echo "3. 🎯 Custom Deployment:"
    echo "   ./deploy.sh deploy-backbone us-east eu-west"
    echo "   ./deploy.sh deploy-regional us-central"
    echo ""
    echo "📋 Management Commands:"
    echo "   ./deploy.sh list              # Show all instances"
    echo "   ./deploy.sh regions           # Show available regions"
    echo "   ./deploy.sh plans             # Show pricing plans"
    echo "   ./deploy.sh delete vx0        # Delete all VX0 instances"
    echo ""
    
    read -p "Would you like to deploy the cost-effective setup now? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        print_step "Deploying cost-effective Backbone nodes..."
        exec "$SCRIPT_DIR/deploy.sh" deploy-backbone us-east eu-west asia-pacific
    else
        echo ""
        print_info "Setup complete! Use the commands above to deploy when ready."
        print_info "Full documentation: vultr-deploy/README.md"
    fi
}

# Main setup flow
main() {
    print_header
    check_prerequisites
    get_api_key
    show_deployment_options
    
    echo ""
    print_success "VX0 Vultr setup completed! 🎉"
}

# Run main function
main "$@"
