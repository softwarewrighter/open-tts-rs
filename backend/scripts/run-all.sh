#!/bin/bash
# Run all TTS Docker containers
# Ports: OpenVoice=9280, OpenF5-TTS=9288
#
# Copyright (c) 2025 Michael A Wright
# MIT License

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================="
echo "Starting Open-TTS-RS Backend Services"
echo "=========================================="
echo ""

# Run both containers
"$SCRIPT_DIR/run-openvoice.sh" &
"$SCRIPT_DIR/run-openf5.sh" &

# Wait a moment for containers to start
sleep 5

echo ""
echo "=========================================="
echo "Services Starting..."
echo "=========================================="
echo ""
echo "OpenVoice V2:  http://localhost:9280"
echo "OpenF5-TTS:    http://localhost:9288"
echo ""
echo "Health endpoints:"
echo "  curl http://localhost:9280/health"
echo "  curl http://localhost:9288/health"
echo ""
echo "Check status with:"
echo "  docker ps | grep open-tts-rs"
echo ""
echo "View logs:"
echo "  docker logs -f openvoice-server"
echo "  docker logs -f openf5-server"
echo ""
echo "Stop services:"
echo "  ./stop-all.sh"
echo ""

# Wait for background processes
wait
