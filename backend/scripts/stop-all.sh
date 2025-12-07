#!/bin/bash
# Stop all TTS Docker containers
#
# Copyright (c) 2025 Michael A Wright
# MIT License

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo "Stopping Open-TTS-RS Backend Services..."

# Stop OpenVoice
if docker ps -q -f name="openvoice-server" | grep -q .; then
    echo -e "${RED}[STOP]${NC} openvoice-server"
    docker stop openvoice-server
fi

# Stop OpenF5
if docker ps -q -f name="openf5-server" | grep -q .; then
    echo -e "${RED}[STOP]${NC} openf5-server"
    docker stop openf5-server
fi

echo ""
echo -e "${GREEN}All services stopped.${NC}"
