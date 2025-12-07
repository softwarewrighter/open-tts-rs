# Project Status: open-tts-rs

## Current Version: 0.1.0-dev

## Overall Progress

| Phase | Description | Status | Progress |
|-------|-------------|--------|----------|
| 0 | Project Setup | Complete | 100% |
| 1 | CLI Foundation | Complete | 100% |
| 2 | Audio I/O | Deferred | 0% |
| 3 | Voice Management | Complete | 100% |
| 4 | Backend Infrastructure | Complete | 100% |
| 5 | Docker Backend Setup | Complete | 100% |
| 6 | Engine Integration | Complete | 100% |
| 7 | E2E Testing | Complete | 100% |
| 8 | Polish & Release | Not Started | 0% |

**Overall: ~90% complete** (ready for release polish)

---

## Detailed Status

### Phase 0: Project Setup
- [x] Create initial project structure
- [x] Write documentation (PRD, Architecture, Design, Plan)
- [x] Update Cargo.toml with dependencies (Rust 2024 edition)
- [x] Setup module structure
- [x] Add .gitignore
- [x] Create CLAUDE.md with TDD emphasis

### Phase 1: CLI Foundation (TDD Complete)
- [x] Implement argument parsing (clap derive)
- [x] Reference string parsing (`-r "file.wav;transcript"`)
- [x] Model selection (`-m ov|of`)
- [x] Error types (thiserror)
- [x] Wire up main dispatch
- [x] 11 unit tests passing

### Phase 2: Audio I/O
- [ ] WAV reading (deferred - backend handles audio)
- [ ] WAV writing (raw bytes from backend)
- [ ] Resampling support (backend handles)

### Phase 3: Voice Management (TDD Complete)
- [x] VoiceMetadata type
- [x] VoiceManager with local storage
- [x] Save/Load metadata operations
- [x] List voices command
- [x] Delete voice command
- [x] Path traversal protection
- [x] 8 unit tests passing

### Phase 4: Backend Infrastructure (TDD Complete)
- [x] Backend trait definition
- [x] HttpBackend HTTP client implementation
- [x] MockBackend for testing (mockall)
- [x] Request/Response types (serde)
- [x] Error handling
- [x] 13 unit tests passing

### Phase 5: Docker Backend Setup
- [x] OpenVoice V2 Dockerfile (port 9280)
- [x] OpenF5-TTS Dockerfile (port 9288)
- [x] CUDA 12.8+ base images (Blackwell GPU support)
- [x] PyTorch nightly builds
- [x] REST API endpoints documented
- [x] Setup scripts for Arch Linux
- [x] Build/run/stop/status scripts

### Phase 6: Engine Integration (TDD Complete)
- [x] TTSEngine orchestrator
- [x] health_check command
- [x] extract_voice workflow
- [x] synthesize workflow
- [x] list_voices command
- [x] delete_voice command
- [x] 8 unit tests passing
- [x] Main.rs wired up

### Phase 7: E2E Testing (Complete)
- [x] Start Docker backends on curiosity (RTX 5060 Ti)
- [x] Test voice extraction (OpenVoice V2 + OpenF5-TTS)
- [x] Test speech synthesis (both models)
- [x] Test voice management (list, delete)
- [x] Multi-format input testing (WAV, M4A→WAV via ffmpeg)

### Phase 8: Polish & Release
- [ ] Error message improvements
- [ ] README examples with actual output
- [ ] Cross-platform testing
- [ ] Release preparation

---

## Test Summary

```
running 40 tests
test backend::tests::... ok (13 tests)
test cli::tests::... ok (11 tests)
test voice::tests::... ok (8 tests)
test engine::tests::... ok (8 tests)

test result: ok. 40 passed; 0 failed
```

---

## Recent Updates

### 2025-12-07 (E2E Testing Complete)
- **End-to-End Testing Complete**
  - Both OpenVoice V2 and OpenF5-TTS models verified working
  - Voice extraction tested with real audio (WAV and M4A→WAV)
  - Speech synthesis generating valid audio output
  - Full CLI workflow validated against Docker backends on curiosity

- **Backend API Fixes**
  - Fixed SynthesizeRequest field name mismatch (`voice_name` → `name` via serde rename)
  - Server-side response format aligned with VoiceInfo struct
  - Both backends returning correct JSON format

- **Code Quality**
  - cargo fmt applied
  - cargo clippy clean (no warnings)
  - All 40 unit tests + 1 doc test passing

### 2025-12-07 (Earlier)
- **TDD Implementation Complete**
  - All core modules implemented with comprehensive tests
  - 40 unit tests + 1 doc test passing
  - MockBackend for isolated testing

- **CLI Fully Functional**
  - All arguments wired to engine
  - Host and speed parameters added
  - Help text documented

- **Architecture Finalized**
  - Docker-based backend (not PyO3)
  - REST API communication
  - Local voice metadata storage

---

## Blockers

*No blockers currently*

---

## Next Steps

1. Final commit with Docker script updates
2. Add example usage to README with actual output
3. Cross-platform testing (Linux, macOS)
4. Release preparation (v0.1.0)

---

## Architecture Decision: Docker over PyO3

Changed from PyO3 embedding to Docker containers because:
1. **Simpler deployment** - No Python environment management in Rust
2. **GPU isolation** - Docker handles CUDA setup
3. **Blackwell support** - PyTorch nightly builds only available in Python
4. **Testing** - MockBackend allows full unit testing without GPU

---

## Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Lines of Code | ~2000 | ~800 |
| Test Coverage | >70% | ~90% |
| CLI Commands | 7 | 7 |
| Supported Models | 2 | 2 |
| Unit Tests | 30+ | 41 |

---

## Links

- [OpenVoice V2](https://github.com/myshell-ai/OpenVoice) - MIT License
- [OpenF5-TTS](https://huggingface.co/SWivid/F5-TTS) - Apache 2.0
- [CUDA 12.8](https://developer.nvidia.com/cuda-toolkit) - Required for Blackwell
- [PyTorch Nightly](https://pytorch.org/get-started/locally/) - Required for sm_120
