---
name: Bug report
about: Create a report to help us improve VX0 Network
title: '[BUG] '
labels: 'bug'
assignees: ''

---

**🐛 Bug Description**
A clear and concise description of what the bug is.

**🔄 Steps to Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**✅ Expected Behavior**
A clear and concise description of what you expected to happen.

**❌ Actual Behavior**
A clear and concise description of what actually happened.

**📸 Screenshots**
If applicable, add screenshots to help explain your problem.

**🖥️ System Information**
- OS: [e.g. Ubuntu 22.04, macOS 13.0, Windows 11]
- Architecture: [e.g. x86_64, arm64]
- Docker version: [e.g. 24.0.6]
- VX0 version: [e.g. v1.0.0]
- Installation method: [e.g. one-command, manual, Docker]

**📋 VX0 Node Information**
- Node type: [e.g. Edge, Regional, Backbone]
- ASN: [if known]
- Connected peers: [number]
- Dashboard accessible: [yes/no]

**📝 Logs**
Please include relevant logs:

```
# VX0 node logs
docker-compose logs vx0-edge

# System logs (if relevant)
journalctl -u docker

# Auto-update logs (if relevant)
cat ~/vx0-network/logs/auto-update.log
```

**🔧 Configuration**
If relevant, include your configuration (remove any sensitive information):

```toml
# Your vx0net.toml (remove private keys, IPs, etc.)
```

**🌐 Network Environment**
- Behind firewall: [yes/no]
- NAT/Router: [yes/no]
- VPN active: [yes/no]
- ISP restrictions: [if known]

**📦 Additional Context**
Add any other context about the problem here.

**🤝 Are you willing to help fix this?**
- [ ] Yes, I can provide more information if needed
- [ ] Yes, I can test potential fixes
- [ ] Yes, I might be able to submit a fix
- [ ] No, I'm just reporting the issue
