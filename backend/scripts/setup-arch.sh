#!/bin/bash
# Setup script for Arch Linux host (curiosity)
# Installs Docker, NVIDIA Container Toolkit for Blackwell GPU support
#
# Copyright (c) 2025 Michael A Wright
# MIT License

set -euo pipefail

echo "=========================================="
echo "Open-TTS-RS Backend Setup for Arch Linux"
echo "Target GPU: NVIDIA RTX 5060 (Blackwell)"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    log_error "Do not run this script as root. It will use sudo when needed."
    exit 1
fi

# Check for Arch Linux
if [[ ! -f /etc/arch-release ]]; then
    log_error "This script is designed for Arch Linux"
    exit 1
fi

log_info "Updating system packages..."
sudo pacman -Syu --noconfirm

# Install Docker
log_info "Installing Docker..."
sudo pacman -S --noconfirm --needed docker docker-compose

# Enable and start Docker service
log_info "Enabling Docker service..."
sudo systemctl enable docker
sudo systemctl start docker

# Add current user to docker group
log_info "Adding user to docker group..."
sudo usermod -aG docker "$USER"

# Install NVIDIA drivers (if not already installed)
log_info "Checking NVIDIA drivers..."
if ! command -v nvidia-smi &> /dev/null; then
    log_warn "NVIDIA drivers not found. Installing..."
    sudo pacman -S --noconfirm --needed nvidia nvidia-utils nvidia-settings

    log_warn "NVIDIA drivers installed. A reboot may be required."
fi

# Install NVIDIA Container Toolkit
log_info "Installing NVIDIA Container Toolkit..."

# Add NVIDIA Container Toolkit repository
if [[ ! -f /etc/pacman.d/nvidia-container-toolkit.conf ]]; then
    log_info "Adding NVIDIA Container Toolkit repository..."

    # Install from AUR or use nvidia-container-toolkit package
    if command -v yay &> /dev/null; then
        yay -S --noconfirm nvidia-container-toolkit
    elif command -v paru &> /dev/null; then
        paru -S --noconfirm nvidia-container-toolkit
    else
        log_warn "No AUR helper found. Installing nvidia-container-toolkit manually..."

        # Install dependencies
        sudo pacman -S --noconfirm --needed base-devel git

        # Clone and build nvidia-container-toolkit
        TEMP_DIR=$(mktemp -d)
        cd "$TEMP_DIR"
        git clone https://aur.archlinux.org/nvidia-container-toolkit.git
        cd nvidia-container-toolkit
        makepkg -si --noconfirm
        cd /
        rm -rf "$TEMP_DIR"
    fi
fi

# Configure Docker to use NVIDIA runtime
log_info "Configuring Docker for NVIDIA runtime..."
sudo nvidia-ctk runtime configure --runtime=docker
sudo systemctl restart docker

# Verify installation
log_info "Verifying NVIDIA Docker support..."
echo ""

# Check NVIDIA driver
log_info "NVIDIA Driver Info:"
nvidia-smi --query-gpu=name,driver_version,cuda_version --format=csv

echo ""

# Test NVIDIA in Docker
log_info "Testing NVIDIA in Docker..."
if docker run --rm --gpus all nvidia/cuda:12.8.0-base-ubuntu24.04 nvidia-smi &> /dev/null; then
    log_info "NVIDIA Docker support is working!"
    docker run --rm --gpus all nvidia/cuda:12.8.0-base-ubuntu24.04 nvidia-smi
else
    log_warn "NVIDIA Docker test failed. You may need to:"
    log_warn "  1. Reboot the system"
    log_warn "  2. Log out and back in (for docker group)"
    log_warn "  3. Check NVIDIA driver installation"
fi

echo ""
echo "=========================================="
echo "Setup Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  1. Log out and back in (for docker group access)"
echo "  2. Run: ./build-all.sh"
echo "  3. Run: ./run-all.sh"
echo ""
echo "Ports:"
echo "  - OpenVoice V2:  http://localhost:9280"
echo "  - OpenF5-TTS:    http://localhost:9288"
echo ""
