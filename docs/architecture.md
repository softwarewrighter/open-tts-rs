# Architecture Document: open-tts-rs

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         open-tts-rs CLI                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌───────────┐    ┌──────────────┐    ┌───────────────────────┐   │
│  │    CLI    │───▶│   Command    │───▶│    Voice Manager      │   │
│  │  Parser   │    │   Router     │    │  (load/save/delete)   │   │
│  └───────────┘    └──────────────┘    └───────────────────────┘   │
│                          │                       │                 │
│                          ▼                       ▼                 │
│                   ┌──────────────┐    ┌───────────────────────┐   │
│                   │   Model      │    │    Voice Storage      │   │
│                   │   Backend    │    │  (~/.open-tts-rs/)    │   │
│                   │   Selector   │    └───────────────────────┘   │
│                   └──────────────┘                                 │
│                          │                                         │
│            ┌─────────────┴─────────────┐                          │
│            ▼                           ▼                          │
│  ┌──────────────────┐       ┌──────────────────┐                  │
│  │  OpenVoice V2    │       │   OpenF5-TTS     │                  │
│  │    Backend       │       │     Backend      │                  │
│  └──────────────────┘       └──────────────────┘                  │
│            │                           │                          │
│            └─────────────┬─────────────┘                          │
│                          ▼                                         │
│                   ┌──────────────┐                                 │
│                   │    Audio     │                                 │
│                   │   Output     │                                 │
│                   └──────────────┘                                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Component Architecture

### 1. CLI Layer (`src/cli/`)

Handles command-line argument parsing and validation.

```
src/cli/
├── mod.rs          # CLI module entry point
├── args.rs         # Argument definitions (clap)
└── validators.rs   # Input validation functions
```

**Responsibilities:**
- Parse command-line arguments using `clap`
- Validate input formats (reference string parsing, file existence)
- Route commands to appropriate handlers

**Key Structures:**
```rust
pub struct Args {
    pub model: Model,           // ov | of
    pub reference: Option<Reference>,
    pub generate: Option<String>,
    pub name: Option<String>,
    pub output: PathBuf,
    pub verbose: bool,
    pub list_voices: bool,
    pub delete_voice: Option<String>,
}

pub struct Reference {
    pub audio_path: PathBuf,
    pub transcript: String,
}

pub enum Model {
    OpenVoiceV2,  // ov
    OpenF5TTS,    // of
}
```

### 2. Core Layer (`src/core/`)

Business logic and orchestration.

```
src/core/
├── mod.rs          # Core module entry point
├── engine.rs       # Main TTS engine orchestrator
├── voice.rs        # Voice representation and operations
└── error.rs        # Error types
```

**Responsibilities:**
- Coordinate voice extraction and synthesis operations
- Manage voice state during session
- Handle error propagation

### 3. Backend Layer (`src/backend/`)

Model-specific implementations.

```
src/backend/
├── mod.rs          # Backend trait definition
├── openvoice.rs    # OpenVoice V2 implementation
└── openf5.rs       # OpenF5-TTS implementation
```

**Backend Trait:**
```rust
pub trait TTSBackend: Send + Sync {
    /// Extract voice embedding from reference audio + transcript
    fn extract_voice(
        &self,
        audio_path: &Path,
        transcript: &str,
    ) -> Result<VoiceEmbedding>;

    /// Generate speech from text using voice embedding
    fn synthesize(
        &self,
        text: &str,
        voice: &VoiceEmbedding,
    ) -> Result<AudioBuffer>;

    /// Get model name for display
    fn name(&self) -> &'static str;
}
```

**Integration Strategy:**
Both models are Python-based. We'll use one of these approaches:
1. **PyO3** - Direct Python binding (preferred for performance)
2. **Subprocess** - Shell out to Python scripts (simpler, fallback)
3. **ONNX Runtime** - If models export to ONNX (best performance)

### 4. Voice Manager (`src/voice/`)

Persistent voice storage and retrieval.

```
src/voice/
├── mod.rs          # Voice manager entry point
├── storage.rs      # File system operations
└── embedding.rs    # Voice embedding serialization
```

**Storage Layout:**
```
~/.open-tts-rs/
├── config.toml           # User configuration
├── voices/
│   ├── index.json        # Voice metadata index
│   ├── my_voice.bin      # Serialized voice embedding
│   └── another_voice.bin
└── cache/
    └── models/           # Downloaded model weights
```

**Voice Index Schema:**
```json
{
  "voices": {
    "my_voice": {
      "model": "openvoice_v2",
      "created": "2025-01-15T10:30:00Z",
      "reference_transcript": "Hello, this is my voice.",
      "file": "my_voice.bin"
    }
  }
}
```

### 5. Audio Layer (`src/audio/`)

Audio file I/O and processing.

```
src/audio/
├── mod.rs          # Audio module entry point
├── reader.rs       # Read WAV/MP3/FLAC files
├── writer.rs       # Write WAV/MP3 output
└── buffer.rs       # In-memory audio representation
```

**Key Types:**
```rust
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}
```

## Data Flow

### Voice Cloning Flow

```
┌────────────┐    ┌───────────────┐    ┌─────────────┐    ┌──────────────┐
│ Reference  │───▶│ Audio Reader  │───▶│  Backend    │───▶│   Voice      │
│ WAV File   │    │ (decode)      │    │  extract()  │    │  Embedding   │
└────────────┘    └───────────────┘    └─────────────┘    └──────────────┘
                                              │                   │
                         ┌────────────────────┘                   │
                         ▼                                        ▼
                  ┌─────────────┐                         ┌──────────────┐
                  │ Transcript  │                         │ Voice Store  │
                  │ (from CLI)  │                         │ (if --name)  │
                  └─────────────┘                         └──────────────┘
```

### Speech Generation Flow

```
┌────────────┐    ┌───────────────┐    ┌─────────────┐    ┌──────────────┐
│ Text Input │───▶│   Backend     │───▶│ Audio       │───▶│ Output File  │
│ (--generate)│   │  synthesize() │    │ Writer      │    │ (WAV/MP3)    │
└────────────┘    └───────────────┘    └─────────────┘    └──────────────┘
                         ▲
                         │
                  ┌──────┴───────┐
                  │    Voice     │
                  │  Embedding   │
                  │ (from ref or │
                  │  saved name) │
                  └──────────────┘
```

## Dependency Architecture

```toml
[dependencies]
# CLI
clap = { version = "4", features = ["derive"] }

# Audio processing
hound = "3"              # WAV I/O
symphonia = "0.5"        # Multi-format audio decoding
rubato = "0.15"          # Resampling

# Python integration (choose one)
pyo3 = { version = "0.22", features = ["auto-initialize"] }
# OR for subprocess approach:
# (no additional deps, use std::process)

# ONNX (optional, for direct inference)
ort = { version = "2", optional = true }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Utilities
anyhow = "1"             # Error handling
thiserror = "2"          # Custom errors
dirs = "5"               # Platform directories
indicatif = "0.17"       # Progress bars
tracing = "0.1"          # Logging
```

## Error Handling Strategy

```rust
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Audio file not found: {0}")]
    AudioNotFound(PathBuf),

    #[error("Invalid reference format. Expected 'file.wav;transcript'")]
    InvalidReferenceFormat,

    #[error("Voice '{0}' not found. Use --list-voices to see available voices.")]
    VoiceNotFound(String),

    #[error("Model error: {0}")]
    ModelError(String),

    #[error("Audio processing error: {0}")]
    AudioError(#[from] hound::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## Configuration

**Default Config (`~/.open-tts-rs/config.toml`):**
```toml
[general]
default_model = "ov"
default_output_format = "wav"

[openvoice]
# Path to OpenVoice installation (auto-detected if in PATH)
python_path = "/usr/bin/python3"

[openf5]
python_path = "/usr/bin/python3"

[audio]
sample_rate = 24000
```

## Security Considerations

1. **Input Validation**: Sanitize all file paths to prevent path traversal
2. **Subprocess Safety**: If using Python subprocess, escape arguments properly
3. **Model Integrity**: Verify model checksums on download
4. **No Network by Default**: All operations are local-only

## Platform Support

| Platform | GPU Support | Status |
|----------|-------------|--------|
| Linux x64 | CUDA | Primary |
| macOS ARM | MPS (future) | Secondary |
| macOS x64 | CPU only | Secondary |
| Windows x64 | CUDA | Secondary |

## Docker Backend Architecture

The TTS models run as Docker containers on a backend server, exposing REST APIs.

```
+------------------+          +----------------------------+
|   open-tts-rs    |   HTTP   |    Backend Host            |
|      CLI         |<-------->|    (curiosity)             |
+------------------+          |                            |
                              |  +----------------------+  |
                              |  | openvoice-server     |  |
                              |  | Port: 9280           |  |
                              |  | CUDA 12.8 / sm_120   |  |
                              |  +----------------------+  |
                              |                            |
                              |  +----------------------+  |
                              |  | openf5-server        |  |
                              |  | Port: 9288           |  |
                              |  | CUDA 12.8 / sm_120   |  |
                              |  +----------------------+  |
                              |                            |
                              |  +----------------------+  |
                              |  | NVIDIA RTX 5060 16GB |  |
                              |  | (Blackwell)          |  |
                              |  +----------------------+  |
                              +----------------------------+
```

### CRITICAL: Blackwell GPU Requirements

The target GPU is an **NVIDIA RTX 5060 (Blackwell architecture)** which has specific requirements:

| Requirement | Value | Notes |
|-------------|-------|-------|
| CUDA Version | 12.8+ | Blackwell (sm_120) requires CUDA 12.8 minimum |
| PyTorch | Nightly | Stable releases do not support sm_120 yet |
| Driver | 560+ | Latest NVIDIA driver required |
| Base Image | `nvidia/cuda:12.8.0-cudnn-devel-ubuntu24.04` | Ubuntu 24.04 for Python 3.12 |

**PyTorch Installation for Blackwell:**
```bash
# MUST use nightly with cu128
pip install --pre torch torchaudio \
    --index-url https://download.pytorch.org/whl/nightly/cu128
```

**Environment Variables:**
```bash
export TORCH_CUDA_ARCH_LIST="12.0"  # Blackwell compute capability
```

### Container Ports

| Service | Port | Model | License |
|---------|------|-------|---------|
| OpenVoice V2 | 9280 | Voice cloning + MeloTTS | MIT |
| OpenF5-TTS | 9288 | Flow-matching TTS | Apache 2.0 |

### Backend Scripts

Located in `backend/scripts/`:

| Script | Description |
|--------|-------------|
| `setup-arch.sh` | Install Docker + NVIDIA runtime on Arch Linux |
| `build-all.sh` | Build both Docker containers |
| `run-all.sh` | Start both services |
| `stop-all.sh` | Stop all services |
| `status.sh` | Check service and GPU status |

## Testing Strategy

- **Unit Tests**: Core logic, voice parsing, audio I/O
- **Integration Tests**: End-to-end CLI workflows
- **Model Tests**: Backend correctness with known inputs (requires model setup)
