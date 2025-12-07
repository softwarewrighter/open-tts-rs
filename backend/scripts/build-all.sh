#!/bin/bash
# Build all TTS Docker containers
# Target: NVIDIA RTX 5060 (Blackwell) with CUDA 12.8+
#
# Copyright (c) 2025 Michael A Wright
# MIT License

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(dirname "$SCRIPT_DIR")"
DOCKER_DIR="$BACKEND_DIR/docker"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

echo "=========================================="
echo "Building Open-TTS-RS Docker Containers"
echo "Target: NVIDIA RTX 5060 (Blackwell)"
echo "CUDA: 12.8+"
echo "=========================================="
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    log_error "Docker is not installed. Run setup-arch.sh first."
    exit 1
fi

# Check NVIDIA support
if ! docker info 2>/dev/null | grep -q "Runtimes.*nvidia"; then
    log_warn "NVIDIA Docker runtime not configured."
    log_warn "Run setup-arch.sh first or configure manually."
fi

# Build OpenVoice container
log_step "Building OpenVoice V2 container..."
echo ""

docker build \
    --tag open-tts-rs/openvoice:latest \
    --tag open-tts-rs/openvoice:cuda12.8 \
    --file "$DOCKER_DIR/openvoice/Dockerfile" \
    "$DOCKER_DIR/openvoice"

log_info "OpenVoice V2 container built successfully"
echo ""

# Build OpenF5-TTS container
log_step "Building OpenF5-TTS container..."
echo ""

docker build \
    --tag open-tts-rs/openf5:latest \
    --tag open-tts-rs/openf5:cuda12.8 \
    --file "$DOCKER_DIR/openf5/Dockerfile" \
    "$DOCKER_DIR/openf5"

log_info "OpenF5-TTS container built successfully"
echo ""

# List built images
echo "=========================================="
echo "Built Images:"
echo "=========================================="
docker images | grep "open-tts-rs"
echo ""

log_info "Build complete!"
echo ""
echo "To run the containers:"
echo "  ./run-all.sh"
echo ""
echo "Or run individually:"
echo "  ./run-openvoice.sh"
echo "  ./run-openf5.sh"
echo ""
