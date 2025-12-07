# open-tts-rs

A Rust command-line interface for text-to-speech generation using open-source, commercially licensed TTS models. Supports voice cloning via reference audio and speech synthesis using the cloned voice.

## Features

- **Voice Cloning**: Clone voices from reference audio samples (3-30 seconds)
- **Multiple Models**: Support for OpenVoice V2 (MIT) and OpenF5-TTS (Apache 2.0)
- **Voice Management**: Save, load, list, and delete named voices
- **Commercial Use**: All supported models are permissively licensed

## Installation

```bash
# Clone the repository
git clone https://github.com/softwarewrighter/open-tts-rs.git
cd open-tts-rs

# Build (requires Rust 2024 edition)
cargo build --release

# Install (optional)
cargo install --path .
```

### Prerequisites

- Rust 1.85+ (2024 edition)
- Python 3.10+ with pip
- NVIDIA GPU with CUDA (recommended) or CPU-only mode

### Python Dependencies

```bash
# Install Python TTS libraries
pip install openvoice melo-tts f5-tts torch numpy
```

## Usage

```
open-tts-rs [OPTIONS]

OPTIONS:
    -m, --model <MODEL>        TTS model: "ov" (OpenVoiceV2) or "of" (OpenF5-TTS) [default: ov]
    -r, --reference <REF>      Reference audio with transcript: "file.wav;transcript text"
    -g, --generate <TEXT>      Text to generate speech from
    -n, --name <NAME>          Name for saving/loading voice
    -o, --output <FILE>        Output audio file [default: output.wav]
    -s, --speed <SPEED>        Speech speed multiplier 0.5-2.0 [default: 1.0]
        --host <HOST>          Backend server address [default: localhost]
    -v, --verbose              Enable verbose output
        --list-voices          List all saved voices
        --delete-voice <NAME>  Delete a saved voice
    -h, --help                 Print help information
    -V, --version              Print version information
```

### Examples

```bash
# Clone voice from reference and generate speech immediately
open-tts-rs --host curiosity -m ov -n myvoice \
            -r "sample.wav;Hello, this is a sample of my voice." \
            -g "Welcome to the demonstration." \
            -o output.wav

# Clone and save a voice for later use (OpenF5 model)
open-tts-rs --host curiosity -m of -n my_voice \
            -r "recording.wav;This is my recorded message."

# Generate using a previously saved voice
open-tts-rs --host curiosity -m ov -n my_voice \
            -g "Generate this text with my saved voice." \
            -o speech.wav

# Adjust speech speed (0.5 = slow, 2.0 = fast)
open-tts-rs --host curiosity -m ov -n my_voice -s 1.2 \
            -g "This will be spoken slightly faster." \
            -o fast_speech.wav

# List all saved voices on backend
open-tts-rs --host curiosity -m ov --list-voices

# Delete a saved voice from backend
open-tts-rs --host curiosity -m ov --delete-voice old_voice
```

## Supported Models

| Model | Flag | License | Best For |
|-------|------|---------|----------|
| OpenVoice V2 | `ov` | MIT | Fast voice cloning with good timbre matching |
| OpenF5-TTS | `of` | Apache 2.0 | Advanced atmospheric cloning with emotion preservation |

## Backend Server

The TTS models run as Docker containers on a backend server. See [backend/README.md](backend/README.md).

### Quick Start (Arch Linux host)

```bash
# Copy backend to server
scp -r backend/ curiosity:~/open-tts-rs/

# On the server
cd ~/open-tts-rs/backend/scripts
chmod +x *.sh
./setup-arch.sh   # Install Docker + NVIDIA runtime
# Log out/in for docker group
./build-all.sh    # Build containers (15-30 min)
./run-all.sh      # Start services
```

### Service Ports

| Service | Port | URL |
|---------|------|-----|
| OpenVoice V2 | 9280 | http://curiosity:9280 |
| OpenF5-TTS | 9288 | http://curiosity:9288 |

### CRITICAL: Blackwell GPU Requirements

The backend is designed for **NVIDIA RTX 5060 (Blackwell)** which requires:

| Requirement | Value |
|-------------|-------|
| **CUDA** | 12.8+ (Blackwell sm_120 not supported by older versions) |
| **PyTorch** | Nightly builds only (stable does not support Blackwell yet) |
| **Driver** | 560+ |
| **Base Image** | `nvidia/cuda:12.8.0-cudnn-devel-ubuntu24.04` |

The Dockerfiles handle all CUDA/PyTorch requirements automatically.

## Documentation

- [Product Requirements (PRD)](docs/prd.md) - Features, requirements, and use cases
- [Architecture](docs/architecture.md) - System design and component overview
- [Design](docs/design.md) - Technical design and code structure
- [Implementation Plan](docs/plan.md) - Phased development roadmap
- [Project Status](docs/status.md) - Current progress and next steps
- [AI Agent Instructions](docs/ai_agent_instructions.md) - Guidelines for AI coding agents
- [Development Process](docs/process.md) - TDD workflow and quality standards
- [Development Tools](docs/tools.md) - Recommended tooling

## Development

This project uses Rust 2024 edition and follows Test-Driven Development (TDD).

```bash
# Run tests
cargo test

# Run linter (zero warnings policy)
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Build documentation
cargo doc --open
```

### Pre-Commit Checklist

Before committing, ensure:
1. All tests pass (`cargo test`)
2. Zero clippy warnings (`cargo clippy -- -D warnings`)
3. Code formatted (`cargo fmt`)
4. Documentation updated

See [docs/process.md](docs/process.md) for the complete development workflow.

## Project Structure

```
open-tts-rs/
+-- src/
|   +-- main.rs           # Entry point
|   +-- cli/              # Command-line interface
|   +-- core/             # Business logic
|   +-- backend/          # TTS model backends (Rust)
|   +-- voice/            # Voice management
|   +-- audio/            # Audio I/O
+-- backend/
|   +-- docker/
|   |   +-- openvoice/    # OpenVoice V2 container
|   |   +-- openf5/       # OpenF5-TTS container
|   +-- scripts/          # Build/run scripts for Arch Linux
|   +-- README.md         # Backend documentation
+-- docs/
|   +-- prd.md            # Product requirements
|   +-- architecture.md   # System architecture
|   +-- design.md         # Technical design
|   +-- plan.md           # Implementation plan
|   +-- status.md         # Project status
|   +-- process.md        # Development process
|   +-- tools.md          # Development tools
+-- Cargo.toml
+-- README.md
+-- LICENSE
```

## License

Copyright (c) 2025 Michael A Wright

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

- [OpenVoice](https://github.com/myshell-ai/OpenVoice) - MIT licensed voice cloning
- [F5-TTS](https://github.com/SWivid/F5-TTS) - Flow-matching TTS (use OpenF5 weights for commercial use)
- [Kokoro](https://github.com/hexgrad/kokoro) - Research reference for TTS quality standards
