#!/bin/bash

# VX0 Network Auto-Update and Self-Healing System
# Automatically keeps your VX0 node updated and healthy

set -e

# Configuration
VX0_DIR="${VX0_DIR:-$HOME/vx0-network}"
UPDATE_CHECK_INTERVAL="${UPDATE_CHECK_INTERVAL:-3600}"  # 1 hour
HEALTH_CHECK_INTERVAL="${HEALTH_CHECK_INTERVAL:-300}"   # 5 minutes
LOG_FILE="$VX0_DIR/logs/auto-update.log"
LOCK_FILE="/tmp/vx0-auto-update.lock"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging function
log() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    # Create log directory if it doesn't exist
    mkdir -p "$(dirname "$LOG_FILE")"
    
    # Log to file
    echo "[$timestamp] [$level] $message" >> "$LOG_FILE"
    
    # Also log to console if running interactively
    if [ -t 1 ]; then
        case $level in
            "INFO") echo -e "${BLUE}[INFO]${NC} $message" ;;
            "WARN") echo -e "${YELLOW}[WARN]${NC} $message" ;;
            "ERROR") echo -e "${RED}[ERROR]${NC} $message" ;;
            "SUCCESS") echo -e "${GREEN}[SUCCESS]${NC} $message" ;;
            *) echo "[$level] $message" ;;
        esac
    fi
}

# Check if another instance is running
check_lock() {
    if [ -f "$LOCK_FILE" ]; then
        local pid=$(cat "$LOCK_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            log "INFO" "Another auto-update instance is running (PID: $pid)"
            exit 0
        else
            log "WARN" "Stale lock file found, removing..."
            rm -f "$LOCK_FILE"
        fi
    fi
    
    # Create lock file
    echo $$ > "$LOCK_FILE"
    trap 'rm -f "$LOCK_FILE"' EXIT
}

# Check if VX0 is installed
check_installation() {
    if [ ! -d "$VX0_DIR" ]; then
        log "ERROR" "VX0 installation not found at $VX0_DIR"
        return 1
    fi
    
    if [ ! -f "$VX0_DIR/docker-compose.yml" ]; then
        log "ERROR" "VX0 docker-compose.yml not found"
        return 1
    fi
    
    return 0
}

# Check Docker daemon
check_docker() {
    if ! command -v docker >/dev/null 2>&1; then
        log "ERROR" "Docker not found"
        return 1
    fi
    
    if ! docker info >/dev/null 2>&1; then
        log "WARN" "Docker daemon not running, attempting to start..."
        if command -v systemctl >/dev/null 2>&1; then
            sudo systemctl start docker || {
                log "ERROR" "Failed to start Docker daemon"
                return 1
            }
        else
            log "ERROR" "Cannot start Docker daemon"
            return 1
        fi
    fi
    
    return 0
}

# Check VX0 node health
check_health() {
    cd "$VX0_DIR"
    
    # Check if containers are running
    local running_containers
    running_containers=$(docker-compose ps -q 2>/dev/null | wc -l)
    
    if [ "$running_containers" -eq 0 ]; then
        log "WARN" "No VX0 containers running"
        return 1
    fi
    
    # Check if main VX0 container is healthy
    local vx0_status
    vx0_status=$(docker-compose ps vx0-edge 2>/dev/null | grep -c "Up" || echo "0")
    
    if [ "$vx0_status" -eq 0 ]; then
        log "WARN" "VX0 edge container not running"
        return 1
    fi
    
    # Check if dashboard is accessible
    if command -v curl >/dev/null 2>&1; then
        if ! curl -f -s --max-time 10 http://localhost:8090 >/dev/null 2>&1; then
            log "WARN" "VX0 dashboard not accessible"
            return 1
        fi
    fi
    
    # Check if node is connecting to network
    local peer_count
    peer_count=$(docker-compose exec -T vx0-edge vx0net peers 2>/dev/null | grep -c "Connected" || echo "0")
    
    if [ "$peer_count" -eq 0 ]; then
        log "WARN" "No peer connections detected"
        return 1
    fi
    
    log "INFO" "Health check passed: $running_containers containers, $peer_count peers"
    return 0
}

# Heal VX0 node
heal_node() {
    log "INFO" "Attempting to heal VX0 node..."
    
    cd "$VX0_DIR"
    
    # Try restarting containers
    log "INFO" "Restarting VX0 containers..."
    if docker-compose restart; then
        sleep 30  # Give containers time to start
        
        if check_health; then
            log "SUCCESS" "Node healed successfully"
            return 0
        fi
    fi
    
    # If restart didn't work, try full stop/start
    log "INFO" "Performing full restart..."
    docker-compose down
    sleep 10
    docker-compose up -d
    
    sleep 60  # Give more time for full startup
    
    if check_health; then
        log "SUCCESS" "Node healed with full restart"
        return 0
    fi
    
    # If still not working, try updating images
    log "INFO" "Attempting to update Docker images..."
    docker-compose pull
    docker-compose up -d
    
    sleep 60
    
    if check_health; then
        log "SUCCESS" "Node healed with image update"
        return 0
    fi
    
    log "ERROR" "Unable to heal node automatically"
    return 1
}

# Check for updates
check_updates() {
    log "INFO" "Checking for VX0 updates..."
    
    cd "$VX0_DIR"
    
    # Pull latest images
    local pull_output
    pull_output=$(docker-compose pull 2>&1)
    
    # Check if any images were updated
    if echo "$pull_output" | grep -q "Downloaded newer image"; then
        log "INFO" "New VX0 version available, updating..."
        return 0  # Update available
    else
        log "INFO" "VX0 is up to date"
        return 1  # No update needed
    fi
}

# Apply updates
apply_updates() {
    log "INFO" "Applying VX0 updates..."
    
    cd "$VX0_DIR"
    
    # Gracefully restart with new images
    if docker-compose up -d; then
        sleep 30
        
        if check_health; then
            log "SUCCESS" "Update applied successfully"
            return 0
        else
            log "ERROR" "Update applied but health check failed"
            heal_node
            return $?
        fi
    else
        log "ERROR" "Failed to apply update"
        return 1
    fi
}

# Check system resources
check_resources() {
    # Check disk space
    local disk_usage
    disk_usage=$(df "$VX0_DIR" | awk 'NR==2 {print $5}' | sed 's/%//')
    
    if [ "$disk_usage" -gt 90 ]; then
        log "WARN" "Disk usage is ${disk_usage}%, cleaning up..."
        
        # Clean up Docker
        docker system prune -f >/dev/null 2>&1 || true
        
        # Clean up logs older than 7 days
        find "$VX0_DIR/logs" -name "*.log" -mtime +7 -delete 2>/dev/null || true
        
        log "INFO" "Cleanup completed"
    fi
    
    # Check memory usage
    local memory_usage
    memory_usage=$(free | awk 'NR==2{printf "%.0f", $3*100/$2}')
    
    if [ "$memory_usage" -gt 90 ]; then
        log "WARN" "Memory usage is ${memory_usage}%, restarting containers..."
        cd "$VX0_DIR"
        docker-compose restart
    fi
}

# Send status notification
send_notification() {
    local status=$1
    local message=$2
    
    # If notification webhook is configured, send notification
    if [ -n "${VX0_WEBHOOK_URL:-}" ]; then
        curl -s -X POST "$VX0_WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d "{\"status\":\"$status\",\"message\":\"$message\",\"node\":\"$(hostname)\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}" \
            >/dev/null 2>&1 || true
    fi
}

# Main health monitoring loop
health_monitor() {
    log "INFO" "Starting VX0 health monitor..."
    
    while true; do
        if check_health; then
            log "INFO" "Node is healthy"
        else
            log "WARN" "Node health check failed, attempting to heal..."
            if heal_node; then
                send_notification "healed" "VX0 node was healed automatically"
            else
                send_notification "error" "VX0 node healing failed, manual intervention required"
            fi
        fi
        
        # Check system resources
        check_resources
        
        sleep "$HEALTH_CHECK_INTERVAL"
    done
}

# Main update monitoring loop
update_monitor() {
    log "INFO" "Starting VX0 update monitor..."
    
    while true; do
        if check_updates; then
            log "INFO" "Updates available, applying..."
            if apply_updates; then
                send_notification "updated" "VX0 node updated successfully"
            else
                send_notification "update_failed" "VX0 node update failed"
            fi
        fi
        
        sleep "$UPDATE_CHECK_INTERVAL"
    done
}

# Install as systemd service
install_service() {
    log "INFO" "Installing VX0 auto-update service..."
    
    sudo tee /etc/systemd/system/vx0-auto-update.service >/dev/null <<EOF
[Unit]
Description=VX0 Network Auto-Update and Health Monitor
After=docker.service
Requires=docker.service

[Service]
Type=simple
User=$USER
Environment=VX0_DIR=$VX0_DIR
ExecStart=$VX0_DIR/auto-update.sh daemon
Restart=always
RestartSec=60

[Install]
WantedBy=multi-user.target
EOF
    
    sudo systemctl daemon-reload
    sudo systemctl enable vx0-auto-update
    sudo systemctl start vx0-auto-update
    
    log "SUCCESS" "VX0 auto-update service installed and started"
}

# Remove systemd service
remove_service() {
    log "INFO" "Removing VX0 auto-update service..."
    
    sudo systemctl stop vx0-auto-update 2>/dev/null || true
    sudo systemctl disable vx0-auto-update 2>/dev/null || true
    sudo rm -f /etc/systemd/system/vx0-auto-update.service
    sudo systemctl daemon-reload
    
    log "SUCCESS" "VX0 auto-update service removed"
}

# Show status
show_status() {
    echo "VX0 Auto-Update System Status"
    echo "=============================="
    
    if systemctl is-active vx0-auto-update >/dev/null 2>&1; then
        echo "✅ Auto-update service: Running"
    else
        echo "❌ Auto-update service: Not running"
    fi
    
    if check_installation && check_docker && check_health; then
        echo "✅ VX0 node health: Good"
    else
        echo "❌ VX0 node health: Issues detected"
    fi
    
    echo ""
    echo "Recent logs:"
    tail -10 "$LOG_FILE" 2>/dev/null || echo "No logs found"
}

# Main function
main() {
    case "${1:-help}" in
        "daemon")
            check_lock
            
            if ! check_installation; then
                log "ERROR" "VX0 installation check failed"
                exit 1
            fi
            
            if ! check_docker; then
                log "ERROR" "Docker check failed"
                exit 1
            fi
            
            # Start both monitors in background
            health_monitor &
            update_monitor &
            
            # Wait for all background jobs
            wait
            ;;
        "install")
            install_service
            ;;
        "remove")
            remove_service
            ;;
        "status")
            show_status
            ;;
        "heal")
            check_lock
            heal_node
            ;;
        "update")
            check_lock
            if check_updates; then
                apply_updates
            else
                log "INFO" "No updates available"
            fi
            ;;
        "help"|*)
            echo "VX0 Network Auto-Update System"
            echo ""
            echo "Usage: $0 [command]"
            echo ""
            echo "Commands:"
            echo "  daemon   Run as daemon (health + update monitoring)"
            echo "  install  Install as systemd service"
            echo "  remove   Remove systemd service"
            echo "  status   Show system status"
            echo "  heal     Force heal node now"
            echo "  update   Check and apply updates now"
            echo "  help     Show this help"
            ;;
    esac
}

# Run main function
main "$@"
