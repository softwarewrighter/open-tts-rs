# Implementation Plan: open-tts-rs

## Phase 0: Project Setup

### 0.1 Repository Structure
- [ ] Update `Cargo.toml` with dependencies
- [ ] Create module directory structure
- [ ] Setup `.gitignore` for Rust and Python artifacts
- [ ] Add LICENSE file (MIT or Apache 2.0)

### 0.2 Development Environment
- [ ] Document Python environment requirements
- [ ] Create `requirements.txt` for Python dependencies
- [ ] Setup GitHub Actions for CI (optional)

---

## Phase 1: CLI Foundation

### 1.1 Argument Parsing
- [ ] Implement `Args` struct with clap derive macros
- [ ] Add `ModelArg` enum (`ov`, `of`)
- [ ] Implement `Reference::parse()` for `"file;transcript"` format
- [ ] Add validation for file existence

**Files:** `src/cli/mod.rs`, `src/cli/args.rs`

### 1.2 Error Types
- [ ] Create `Error` enum with thiserror
- [ ] Implement user-friendly error messages
- [ ] Add context to errors where helpful

**Files:** `src/core/error.rs`

### 1.3 Basic CLI Dispatch
- [ ] Wire up main.rs to parse args
- [ ] Implement `--help` and `--version`
- [ ] Add stub handlers for each command path

**Files:** `src/main.rs`, `src/cli/commands.rs`

**Milestone:** CLI parses all arguments and prints parsed values

---

## Phase 2: Audio I/O

### 2.1 Audio Reading
- [ ] Implement WAV reading with `hound`
- [ ] Add MP3/FLAC support with `symphonia` (optional)
- [ ] Create `AudioBuffer` type
- [ ] Add mono conversion

**Files:** `src/audio/mod.rs`, `src/audio/reader.rs`, `src/audio/buffer.rs`

### 2.2 Audio Writing
- [ ] Implement WAV writing with `hound`
- [ ] Add sample rate / bit depth options

**Files:** `src/audio/writer.rs`

### 2.3 Resampling
- [ ] Integrate `rubato` for resampling
- [ ] Resample to model-required rate (24kHz)

**Files:** `src/audio/buffer.rs`

**Milestone:** Can read WAV, convert to mono, resample, and write output

---

## Phase 3: Voice Management

### 3.1 Voice Embedding Type
- [ ] Define `VoiceEmbedding` struct
- [ ] Implement serde serialization
- [ ] Add model type tracking

**Files:** `src/voice/embedding.rs`

### 3.2 Storage Layer
- [ ] Determine storage directory (`~/.open-tts-rs/voices/`)
- [ ] Implement voice index (JSON metadata)
- [ ] Write/read binary embedding files

**Files:** `src/voice/storage.rs`

### 3.3 Voice Manager
- [ ] Implement `save(name, voice)`
- [ ] Implement `load(name) -> voice`
- [ ] Implement `list() -> Vec<(name, metadata)>`
- [ ] Implement `delete(name)`
- [ ] Add `--list-voices` command
- [ ] Add `--delete-voice` command

**Files:** `src/voice/manager.rs`, `src/voice/mod.rs`

**Milestone:** Can save, load, list, and delete named voices

---

## Phase 4: Backend Trait & Infrastructure

### 4.1 Backend Trait
- [ ] Define `TTSBackend` trait
- [ ] Define `BackendError` type
- [ ] Create factory function for backend selection

**Files:** `src/backend/mod.rs`

### 4.2 Python Integration Setup
- [ ] Add PyO3 dependency
- [ ] Test basic Python interop
- [ ] Create helper functions for numpy array conversion
- [ ] Document Python environment requirements

**Files:** `src/backend/python_bridge.rs` (new)

**Milestone:** Can call Python code from Rust and exchange data

---

## Phase 5: OpenVoice V2 Backend

### 5.1 Research & Setup
- [ ] Install OpenVoice V2 Python package
- [ ] Study OpenVoice API (`se_extractor`, `ToneColorConverter`)
- [ ] Identify exact Python calls needed
- [ ] Test voice extraction from Python REPL

### 5.2 Voice Extraction
- [ ] Implement `extract_voice()` for OpenVoice
- [ ] Handle speaker embedding extraction
- [ ] Convert numpy arrays to Rust Vec<f32>
- [ ] Serialize embedding to `VoiceEmbedding`

**Files:** `src/backend/openvoice.rs`

### 5.3 Speech Synthesis
- [ ] Integrate MeloTTS for base generation
- [ ] Apply tone color conversion
- [ ] Return generated audio as `AudioBuffer`

**Files:** `src/backend/openvoice.rs`

### 5.4 Testing
- [ ] Test voice extraction with sample audio
- [ ] Test synthesis quality
- [ ] Test round-trip (extract -> save -> load -> synthesize)

**Milestone:** Complete OpenVoice V2 integration working end-to-end

---

## Phase 6: OpenF5-TTS Backend

### 6.1 Research & Setup
- [ ] Install OpenF5-TTS Python package (Apache 2.0 weights)
- [ ] Study F5-TTS API
- [ ] Verify using commercially-licensed weights

### 6.2 Voice Reference Handling
- [ ] F5-TTS uses reference audio directly (no separate embedding)
- [ ] Store reference audio + transcript as "voice"
- [ ] Handle reference format conversion

**Files:** `src/backend/openf5.rs`

### 6.3 Speech Synthesis
- [ ] Implement `synthesize()` for F5-TTS
- [ ] Pass reference audio + transcript + target text
- [ ] Convert output to `AudioBuffer`

**Files:** `src/backend/openf5.rs`

### 6.4 Testing
- [ ] Test with various reference audio lengths
- [ ] Compare quality with OpenVoice
- [ ] Test model switching

**Milestone:** Complete OpenF5-TTS integration working end-to-end

---

## Phase 7: Engine Integration

### 7.1 TTS Engine
- [ ] Implement `TTSEngine` orchestrator
- [ ] Backend selection based on `--model`
- [ ] Session state management (current voice)

**Files:** `src/core/engine.rs`

### 7.2 Command Handlers
- [ ] Wire `--reference` to voice extraction
- [ ] Wire `--name` to save/load
- [ ] Wire `--generate` to synthesis
- [ ] Wire output to audio writing

**Files:** `src/cli/commands.rs`, `src/main.rs`

### 7.3 Progress Indication
- [ ] Add spinners for long operations
- [ ] Show extraction progress
- [ ] Show synthesis progress

**Files:** `src/cli/progress.rs`

**Milestone:** Full CLI workflow operational

---

## Phase 8: Polish & Release

### 8.1 Error Handling
- [ ] Improve error messages
- [ ] Add suggestions for common errors
- [ ] Handle missing Python dependencies gracefully

### 8.2 Documentation
- [ ] Write README.md
- [ ] Add usage examples
- [ ] Document Python setup requirements
- [ ] Add troubleshooting guide

### 8.3 Testing
- [ ] Add unit tests for parsing
- [ ] Add unit tests for audio I/O
- [ ] Add integration tests (requires Python env)
- [ ] Test on multiple platforms

### 8.4 Distribution
- [ ] Setup crates.io publishing (optional)
- [ ] Create release binaries
- [ ] Add installation instructions

**Milestone:** v0.1.0 release ready

---

## Dependency Summary

```toml
[dependencies]
# CLI
clap = { version = "4", features = ["derive"] }

# Audio
hound = "3.5"
rubato = "0.15"

# Python integration
pyo3 = { version = "0.22", features = ["auto-initialize"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
bincode = "1"
chrono = { version = "0.4", features = ["serde"] }

# Utilities
anyhow = "1"
thiserror = "2"
dirs = "5"
indicatif = "0.17"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

## Python Requirements

```
# requirements.txt
openvoice>=2.0
melo-tts
f5-tts  # Use OpenF5 weights from HuggingFace
torch
numpy
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| PyO3 complexity | Fallback to subprocess-based Python calls |
| Model API changes | Pin specific Python package versions |
| GPU dependencies | Support CPU-only mode, document CUDA setup |
| Large model downloads | Cache models, provide download progress |
| License ambiguity | Strictly verify Apache 2.0/MIT for all weights |

---

## Definition of Done

- [ ] All CLI arguments work as documented
- [ ] Both model backends (ov, of) functional
- [ ] Voice save/load works correctly
- [ ] Generated audio plays correctly
- [ ] Error messages are clear and actionable
- [ ] README documents all setup steps
- [ ] Works on Linux (primary), macOS, Windows
