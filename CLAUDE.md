# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Process: Test-Driven Development (TDD)

**MANDATORY**: All code changes follow strict Red/Green/Refactor:

```
RED: Write failing test -> GREEN: Minimal code to pass -> REFACTOR: Improve -> REPEAT
```

### TDD Rules

1. **Write the test FIRST** - Before any implementation code
2. **Run the test, watch it FAIL** - Confirm it fails for the right reason
3. **Write MINIMAL code** - Just enough to make the test pass
4. **Refactor** - Clean up while keeping tests green
5. **Never skip tests** - No `#[ignore]`, no commenting out

### Testing Approach

- **Unit tests**: In same file, `#[cfg(test)]` module
- **Mocks/Spies**: Use traits for dependencies, mock implementations in tests
- **Test fixtures**: `tests/fixtures/` for sample audio files, JSON responses
- **Integration tests**: `tests/` directory for cross-module tests

### Example TDD Cycle

```rust
// 1. RED - Write failing test first
#[test]
fn test_parse_reference_valid() {
    let input = "audio.wav;Hello world";
    let result = Reference::parse(input);
    assert!(result.is_ok());
    let reference = result.unwrap();
    assert_eq!(reference.transcript, "Hello world");
}

// 2. GREEN - Implement minimal code to pass
// 3. REFACTOR - Clean up, add edge cases
```

## Project Overview

open-tts-rs is a Rust CLI for text-to-speech generation with voice cloning. It supports two commercially-licensed TTS models (OpenVoice V2 and OpenF5-TTS) running as Docker containers on a backend GPU server.

## Build Commands

```bash
# Build
cargo build --release

# Test
cargo test                    # Run all tests
cargo test test_name          # Run specific test
cargo test -- --nocapture     # Run with output

# Lint (zero warnings policy - MUST pass before commit)
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt
cargo fmt --check             # Verify formatting
```

## Pre-Commit Requirements

All commits MUST pass these checks in order:
1. `cargo test` - all tests pass
2. `cargo clippy --all-targets --all-features -- -D warnings` - zero warnings
3. `cargo fmt` - code formatted
4. `markdown-checker -f "**/*.md"` - ASCII-only markdown (if docs changed)

## Architecture

### Two-Tier System

1. **Rust CLI** (`src/`) - Parses args, manages voices, calls backend APIs
2. **Docker Backend** (`backend/`) - Python TTS servers on GPU host (curiosity)

### Rust Module Structure (Planned)

```
src/
+-- cli/        # Clap argument parsing, command routing
+-- core/       # TTSEngine orchestrator, session state
+-- backend/    # TTSBackend trait + HTTP client to Docker services
+-- voice/      # VoiceManager, embedding storage (~/.open-tts-rs/voices/)
+-- audio/      # AudioBuffer, WAV reading/writing, resampling
```

### Backend Services

| Service | Port | Model | License |
|---------|------|-------|---------|
| OpenVoice V2 | 9280 | Voice cloning + MeloTTS | MIT |
| OpenF5-TTS | 9288 | Flow-matching TTS | Apache 2.0 |

### Key Backend Endpoints

Both services expose identical REST APIs:
- `GET /health` - Health check with GPU info
- `POST /extract_voice` - Upload reference audio + transcript, get embedding
- `POST /synthesize` - Generate speech from text + voice name/embedding
- `GET /voices` - List saved voices
- `DELETE /voices/<name>` - Delete voice

## CLI Design

```
open-tts-rs -m <ov|of> -r "file.wav;transcript" -g "text to generate" -n voice_name -o output.wav
```

- `-m/--model`: "ov" (OpenVoiceV2) or "of" (OpenF5-TTS)
- `-r/--reference`: Reference audio with transcript (semicolon-separated)
- `-g/--generate`: Text to synthesize
- `-n/--name`: Voice name for save/load
- `--list-voices`, `--delete-voice`: Voice management

## Blackwell GPU Requirements (CRITICAL)

The backend targets **NVIDIA RTX 5060 (Blackwell, sm_120)** which requires:
- CUDA 12.8+ (older versions do not support Blackwell)
- PyTorch nightly builds (stable does not support sm_120)
- NVIDIA driver 560+
- Base image: `nvidia/cuda:12.8.0-cudnn-devel-ubuntu24.04`

The Dockerfiles in `backend/docker/` handle this automatically.

## Code Style

- Rust 2024 edition
- Use inline format args: `format!("{name}")` not `format!("{}", name)`
- Module docs use `//!`, item docs use `///`
- Files under 500 lines, functions under 50 lines
- Max 3 TODOs per file, never commit FIXMEs

## Backend Deployment

```bash
# Copy to GPU host and run
scp -r backend/ curiosity:~/open-tts-rs/
ssh curiosity
cd ~/open-tts-rs/backend/scripts
chmod +x *.sh
./setup-arch.sh   # Install Docker + NVIDIA runtime
./build-all.sh    # Build containers
./run-all.sh      # Start services
./status.sh       # Check status
```

## Voice Storage

Local voices stored in `~/.open-tts-rs/voices/` with JSON index and binary embeddings.

## Documentation

- `docs/prd.md` - Product requirements
- `docs/architecture.md` - System design with diagrams
- `docs/design.md` - Detailed code structure and types
- `docs/plan.md` - Implementation phases
- `docs/status.md` - Current progress
