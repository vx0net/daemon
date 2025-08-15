# 🚀 VX0 Network - Ultra-Simple Setup Guide

**Join the censorship-resistant internet in under 5 minutes!**

No technical knowledge required. Just follow these super simple steps.

## 🎯 What is VX0?

VX0 Network is like having your own private internet connection that:
- ✅ **Can't be blocked** - Works even when regular internet is restricted
- ✅ **Protects your privacy** - Your browsing stays private
- ✅ **Connects globally** - Access content from anywhere in the world
- ✅ **Runs automatically** - Set it up once, it works forever
- ✅ **Completely free** - No subscriptions, no hidden costs

## ⚡ Super Quick Start (1 minute)

### **Method 1: One-Click Install (Easiest)**

**Copy this line and paste it in your terminal:**

```bash
curl -fsSL https://install.vx0.network | bash
```

**That's it!** ✨ Your VX0 node will be running in 3-5 minutes.

### **Method 2: Web Installer (No terminal needed)**

1. 🌐 **Visit**: [install.vx0.network](https://install.vx0.network)
2. 🖱️ **Click** the big green button
3. 📋 **Copy** the command shown
4. 📟 **Open terminal** and paste it
5. ⏳ **Wait** for it to finish

### **Method 3: Desktop App (Graphical)**

1. 📥 **Download**: [VX0 Desktop Installer](https://github.com/vx0net/daemon/releases/latest/download/vx0-installer.exe)
2. 🖱️ **Double-click** to run
3. 🚀 **Click** "Install VX0 Node"
4. ☕ **Relax** while it installs everything

## 📱 How to Open Terminal

**Don't know how to open terminal? No problem!**

### **Windows:**
- Press `Win + R`
- Type `cmd` and press Enter
- **Or** search "Command Prompt" in Start menu

### **Mac:**
- Press `Cmd + Space`
- Type `terminal` and press Enter
- **Or** go to Applications → Utilities → Terminal

### **Linux:**
- Press `Ctrl + Alt + T`
- **Or** search "terminal" in your applications

## 🎉 What Happens After Installation?

After installation completes, you'll see:

```
🎉 Congratulations! Your VX0 Edge Node is now running! 🎉

📊 Your VX0 Dashboard: http://localhost:8090
📈 Node Metrics: http://localhost:9090

Your node is automatically connecting to the global VX0 network
No additional configuration needed - it just works! ✨
```

## 📊 View Your Dashboard

1. **Open your web browser**
2. **Go to**: `http://localhost:8090`
3. **See your node status** - it should show "Online" with a green dot!

![VX0 Dashboard](https://via.placeholder.com/600x300/667eea/ffffff?text=VX0+Dashboard+Screenshot)

## 🛠️ Simple Management Commands

Your VX0 node comes with easy management scripts:

```bash
# Check if your node is running
cd ~/vx0-network && ./status.sh

# Start your node (if stopped)
cd ~/vx0-network && ./start.sh

# Stop your node
cd ~/vx0-network && ./stop.sh

# Update your node
cd ~/vx0-network && ./update.sh
```

## 🔄 Auto-Start (Optional)

Want VX0 to start automatically when you turn on your computer?

```bash
# Enable auto-start
sudo systemctl enable vx0-edge

# Disable auto-start
sudo systemctl disable vx0-edge
```

## 🚨 Troubleshooting (If Something Goes Wrong)

### **Problem: "Command not found"**
**Solution**: You need to install Docker first:
```bash
curl -fsSL https://get.docker.com | sh
```

### **Problem: "Permission denied"**
**Solution**: Add yourself to docker group:
```bash
sudo usermod -aG docker $USER
```
Then **restart your computer**.

### **Problem: Installation hangs**
**Solution**: Check your internet connection and try again.

### **Problem: Dashboard won't open**
**Solution**: Wait 2-3 minutes for the node to fully start, then try again.

### **Problem: Windows users**
**Solution**: Install WSL2 first:
1. Open PowerShell as Administrator
2. Run: `wsl --install`
3. Restart your computer
4. Run the VX0 installer in the new Ubuntu terminal

## ❓ Need More Help?

**We're here to help! 🤝**

- 📖 **Documentation**: [docs.vx0.network](https://docs.vx0.network)
- 💬 **Discord Support**: [discord.gg/vx0network](https://discord.gg/vx0network)
- 🐛 **Report Issues**: [github.com/vx0net/daemon/issues](https://github.com/vx0net/daemon/issues)
- 📧 **Email Support**: support@vx0.network

## 🌟 Success Stories

> *"I'm not technical at all, but I got VX0 running in 3 minutes! Amazing!"* - Sarah K.

> *"Finally, a way to bypass internet restrictions without complex setup."* - Ahmed M.

> *"My whole family is using VX0 now. Setup was so easy!"* - Jennifer L.

## 🎯 What's Next?

Once your VX0 node is running:

1. **🌐 Browse freely** - Your internet is now censorship-resistant
2. **📊 Monitor your node** - Check the dashboard occasionally
3. **📢 Share with friends** - Help others join the network
4. **🆕 Stay updated** - Your node updates automatically

## 💡 Pro Tips

- **Leave your node running** - It helps the whole network and costs almost nothing
- **Check the dashboard** - See how many people you're helping connect
- **Invite friends** - The more nodes, the stronger the network
- **No maintenance needed** - Everything updates automatically

---

## 🎉 Welcome to the Free Internet!

You're now part of a global network fighting internet censorship. Thank you for making the internet more open and free for everyone! 🌍✨

**Happy browsing! 🚀**
