#!/bin/bash
# Test that Docker containers can import their required modules
#
# Copyright (c) 2025 Michael A Wright
# MIT License

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo "=========================================="
echo "Testing Open-TTS-RS Container Imports"
echo "=========================================="
echo ""

# Test OpenVoice
log_info "Testing OpenVoice V2 imports..."
if docker run --rm open-tts-rs/openvoice:latest python -c "
from openvoice.api import ToneColorConverter
from melo.api import TTS
print('OpenVoice imports: OK')
" 2>&1; then
    log_info "OpenVoice V2: PASSED"
else
    log_error "OpenVoice V2: FAILED"
    exit 1
fi

echo ""

# Test OpenF5
log_info "Testing OpenF5-TTS imports..."
if docker run --rm open-tts-rs/openf5:latest python -c "
from f5_tts.api import F5TTS
print('OpenF5-TTS imports: OK')
" 2>&1; then
    log_info "OpenF5-TTS: PASSED"
else
    log_error "OpenF5-TTS: FAILED"
    exit 1
fi

echo ""
echo "=========================================="
log_info "All import tests passed!"
echo "=========================================="
