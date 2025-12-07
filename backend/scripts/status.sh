#!/bin/bash
# Check status of TTS Docker containers
#
# Copyright (c) 2025 Michael A Wright
# MIT License

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=========================================="
echo "Open-TTS-RS Backend Status"
echo "=========================================="
echo ""

# Check Docker
echo "Docker Status:"
if docker info >/dev/null 2>&1; then
    echo -e "  Docker: ${GREEN}Running${NC}"
else
    echo -e "  Docker: ${RED}Not Running${NC}"
    exit 1
fi

# Check NVIDIA
echo ""
echo "GPU Status:"
if command -v nvidia-smi &> /dev/null; then
    GPU_NAME=$(nvidia-smi --query-gpu=name --format=csv,noheader 2>/dev/null | head -1)
    GPU_UTIL=$(nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader 2>/dev/null | head -1)
    GPU_MEM=$(nvidia-smi --query-gpu=memory.used,memory.total --format=csv,noheader 2>/dev/null | head -1)
    CUDA_VER=$(nvidia-smi --query-gpu=driver_version --format=csv,noheader 2>/dev/null | head -1)

    echo -e "  GPU: ${GREEN}${GPU_NAME}${NC}"
    echo "  Utilization: ${GPU_UTIL}"
    echo "  Memory: ${GPU_MEM}"
    echo "  Driver: ${CUDA_VER}"
else
    echo -e "  GPU: ${RED}nvidia-smi not found${NC}"
fi

echo ""
echo "Container Status:"

# OpenVoice
if docker ps -q -f name="openvoice-server" | grep -q .; then
    echo -e "  openvoice-server: ${GREEN}Running${NC}"

    # Health check
    if curl -s http://localhost:9280/health >/dev/null 2>&1; then
        HEALTH=$(curl -s http://localhost:9280/health | grep -o '"status":"[^"]*"' | cut -d'"' -f4)
        echo -e "    Health: ${GREEN}${HEALTH}${NC}"
        echo "    URL: http://localhost:9280"
    else
        echo -e "    Health: ${YELLOW}Starting...${NC}"
    fi
else
    echo -e "  openvoice-server: ${RED}Stopped${NC}"
fi

# OpenF5
if docker ps -q -f name="openf5-server" | grep -q .; then
    echo -e "  openf5-server: ${GREEN}Running${NC}"

    # Health check
    if curl -s http://localhost:9288/health >/dev/null 2>&1; then
        HEALTH=$(curl -s http://localhost:9288/health | grep -o '"status":"[^"]*"' | cut -d'"' -f4)
        echo -e "    Health: ${GREEN}${HEALTH}${NC}"
        echo "    URL: http://localhost:9288"
    else
        echo -e "    Health: ${YELLOW}Starting...${NC}"
    fi
else
    echo -e "  openf5-server: ${RED}Stopped${NC}"
fi

echo ""
echo "=========================================="
