# Open-TTS-RS Backend

Docker containers for running TTS models on NVIDIA GPUs.

## Hardware Requirements

**CRITICAL: Blackwell GPU Support**

This backend is designed for the **NVIDIA RTX 5060 (Blackwell architecture)** which requires:

- **CUDA 12.8+** - Blackwell GPUs (sm_120) are NOT supported by older CUDA versions
- **PyTorch Nightly** - Stable PyTorch releases do not yet support Blackwell
- **Latest NVIDIA drivers** (560+)

The Dockerfiles use:
- Base image: `nvidia/cuda:12.8.0-cudnn-devel-ubuntu24.04`
- PyTorch: Nightly builds with CUDA 12.8 (`cu128`)
- Torch CUDA Arch: `TORCH_CUDA_ARCH_LIST="12.0"`

## Services

| Service | Port | Model | License |
|---------|------|-------|---------|
| OpenVoice V2 | 9280 | Voice cloning + MeloTTS | MIT |
| OpenF5-TTS | 9288 | Flow-matching TTS | Apache 2.0 |

## Quick Start (Arch Linux)

```bash
# 1. Copy to target host
scp -r backend/ curiosity:~/open-tts-rs/

# 2. On curiosity: Setup Docker and NVIDIA runtime
cd ~/open-tts-rs/backend/scripts
chmod +x *.sh
./setup-arch.sh

# 3. Log out and back in (for docker group)

# 4. Build containers (takes 15-30 minutes)
./build-all.sh

# 5. Run services
./run-all.sh

# 6. Check status
./status.sh
```

## Scripts

| Script | Description |
|--------|-------------|
| `setup-arch.sh` | Install Docker + NVIDIA Container Toolkit on Arch Linux |
| `build-all.sh` | Build both Docker containers |
| `run-all.sh` | Start both services |
| `run-openvoice.sh` | Start OpenVoice V2 only |
| `run-openf5.sh` | Start OpenF5-TTS only |
| `stop-all.sh` | Stop all services |
| `status.sh` | Check service and GPU status |

## API Endpoints

### OpenVoice V2 (Port 9280)

```bash
# Health check
curl http://localhost:9280/health

# Model info
curl http://localhost:9280/info

# Extract voice from reference audio
curl -X POST http://localhost:9280/extract_voice \
  -F "audio=@reference.wav" \
  -F "transcript=Hello, this is my voice." \
  -F "name=my_voice"

# Synthesize speech
curl -X POST http://localhost:9280/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello world", "name": "my_voice"}' \
  --output output.wav

# List saved voices
curl http://localhost:9280/voices

# Delete a voice
curl -X DELETE http://localhost:9280/voices/my_voice
```

### OpenF5-TTS (Port 9288)

```bash
# Health check
curl http://localhost:9288/health

# Model info
curl http://localhost:9288/info

# Extract/store voice reference
curl -X POST http://localhost:9288/extract_voice \
  -F "audio=@reference.wav" \
  -F "transcript=Hello, this is my voice." \
  -F "name=my_voice"

# Synthesize speech
curl -X POST http://localhost:9288/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello world", "name": "my_voice"}' \
  --output output.wav

# List saved voices
curl http://localhost:9288/voices

# Delete a voice
curl -X DELETE http://localhost:9288/voices/my_voice
```

## Voice Storage

Voices are persisted in:
- OpenVoice: `~/.open-tts-rs/voices/openvoice/`
- OpenF5-TTS: `~/.open-tts-rs/voices/openf5/`

## Troubleshooting

### CUDA Version Errors

If you see errors about unsupported GPU architecture:

```
CUDA error: no kernel image is available for execution on the device
```

This means PyTorch was not built for Blackwell. Ensure:
1. Using CUDA 12.8+ base image
2. Using PyTorch nightly builds
3. `TORCH_CUDA_ARCH_LIST="12.0"` is set

### Container Won't Start

```bash
# Check container logs
docker logs openvoice-server
docker logs openf5-server

# Check GPU access
docker run --rm --gpus all nvidia/cuda:12.8.0-base-ubuntu24.04 nvidia-smi
```

### Out of Memory

The RTX 5060 has 16GB VRAM. If running both models simultaneously causes OOM:

1. Run only one model at a time
2. Reduce batch sizes in the server code
3. Use smaller model variants if available

## Directory Structure

```
backend/
+-- docker/
|   +-- openvoice/
|   |   +-- Dockerfile
|   |   +-- server.py
|   +-- openf5/
|       +-- Dockerfile
|       +-- server.py
+-- scripts/
|   +-- setup-arch.sh
|   +-- build-all.sh
|   +-- run-all.sh
|   +-- run-openvoice.sh
|   +-- run-openf5.sh
|   +-- stop-all.sh
|   +-- status.sh
+-- README.md
```

## License

Copyright (c) 2025 Michael A Wright

MIT License
