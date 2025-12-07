#!/bin/bash
# Run OpenVoice V2 Docker container
# Port: 9280
#
# Copyright (c) 2025 Michael A Wright
# MIT License

set -euo pipefail

CONTAINER_NAME="openvoice-server"
IMAGE_NAME="open-tts-rs/openvoice:latest"
PORT=9280

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}[OpenVoice V2]${NC} Starting on port $PORT..."

# Stop existing container if running
if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
    echo -e "${YELLOW}[OpenVoice V2]${NC} Stopping existing container..."
    docker stop "$CONTAINER_NAME" >/dev/null 2>&1 || true
fi

# Remove existing container
docker rm "$CONTAINER_NAME" >/dev/null 2>&1 || true

# Create voice storage directory
mkdir -p "$HOME/.open-tts-rs/voices/openvoice"

# Run container
docker run -d \
    --name "$CONTAINER_NAME" \
    --gpus all \
    --restart unless-stopped \
    -p "$PORT:9280" \
    -v "$HOME/.open-tts-rs/voices/openvoice:/app/voices" \
    -e NVIDIA_VISIBLE_DEVICES=all \
    -e NVIDIA_DRIVER_CAPABILITIES=compute,utility \
    "$IMAGE_NAME"

echo -e "${GREEN}[OpenVoice V2]${NC} Container started"
echo -e "${GREEN}[OpenVoice V2]${NC} URL: http://localhost:$PORT"
echo -e "${GREEN}[OpenVoice V2]${NC} Health: http://localhost:$PORT/health"
