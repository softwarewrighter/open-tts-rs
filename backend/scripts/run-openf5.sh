#!/bin/bash
# Run OpenF5-TTS Docker container
# Port: 9288
#
# Copyright (c) 2025 Michael A Wright
# MIT License

set -euo pipefail

CONTAINER_NAME="openf5-server"
IMAGE_NAME="open-tts-rs/openf5:latest"
PORT=9288

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}[OpenF5-TTS]${NC} Starting on port $PORT..."

# Stop existing container if running
if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
    echo -e "${YELLOW}[OpenF5-TTS]${NC} Stopping existing container..."
    docker stop "$CONTAINER_NAME" >/dev/null 2>&1 || true
fi

# Remove existing container
docker rm "$CONTAINER_NAME" >/dev/null 2>&1 || true

# Create voice storage directory
mkdir -p "$HOME/.open-tts-rs/voices/openf5"

# Run container
docker run -d \
    --name "$CONTAINER_NAME" \
    --gpus all \
    --restart unless-stopped \
    -p "$PORT:9288" \
    -v "$HOME/.open-tts-rs/voices/openf5:/app/voices" \
    -e NVIDIA_VISIBLE_DEVICES=all \
    -e NVIDIA_DRIVER_CAPABILITIES=compute,utility \
    "$IMAGE_NAME"

echo -e "${GREEN}[OpenF5-TTS]${NC} Container started"
echo -e "${GREEN}[OpenF5-TTS]${NC} URL: http://localhost:$PORT"
echo -e "${GREEN}[OpenF5-TTS]${NC} Health: http://localhost:$PORT/health"
