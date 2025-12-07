# Project Status: open-tts-rs

## Current Version: 0.1.0-dev

## Overall Progress

| Phase | Description | Status | Progress |
|-------|-------------|--------|----------|
| 0 | Project Setup | Not Started | 0% |
| 1 | CLI Foundation | Not Started | 0% |
| 2 | Audio I/O | Not Started | 0% |
| 3 | Voice Management | Not Started | 0% |
| 4 | Backend Infrastructure | Not Started | 0% |
| 5 | OpenVoice V2 Backend | Not Started | 0% |
| 6 | OpenF5-TTS Backend | Not Started | 0% |
| 7 | Engine Integration | Not Started | 0% |
| 8 | Polish & Release | Not Started | 0% |

**Overall: 0% complete**

---

## Detailed Status

### Phase 0: Project Setup
- [x] Create initial project structure
- [x] Write documentation (PRD, Architecture, Design, Plan)
- [ ] Update Cargo.toml with dependencies
- [ ] Setup module structure
- [ ] Add LICENSE file

### Phase 1: CLI Foundation
- [ ] Implement argument parsing
- [ ] Add reference string parsing
- [ ] Create error types
- [ ] Wire up main dispatch

### Phase 2: Audio I/O
- [ ] WAV reading
- [ ] WAV writing
- [ ] Resampling support
- [ ] AudioBuffer type

### Phase 3: Voice Management
- [ ] VoiceEmbedding type
- [ ] Storage layer
- [ ] VoiceManager CRUD operations
- [ ] --list-voices command
- [ ] --delete-voice command

### Phase 4: Backend Infrastructure
- [ ] TTSBackend trait
- [ ] PyO3 integration
- [ ] Backend factory

### Phase 5: OpenVoice V2 Backend
- [ ] Research & setup
- [ ] Voice extraction
- [ ] Speech synthesis
- [ ] Integration testing

### Phase 6: OpenF5-TTS Backend
- [ ] Research & setup
- [ ] Reference handling
- [ ] Speech synthesis
- [ ] Integration testing

### Phase 7: Engine Integration
- [ ] TTSEngine orchestrator
- [ ] Command handlers
- [ ] Progress indication

### Phase 8: Polish & Release
- [ ] Error message improvements
- [ ] README documentation
- [ ] Cross-platform testing
- [ ] Release preparation

---

## Recent Updates

### 2025-12-07
- Initial project scaffolding created
- Documentation completed:
  - `docs/prd.md` - Product Requirements Document
  - `docs/architecture.md` - System Architecture
  - `docs/design.md` - Technical Design
  - `docs/plan.md` - Implementation Plan
  - `docs/status.md` - This status file
- Basic `Cargo.toml` and `main.rs` in place

---

## Blockers

*No blockers currently*

---

## Next Steps

1. Update `Cargo.toml` with all required dependencies
2. Create module directory structure (`src/cli/`, `src/core/`, etc.)
3. Implement CLI argument parsing with clap
4. Begin audio I/O module

---

## Open Questions

1. **PyO3 vs Subprocess**: Should we use PyO3 for direct Python embedding, or start with subprocess for simpler debugging?
   - *Recommendation*: Start with subprocess for rapid iteration, migrate to PyO3 later.

2. **Model Weight Management**: How should we handle model weight downloads?
   - *Options*: Require manual setup, auto-download on first use, provide setup script.

3. **Voice Compatibility**: Should voices be portable between models?
   - *Current decision*: No - voices are model-specific. Track model type in metadata.

---

## Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Lines of Code | ~2000 | ~10 |
| Test Coverage | >70% | 0% |
| CLI Commands | 7 | 0 |
| Supported Models | 2 | 0 |

---

## Contributors

- *Your name here*

---

## Links

- [OpenVoice V2](https://github.com/myshell-ai/OpenVoice) - MIT License
- [OpenF5-TTS](https://huggingface.co/SWivid/F5-TTS) - Apache 2.0 (use OpenF5 weights)
- [Clap Documentation](https://docs.rs/clap)
- [PyO3 User Guide](https://pyo3.rs)
