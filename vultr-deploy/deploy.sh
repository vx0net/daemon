#!/bin/bash

# VX0 Network - Vultr Deployment Wrapper
# Simple wrapper that loads configuration and runs the main deployment script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source configuration if available
if [ -f "$SCRIPT_DIR/vultr-config.env" ]; then
    # Export variables from config file
    set -a
    source "$SCRIPT_DIR/vultr-config.env" 
    set +a
fi

# Check if API key is set
if [ -z "$VULTR_API_KEY" ]; then
    echo -e "\033[0;31m❌ VULTR_API_KEY not set\033[0m"
    echo -e "\033[0;34mℹ️  Please set your Vultr API key:\033[0m"
    echo ""
    echo "   1. Get API key from: https://my.vultr.com/settings/#settingsapi"
    echo "   2. Export it: export VULTR_API_KEY='your_api_key_here'"
    echo "   3. Or edit: vultr-deploy/vultr-config.env"
    echo ""
    exit 1
fi

# Run the main deployment script
exec "$SCRIPT_DIR/deploy-vultr.sh" "$@"
