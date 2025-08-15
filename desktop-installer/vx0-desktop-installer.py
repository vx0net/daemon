#!/usr/bin/env python3
"""
VX0 Network Desktop Installer
Ultra-simple GUI installer for non-technical users
"""

import tkinter as tk
from tkinter import ttk, messagebox, scrolledtext
import subprocess
import threading
import os
import sys
import platform
import urllib.request
import json
import time
import webbrowser

class VX0Installer:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("VX0 Network - Easy Setup")
        self.root.geometry("800x600")
        self.root.configure(bg='#2c3e50')
        
        # Style configuration
        self.style = ttk.Style()
        self.style.theme_use('clam')
        self.style.configure('Title.TLabel', font=('Arial', 24, 'bold'), background='#2c3e50', foreground='white')
        self.style.configure('Header.TLabel', font=('Arial', 14, 'bold'), background='#2c3e50', foreground='white')
        self.style.configure('Info.TLabel', font=('Arial', 10), background='#2c3e50', foreground='#ecf0f1')
        self.style.configure('Success.TLabel', font=('Arial', 12, 'bold'), background='#2c3e50', foreground='#27ae60')
        self.style.configure('Error.TLabel', font=('Arial', 12, 'bold'), background='#2c3e50', foreground='#e74c3c')
        
        # Installation state
        self.installation_running = False
        self.installation_success = False
        
        self.create_widgets()
        self.detect_system()
        
    def create_widgets(self):
        # Main frame
        main_frame = tk.Frame(self.root, bg='#2c3e50')
        main_frame.pack(fill='both', expand=True, padx=20, pady=20)
        
        # Title
        title_label = ttk.Label(main_frame, text="🌐 VX0 Network", style='Title.TLabel')
        title_label.pack(pady=(0, 10))
        
        subtitle_label = ttk.Label(main_frame, text="Join the censorship-resistant internet", style='Header.TLabel')
        subtitle_label.pack(pady=(0, 20))
        
        # System info frame
        info_frame = tk.Frame(main_frame, bg='#34495e', relief='raised', bd=2)
        info_frame.pack(fill='x', pady=(0, 20))
        
        ttk.Label(info_frame, text="System Information:", style='Header.TLabel').pack(anchor='w', padx=10, pady=(10, 5))
        
        self.system_info = ttk.Label(info_frame, text="Detecting...", style='Info.TLabel')
        self.system_info.pack(anchor='w', padx=20, pady=(0, 10))
        
        # Features frame
        features_frame = tk.Frame(main_frame, bg='#34495e', relief='raised', bd=2)
        features_frame.pack(fill='x', pady=(0, 20))
        
        ttk.Label(features_frame, text="What you get:", style='Header.TLabel').pack(anchor='w', padx=10, pady=(10, 5))
        
        features = [
            "🔒 Privacy-first internet access",
            "🌍 Connect to global decentralized network", 
            "🚫 Bypass censorship and restrictions",
            "📊 Beautiful web dashboard",
            "🔄 Automatic updates and maintenance",
            "💰 Completely free - no subscriptions"
        ]
        
        for feature in features:
            ttk.Label(features_frame, text=feature, style='Info.TLabel').pack(anchor='w', padx=20)
        
        ttk.Label(features_frame, text="", style='Info.TLabel').pack(pady=5)  # Spacer
        
        # Installation frame
        self.install_frame = tk.Frame(main_frame, bg='#2c3e50')
        self.install_frame.pack(fill='both', expand=True)
        
        # Install button
        self.install_button = tk.Button(
            self.install_frame,
            text="🚀 Install VX0 Node",
            command=self.start_installation,
            font=('Arial', 14, 'bold'),
            bg='#27ae60',
            fg='white',
            relief='flat',
            padx=30,
            pady=15,
            cursor='hand2'
        )
        self.install_button.pack(pady=20)
        
        # Progress bar
        self.progress = ttk.Progressbar(self.install_frame, mode='indeterminate')
        
        # Status label
        self.status_label = ttk.Label(self.install_frame, text="Ready to install", style='Info.TLabel')
        self.status_label.pack(pady=10)
        
        # Log area
        log_label = ttk.Label(self.install_frame, text="Installation Log:", style='Header.TLabel')
        log_label.pack(anchor='w', pady=(20, 5))
        
        self.log_text = scrolledtext.ScrolledText(
            self.install_frame,
            height=10,
            bg='#1a1a1a',
            fg='#00ff00',
            font=('Courier', 9),
            wrap=tk.WORD
        )
        self.log_text.pack(fill='both', expand=True, pady=(0, 10))
        
        # Bottom buttons frame
        self.buttons_frame = tk.Frame(self.install_frame, bg='#2c3e50')
        self.buttons_frame.pack(fill='x', pady=10)
        
        # Help button
        help_button = tk.Button(
            self.buttons_frame,
            text="❓ Need Help?",
            command=self.show_help,
            font=('Arial', 10),
            bg='#3498db',
            fg='white',
            relief='flat',
            padx=15,
            pady=8,
            cursor='hand2'
        )
        help_button.pack(side='left')
        
        # Dashboard button (initially hidden)
        self.dashboard_button = tk.Button(
            self.buttons_frame,
            text="📊 Open Dashboard",
            command=self.open_dashboard,
            font=('Arial', 10),
            bg='#9b59b6',
            fg='white',
            relief='flat',
            padx=15,
            pady=8,
            cursor='hand2'
        )
        
    def detect_system(self):
        """Detect system information"""
        system = platform.system()
        architecture = platform.machine()
        python_version = platform.python_version()
        
        # Check for Docker
        docker_available = self.check_docker()
        
        info_text = f"""Operating System: {system} {architecture}
Python Version: {python_version}
Docker: {'✅ Available' if docker_available else '❌ Not installed (will be installed automatically)'}
Installation Method: {'Docker (recommended)' if docker_available else 'Automatic Docker installation'}"""
        
        self.system_info.config(text=info_text)
        
    def check_docker(self):
        """Check if Docker is available"""
        try:
            subprocess.run(['docker', '--version'], 
                         capture_output=True, check=True, timeout=5)
            return True
        except (subprocess.CalledProcessError, FileNotFoundError, subprocess.TimeoutExpired):
            return False
            
    def log(self, message):
        """Add message to log"""
        timestamp = time.strftime("%H:%M:%S")
        self.log_text.insert(tk.END, f"[{timestamp}] {message}\n")
        self.log_text.see(tk.END)
        self.root.update_idletasks()
        
    def update_status(self, message):
        """Update status label"""
        self.status_label.config(text=message)
        self.root.update_idletasks()
        
    def start_installation(self):
        """Start the installation process"""
        if self.installation_running:
            return
            
        self.installation_running = True
        self.install_button.config(state='disabled', text="Installing...")
        self.progress.pack(pady=10)
        self.progress.start()
        
        # Start installation in separate thread
        threading.Thread(target=self.run_installation, daemon=True).start()
        
    def run_installation(self):
        """Run the actual installation"""
        try:
            self.log("🚀 Starting VX0 Network installation...")
            self.update_status("Installing...")
            
            # Step 1: Check and install Docker
            if not self.check_docker():
                self.log("📦 Docker not found, installing...")
                self.update_status("Installing Docker...")
                self.install_docker()
            else:
                self.log("✅ Docker already installed")
                
            # Step 2: Download installer
            self.log("📥 Downloading VX0 installer...")
            self.update_status("Downloading VX0 installer...")
            self.download_installer()
            
            # Step 3: Run installer
            self.log("⚙️ Running VX0 installer...")
            self.update_status("Setting up VX0 node...")
            self.run_vx0_installer()
            
            # Step 4: Verify installation
            self.log("🔍 Verifying installation...")
            self.update_status("Verifying installation...")
            time.sleep(2)  # Give services time to start
            
            if self.verify_installation():
                self.log("🎉 Installation completed successfully!")
                self.installation_success = True
                self.update_status("✅ Installation complete!")
                self.show_success()
            else:
                raise Exception("Installation verification failed")
                
        except Exception as e:
            self.log(f"❌ Installation failed: {str(e)}")
            self.update_status("❌ Installation failed")
            self.show_error(str(e))
        finally:
            self.progress.stop()
            self.progress.pack_forget()
            self.installation_running = False
            self.install_button.config(state='normal', text="🚀 Install VX0 Node")
            
    def install_docker(self):
        """Install Docker"""
        system = platform.system().lower()
        
        if system == "linux":
            # Install Docker on Linux
            commands = [
                "curl -fsSL https://get.docker.com | sh",
                "sudo usermod -aG docker $USER"
            ]
            for cmd in commands:
                self.log(f"Running: {cmd}")
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
                if result.returncode != 0:
                    raise Exception(f"Docker installation failed: {result.stderr}")
                    
        elif system == "darwin":  # macOS
            self.log("Please install Docker Desktop from https://www.docker.com/products/docker-desktop")
            webbrowser.open("https://www.docker.com/products/docker-desktop")
            raise Exception("Please install Docker Desktop and run this installer again")
            
        elif system == "windows":
            self.log("Please install Docker Desktop from https://www.docker.com/products/docker-desktop")
            webbrowser.open("https://www.docker.com/products/docker-desktop")
            raise Exception("Please install Docker Desktop and run this installer again")
            
        else:
            raise Exception(f"Unsupported operating system: {system}")
            
    def download_installer(self):
        """Download VX0 installer"""
        installer_url = "https://raw.githubusercontent.com/vx0net/daemon/main/install-vx0.sh"
        installer_path = os.path.expanduser("~/vx0-installer.sh")
        
        try:
            urllib.request.urlretrieve(installer_url, installer_path)
            os.chmod(installer_path, 0o755)
            self.log(f"Downloaded installer to {installer_path}")
        except Exception as e:
            raise Exception(f"Failed to download installer: {e}")
            
    def run_vx0_installer(self):
        """Run the VX0 installer"""
        installer_path = os.path.expanduser("~/vx0-installer.sh")
        
        # Run installer with environment variable to skip interactive prompts
        env = os.environ.copy()
        env['VX0_NONINTERACTIVE'] = '1'
        
        process = subprocess.Popen(
            ['/bin/bash', installer_path],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
            universal_newlines=True
        )
        
        # Stream output to log
        for line in iter(process.stdout.readline, ''):
            if line:
                self.log(line.strip())
                
        process.wait()
        
        if process.returncode != 0:
            raise Exception("VX0 installer failed")
            
    def verify_installation(self):
        """Verify that VX0 is running"""
        try:
            # Check if containers are running
            result = subprocess.run(
                ['docker', 'ps', '--filter', 'name=vx0', '--format', '{{.Names}}'],
                capture_output=True, text=True, timeout=10
            )
            
            if 'vx0' in result.stdout:
                self.log("✅ VX0 containers are running")
                return True
            else:
                self.log("❌ VX0 containers not found")
                return False
                
        except Exception as e:
            self.log(f"❌ Verification failed: {e}")
            return False
            
    def show_success(self):
        """Show success message and enable dashboard button"""
        self.status_label.config(text="🎉 VX0 Node is running!", style='Success.TLabel')
        self.dashboard_button.pack(side='right')
        
        # Show success dialog
        messagebox.showinfo(
            "Installation Complete!", 
            "🎉 Congratulations!\n\n"
            "Your VX0 Edge Node is now running and connected to the global network.\n\n"
            "Click 'Open Dashboard' to monitor your node or visit:\n"
            "http://localhost:8090"
        )
        
    def show_error(self, error_message):
        """Show error message"""
        self.status_label.config(text="❌ Installation failed", style='Error.TLabel')
        messagebox.showerror(
            "Installation Failed",
            f"Sorry, the installation failed:\n\n{error_message}\n\n"
            "Please check the log for details or click 'Need Help?' for support."
        )
        
    def open_dashboard(self):
        """Open the VX0 dashboard"""
        webbrowser.open("http://localhost:8090")
        
    def show_help(self):
        """Show help dialog"""
        help_text = """🆘 Need Help?

If you're having issues:

1. 📖 Documentation: https://docs.vx0.network
2. 💬 Discord Support: https://discord.gg/vx0network  
3. 🐛 Report Bug: https://github.com/vx0net/daemon/issues

Common Issues:
• Docker permission denied: Restart your computer after installation
• Installer hangs: Check your internet connection
• Windows users: Use WSL2 or Docker Desktop

The VX0 community is here to help! 🤝"""
        
        messagebox.showinfo("Help & Support", help_text)
        
    def run(self):
        """Start the application"""
        self.root.mainloop()

def main():
    """Main entry point"""
    # Check if running as root (not recommended)
    if os.geteuid() == 0 if hasattr(os, 'geteuid') else False:
        if not messagebox.askyesno(
            "Running as Root", 
            "Running as root is not recommended for security reasons.\n\n"
            "Continue anyway?"
        ):
            sys.exit(1)
            
    # Create and run installer
    installer = VX0Installer()
    installer.run()

if __name__ == "__main__":
    main()
