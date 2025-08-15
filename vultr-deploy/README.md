# 🚀 VX0 Network - Vultr Deployment

Deploy VX0 Network Backbone and Regional nodes across Vultr's global infrastructure with a single command.

## ⚡ Quick Start

### 1. Get Vultr API Key
1. Sign up at [Vultr](https://www.vultr.com/)
2. Go to [API Settings](https://my.vultr.com/settings/#settingsapi)
3. Generate a new API key

### 2. Set Environment Variable
```bash
export VULTR_API_KEY="your_api_key_here"
```

### 3. Deploy Your Network
```bash
# Deploy core Backbone nodes (3 regions)
./deploy-vultr.sh deploy-backbone

# Deploy Regional nodes (3 regions)  
./deploy-vultr.sh deploy-regional

# Deploy everything at once
./deploy-vultr.sh deploy-all
```

## 🌍 Global Network Architecture

### Backbone Nodes (ASN 65001-65006)
Core network infrastructure providing global connectivity:

- **🇺🇸 us-east** (New York) - ASN 65001
- **🇺🇸 us-west** (Los Angeles) - ASN 65002  
- **🇬🇧 eu-west** (London) - ASN 65003
- **🇩🇪 eu-central** (Frankfurt) - ASN 65004
- **🇸🇬 asia-pacific** (Singapore) - ASN 65005
- **🇯🇵 asia-east** (Tokyo) - ASN 65006

### Regional Nodes (ASN 65101-65106)
Secondary infrastructure for regional connectivity:

- **🇺🇸 us-central** (Chicago) - ASN 65101
- **🇺🇸 us-south** (Dallas) - ASN 65102
- **🇸🇪 eu-north** (Stockholm) - ASN 65103
- **🇮🇳 asia-south** (Mumbai) - ASN 65104
- **🇦🇺 oceania** (Sydney) - ASN 65105
- **🇨🇦 canada** (Toronto) - ASN 65106

## 💰 Cost Breakdown

| Plan | Specs | Monthly Cost | Total (12 nodes) |
|------|-------|--------------|-------------------|
| vc2-1c-512m | 1 vCPU, 512MB | $2.50 | **$30/month** |
| vc2-1c-1gb | 1 vCPU, 1GB | $6.00 | **$72/month** |
| vc2-2c-2gb | 2 vCPU, 2GB | $12.00 | **$144/month** |

*Default: vc2-1c-1gb for reliable performance*

## 🛠️ Commands

### Deployment Commands
```bash
# Deploy specific Backbone locations
./deploy-vultr.sh deploy-backbone us-east eu-west

# Deploy specific Regional locations  
./deploy-vultr.sh deploy-regional us-central asia-south

# Deploy all nodes
./deploy-vultr.sh deploy-all
```

### Management Commands
```bash
# List all VX0 instances
./deploy-vultr.sh list

# Delete instances by pattern
./deploy-vultr.sh delete vx0-backbone
./deploy-vultr.sh delete vx0-regional
./deploy-vultr.sh delete vx0  # Delete all VX0 instances

# Show available regions and plans
./deploy-vultr.sh regions
./deploy-vultr.sh plans
```

## 📊 Monitoring & Management

### After Deployment
Each node provides:

- **📈 Metrics Dashboard**: `http://NODE_IP:9090`
- **🔍 Health Check**: `http://NODE_IP:9090/health`
- **📋 Status Script**: `ssh root@NODE_IP '/opt/vx0-network/status.sh'`
- **🔄 Update Script**: `ssh root@NODE_IP '/opt/vx0-network/update.sh'`

### Network Status
```bash
# Check all nodes at once
./deploy-vultr.sh list

# Individual node status
ssh root@NODE_IP '/opt/vx0-network/status.sh'
```

## 🔧 Configuration

### Custom Instance Specs
Edit `vultr-config.env`:
```bash
VULTR_PLAN="vc2-2c-2gb"        # Upgrade to 2 vCPU, 2GB RAM
VULTR_ENABLE_IPV6="true"       # Enable IPv6
```

### Custom Locations
```bash
# Deploy only US nodes
./deploy-vultr.sh deploy-backbone us-east us-west
./deploy-vultr.sh deploy-regional us-central us-south

# Deploy only EU nodes  
./deploy-vultr.sh deploy-backbone eu-west eu-central
./deploy-vultr.sh deploy-regional eu-north
```

## 🏗️ What Gets Deployed

Each VPS instance includes:

### ✅ Automated Setup
- **Ubuntu 22.04 LTS** with latest updates
- **Docker & Docker Compose** installed
- **VX0 Network daemon** from GitHub Container Registry
- **Security certificates** auto-generated
- **Systemd service** for auto-start
- **Firewall** properly configured

### 🔧 Node Configuration
- **BGP routing** on port 1179
- **IKE/IPSec security** on port 4500  
- **DNS resolution** on port 5353
- **Service discovery** on port 8080
- **Metrics & monitoring** on port 9090

### 🛡️ Security Features
- **TLS certificates** for encrypted communication
- **Firewall rules** for essential ports only
- **Non-root user** for service execution
- **Auto-discovery** for network formation

## 🚨 Troubleshooting

### Common Issues

**Authentication Error**
```bash
# Verify API key
echo $VULTR_API_KEY

# Test API access
curl -H "Authorization: Bearer $VULTR_API_KEY" https://api.vultr.com/v2/account
```

**Instance Creation Failed**
```bash
# Check available regions
./deploy-vultr.sh regions

# Check available plans
./deploy-vultr.sh plans

# Verify account limits
curl -H "Authorization: Bearer $VULTR_API_KEY" https://api.vultr.com/v2/account
```

**Node Not Starting**
```bash
# Check instance status
./deploy-vultr.sh list

# SSH to instance and check logs
ssh root@NODE_IP
docker-compose -f /opt/vx0-network/docker-compose.yml logs
```

### Support
- **GitHub Issues**: [VX0 Network Issues](https://github.com/vx0net/daemon/issues)
- **Discord Community**: [Join VX0 Discord](https://discord.gg/vx0network)
- **Documentation**: [VX0 Network Docs](https://docs.vx0.network)

## 🔄 Updates & Maintenance

### Automatic Updates
Nodes automatically pull the latest VX0 software on restart:

```bash
# Update all nodes
for ip in $(./deploy-vultr.sh list | grep -oE '[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+'); do
    ssh root@$ip '/opt/vx0-network/update.sh'
done
```

### Manual Maintenance
```bash
# SSH to any node
ssh root@NODE_IP

# Check status
/opt/vx0-network/status.sh

# View logs
docker-compose -f /opt/vx0-network/docker-compose.yml logs -f

# Restart node
systemctl restart vx0-node
```

## 🌟 Next Steps

After deployment:

1. **Verify Network**: Check that nodes are discovering each other
2. **Monitor Health**: Set up monitoring alerts for your infrastructure
3. **Scale Up**: Add more Regional nodes in additional locations
4. **Connect Edge Nodes**: Deploy Edge nodes for end-users
5. **Custom Applications**: Build applications on top of VX0 Network

---

**🎉 Congratulations!** You now have a global, decentralized network infrastructure running across Vultr's worldwide data centers!
