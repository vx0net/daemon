# 🐳 VX0 Network Daemon - Docker Deployment Guide

This guide shows how to deploy the VX0 Network Daemon using Docker for easier management and deployment.

## 🚀 Quick Start

### **Option 1: Edge Node (Recommended for most users)**

```bash
# Clone the repository
git clone <this-repo> && cd vx0net-daemon

# Build and start an edge node
docker-compose up -d vx0-edge

# Check logs
docker-compose logs -f vx0-edge
```

### **Option 2: Full Network Stack (Development)**

```bash
# Start edge node with monitoring
docker-compose --profile monitoring up -d vx0-edge prometheus grafana

# Access Grafana dashboard
open http://localhost:3000
# Default login: admin / vx0-admin-change-me
```

## 📋 Available Node Types

### **Edge Node** (Personal/Home Use)
- **ASN Range**: 66000-69999
- **Memory**: 512MB RAM
- **CPU**: 1 core
- **Best for**: Personal nodes, home labs

```bash
docker-compose up -d vx0-edge
```

### **Regional Node** (Community/Small Organization)
- **ASN Range**: 65100-65999  
- **Memory**: 1GB RAM
- **CPU**: 1-2 cores
- **Best for**: Community networks, small ISPs

```bash
docker-compose --profile regional up -d vx0-regional
```

### **Backbone Node** (Infrastructure Providers)
- **ASN Range**: 65000-65099
- **Memory**: 2GB+ RAM  
- **CPU**: 2+ cores
- **Best for**: Core infrastructure, high availability

```bash
docker-compose --profile backbone up -d vx0-backbone
```

## 🔧 Configuration

### **Environment Variables**

All nodes support these environment variables:

```bash
# Node configuration
VX0NET_LOG_LEVEL=info          # debug, info, warn, error
VX0NET_NODE_TIER=Edge          # Edge, Regional, Backbone
VX0NET_NODE_ASN=66001          # Your assigned ASN
VX0NET_NODE_HOSTNAME=my-node   # Your node hostname

# Network configuration
VX0NET_BGP_LISTEN_PORT=1179    # BGP port
VX0NET_IKE_LISTEN_PORT=4500    # IKE port
VX0NET_DNS_LISTEN_PORT=5353    # DNS port
VX0NET_DISCOVERY_PORT=8080     # Service discovery port
VX0NET_METRICS_PORT=9090       # Metrics port
```

### **Custom Configuration**

1. **Copy a template configuration:**
   ```bash
   cp config/edge-node.toml config/my-node.toml
   ```

2. **Edit the configuration:**
   ```bash
   # Update ASN, hostname, and IP addresses
   nano config/my-node.toml
   ```

3. **Mount your configuration:**
   ```yaml
   volumes:
     - ./config/my-node.toml:/app/vx0net.toml:ro
   ```

## 🔐 Certificate Management

### **Generate Development Certificates**

```bash
# Create certificates directory
mkdir -p certs

# Generate a simple CA (development only)
openssl req -x509 -newkey rsa:4096 -keyout certs/ca.key -out certs/ca.crt -days 365 -nodes -subj "/CN=VX0-CA"

# Generate node certificate
openssl req -newkey rsa:4096 -keyout certs/node.key -out certs/node.csr -nodes -subj "/CN=my-node.vx0"
openssl x509 -req -in certs/node.csr -CA certs/ca.crt -CAkey certs/ca.key -CAcreateserial -out certs/node.crt -days 365

# Set permissions
chmod 600 certs/*.key
chmod 644 certs/*.crt
```

### **Production Certificates**

For production deployment, use proper certificates:

```bash
# Mount your production certificates
volumes:
  - /path/to/production/certs:/app/certs:ro
```

## 🌐 Network Ports

The following ports need to be accessible:

| Port | Protocol | Purpose | Required |
|------|----------|---------|----------|
| 1179 | TCP | BGP Routing | ✅ Yes |
| 4500 | UDP | IKE/IPSec | ✅ Yes |
| 5353 | UDP | DNS Server | ⚠️ Optional |
| 8080 | TCP | Service Discovery | ⚠️ Optional |
| 9090 | TCP | Metrics | ⚠️ Optional |

### **Firewall Configuration**

```bash
# Allow VX0 network ports
sudo ufw allow 1179/tcp comment "VX0 BGP"
sudo ufw allow 4500/udp comment "VX0 IKE"
sudo ufw allow 8080/tcp comment "VX0 Discovery"
sudo ufw allow 9090/tcp comment "VX0 Metrics"
```

## 📊 Monitoring

### **Built-in Metrics**

Each node exposes Prometheus metrics on port 9090:

```bash
# Check node metrics
curl http://localhost:9090/metrics
```

### **Grafana Dashboard**

Start the monitoring stack:

```bash
docker-compose --profile monitoring up -d
```

Access Grafana at http://localhost:3000:
- **Username**: admin
- **Password**: vx0-admin-change-me

### **Health Checks**

```bash
# Check container health
docker-compose ps

# Check node status
docker-compose exec vx0-edge vx0net info

# Check network connectivity
docker-compose exec vx0-edge vx0net network-status
```

## 🔄 Common Operations

### **Starting a Node**

```bash
# Edge node
docker-compose up -d vx0-edge

# Regional node  
docker-compose --profile regional up -d vx0-regional

# With monitoring
docker-compose --profile monitoring up -d vx0-edge prometheus grafana
```

### **Checking Logs**

```bash
# Follow logs
docker-compose logs -f vx0-edge

# Last 100 lines
docker-compose logs --tail=100 vx0-edge

# All containers
docker-compose logs
```

### **Node Management**

```bash
# Enter container shell
docker-compose exec vx0-edge /bin/bash

# Check node info
docker-compose exec vx0-edge vx0net info

# Check peers
docker-compose exec vx0-edge vx0net peers

# Check routes  
docker-compose exec vx0-edge vx0net routes
```

### **Updating the Node**

```bash
# Rebuild and restart
docker-compose build vx0-edge
docker-compose up -d vx0-edge

# Or pull latest image (if using published images)
docker-compose pull vx0-edge
docker-compose up -d vx0-edge
```

## 🐛 Troubleshooting

### **Common Issues**

**Issue**: Container fails to start
```bash
# Check logs for errors
docker-compose logs vx0-edge

# Check configuration syntax
docker-compose config
```

**Issue**: Cannot connect to peers
```bash
# Check network connectivity
docker-compose exec vx0-edge ping backbone1.vx0.network

# Check firewall rules
sudo ufw status

# Check if ports are listening
docker-compose exec vx0-edge netstat -tlnp
```

**Issue**: Certificate errors
```bash
# Verify certificate files exist and have correct permissions
docker-compose exec vx0-edge ls -la /app/certs/

# Check certificate validity
docker-compose exec vx0-edge openssl x509 -in /app/certs/node.crt -text -noout
```

### **Debug Mode**

Enable debug logging:

```bash
# Set environment variable
VX0NET_LOG_LEVEL=debug docker-compose up vx0-edge

# Or edit docker-compose.yml:
environment:
  - VX0NET_LOG_LEVEL=debug
```

### **Network Debugging**

```bash
# Check container networking
docker network ls
docker network inspect vx0net-daemon_vx0-network

# Test connectivity between containers
docker-compose exec vx0-edge ping vx0-regional
```

## 🚢 Production Deployment

### **Docker Swarm (Multi-host)**

```bash
# Initialize swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.yml vx0net
```

### **Kubernetes**

Convert docker-compose to Kubernetes:

```bash
# Install kompose
curl -L https://github.com/kubernetes/kompose/releases/download/v1.28.0/kompose-linux-amd64 -o kompose

# Convert to Kubernetes manifests
kompose convert -f docker-compose.yml
```

### **Environment-specific Configuration**

Create environment-specific compose files:

```bash
# docker-compose.production.yml
version: '3.8'
services:
  vx0-edge:
    environment:
      - VX0NET_LOG_LEVEL=info
    restart: always
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '1.0'
```

Deploy with environment override:

```bash
docker-compose -f docker-compose.yml -f docker-compose.production.yml up -d
```

## 📖 Related Documentation

- [README.md](README.md) - Main project documentation
- [JOINING.md](JOINING.md) - How to join the VX0 network
- [DEPLOYMENT.md](DEPLOYMENT.md) - Traditional deployment methods
- [deploy/](deploy/) - Configuration templates

## ⚡ Quick Commands Reference

```bash
# Start edge node
docker-compose up -d vx0-edge

# View logs
docker-compose logs -f vx0-edge

# Check status
docker-compose exec vx0-edge vx0net info

# Stop node
docker-compose down

# Update and restart
docker-compose build && docker-compose up -d

# Start with monitoring
docker-compose --profile monitoring up -d

# Clean up everything
docker-compose down -v
docker system prune -f
```
