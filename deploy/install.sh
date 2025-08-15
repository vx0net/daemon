#!/bin/bash

# VX0 Network Node Installation Script
# Run this script on each VPS to set up a VX0 network node

set -e

echo "=== VX0 Network Node Installation ==="
echo ""

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   echo "⚠️  This script should not be run as root. Please run as a regular user with sudo access."
   exit 1
fi

# Update system
echo "📦 Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install dependencies
echo "🔧 Installing dependencies..."
sudo apt install -y curl build-essential pkg-config libssl-dev

# Install Rust if not already installed
if ! command -v cargo &> /dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✅ Rust already installed"
fi

# Create VX0 directory
VX0_DIR="$HOME/vx0-network"
echo "📁 Creating VX0 directory: $VX0_DIR"
mkdir -p "$VX0_DIR"
cd "$VX0_DIR"

# Clone or copy the VX0 daemon code
echo "📥 Setting up VX0 daemon..."
if [ ! -d "vx0net-daemon" ]; then
    # If you have the code in git, clone it:
    # git clone https://github.com/your-username/vx0net-daemon.git
    
    # For now, we'll create the directory structure
    mkdir -p vx0net-daemon
    echo "ℹ️  Please copy your vx0net-daemon source code to $VX0_DIR/vx0net-daemon/"
    echo "   You can use scp to copy from your development machine:"
    echo "   scp -r ./vx0net-daemon user@your-vps-ip:$VX0_DIR/"
fi

# Create directories
mkdir -p "$VX0_DIR/logs"
mkdir -p "$VX0_DIR/certs"
mkdir -p "$VX0_DIR/config"

# Create systemd service
echo "🖥️  Creating systemd service..."
sudo tee /etc/systemd/system/vx0net.service > /dev/null <<EOF
[Unit]
Description=VX0 Network Daemon
After=network.target
Wants=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$VX0_DIR/vx0net-daemon
ExecStart=$VX0_DIR/vx0net-daemon/target/release/vx0net start --foreground
Restart=always
RestartSec=10
StandardOutput=append:$VX0_DIR/logs/vx0net.log
StandardError=append:$VX0_DIR/logs/vx0net-error.log

# Environment
Environment=RUST_LOG=info
Environment=VX0NET_CONFIG_PATH=$VX0_DIR/config/vx0net.toml

[Install]
WantedBy=multi-user.target
EOF

# Create firewall rules
echo "🔥 Configuring firewall..."
sudo ufw allow 1179/tcp comment "VX0 BGP"
sudo ufw allow 4500/udp comment "VX0 IKE"
sudo ufw allow 8080:8090/tcp comment "VX0 Services"
sudo ufw allow 9090:9099/tcp comment "VX0 Monitoring"

echo ""
echo "✅ VX0 Network node installation complete!"
echo ""
echo "Next steps:"
echo "1. Copy your vx0net-daemon code to: $VX0_DIR/vx0net-daemon/"
echo "2. Copy your node configuration to: $VX0_DIR/config/vx0net.toml"
echo "3. Generate certificates (see generate_certs.sh)"
echo "4. Build the daemon: cd $VX0_DIR/vx0net-daemon && cargo build --release"
echo "5. Start the service: sudo systemctl enable vx0net && sudo systemctl start vx0net"
echo "6. Check status: sudo systemctl status vx0net"
echo "7. View logs: tail -f $VX0_DIR/logs/vx0net.log"
echo ""
echo "Configuration file template available in deploy/ directory"