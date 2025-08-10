#!/bin/bash

# VPS Deployment Setup Script
# Run this script on your VPS to set up the deployment environment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   print_error "This script should not be run as root"
   exit 1
fi

print_status "Setting up VPS for Axum application deployment..."

# Update system
print_status "Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install Docker
if ! command -v docker &> /dev/null; then
    print_status "Installing Docker..."
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    sudo usermod -aG docker $USER
    rm get-docker.sh
else
    print_status "Docker already installed"
fi

# Install Git
if ! command -v git &> /dev/null; then
    print_status "Installing Git..."
    sudo apt install -y git
else
    print_status "Git already installed"
fi

# Install curl for health checks
if ! command -v curl &> /dev/null; then
    print_status "Installing curl..."
    sudo apt install -y curl
else
    print_status "curl already installed"
fi

# Clone repository (you'll need to update this URL)
cd $APP_DIR
if [ ! -d ".git" ]; then
    print_status "Cloning repository..."
    # Replace with your actual repository URL
    read -p "Enter your Git repository URL: " REPO_URL
    git clone $REPO_URL .
else
    print_status "Repository already cloned"
fi

# Create systemd service for auto-restart
SERVICE_FILE="/etc/systemd/system/axum-app.service"
if [ ! -f "$SERVICE_FILE" ]; then
    print_status "Creating systemd service..."
    sudo tee $SERVICE_FILE > /dev/null <<EOF
[Unit]
Description=Axum Application
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=$APP_DIR
ExecStart=/usr/local/bin/docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
ExecStop=/usr/local/bin/docker-compose -f docker-compose.yml -f docker-compose.prod.yml down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    sudo systemctl enable axum-app.service
fi

print_status "VPS setup completed!"
print_status ""
print_status "Next steps:"
print_status "1. Edit .env file: nano $APP_DIR/.env"
print_status "2. Configure GitHub Secrets for CI/CD:"
print_status "   - VPS_HOST: Your VPS IP address"
print_status "   - VPS_USERNAME: Your VPS username"
print_status "   - VPS_SSH_KEY: Your private SSH key"
print_status "   - VPS_PORT: SSH port (default: 22)"
print_status "3. Test deployment: sudo systemctl start axum-app"
print_status "4. Check status: sudo systemctl status axum-app"
print_status "5. View logs: docker-compose logs -f"
print_status ""
print_warning "Remember to:"
print_warning "- Configure your domain DNS to point to this VPS"
print_warning "- Set up SSL certificates if using HTTPS"
print_warning "- Configure your GitHub repository secrets"
print_warning "- Test the deployment manually before relying on CI/CD"
