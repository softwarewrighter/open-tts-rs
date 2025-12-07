# Product Requirements Document: open-tts-rs

## Overview

**open-tts-rs** is a Rust command-line interface (CLI) tool for text-to-speech generation using open-source, commercially licensed TTS models. It supports voice cloning via reference audio and subsequent speech synthesis using the cloned voice.

## Problem Statement

Developers and content creators need a simple, cross-platform CLI tool to:
1. Clone voices from reference audio samples
2. Generate high-quality speech using cloned voices
3. Use commercially permissive TTS models (MIT/Apache 2.0 licensed)
4. Operate locally without cloud dependencies

## Target Users

- Developers building TTS-enabled applications
- Content creators needing voice synthesis
- Researchers experimenting with voice cloning
- Businesses requiring commercially-licensed TTS solutions

## Supported Models

| Model | Flag | License | Cloning Type | Best For |
|-------|------|---------|--------------|----------|
| OpenVoice V2 | `ov` | MIT | Zero-shot (reference audio) | Fast voice cloning with good timbre matching |
| OpenF5-TTS | `of` | Apache 2.0 | Zero-shot (reference audio) | Advanced atmospheric cloning with emotion preservation |

## Functional Requirements

### FR-1: Model Selection
- **FR-1.1**: User can specify model via `-m` or `--model` flag
- **FR-1.2**: Accepted values: `ov` (OpenVoiceV2), `of` (OpenF5-TTS)
- **FR-1.3**: Default model: `ov` (OpenVoiceV2) - faster and more permissive license

### FR-2: Voice Reference Upload
- **FR-2.1**: User can provide reference audio via `-r` or `--reference` flag
- **FR-2.2**: Format: `"wav-file-path;transcript of the audio"`
- **FR-2.3**: Supported audio formats: WAV (primary), MP3, FLAC
- **FR-2.4**: Reference audio should be 3-30 seconds for optimal results
- **FR-2.5**: Transcript must match the spoken content in the reference audio

### FR-3: Voice Naming
- **FR-3.1**: User can name a voice via `-n` or `--name` flag
- **FR-3.2**: Named voices are stored locally for reuse
- **FR-3.3**: Names must be alphanumeric with underscores (e.g., `my_voice_1`)

### FR-4: Speech Generation
- **FR-4.1**: User can generate speech via `-g` or `--generate` flag
- **FR-4.2**: Input: text string to synthesize
- **FR-4.3**: Uses the most recently provided/loaded voice reference
- **FR-4.4**: Output: WAV file (default: `output.wav` or specified via `-o`)

### FR-5: Output Configuration
- **FR-5.1**: User can specify output file via `-o` or `--output` flag
- **FR-5.2**: Default output: `output.wav` in current directory
- **FR-5.3**: Supported output formats: WAV, MP3

### FR-6: Voice Management
- **FR-6.1**: List saved voices via `--list-voices`
- **FR-6.2**: Delete saved voice via `--delete-voice <name>`
- **FR-6.3**: Voices stored in `~/.open-tts-rs/voices/`

## Non-Functional Requirements

### NFR-1: Performance
- Reference voice extraction: < 10 seconds
- Speech generation: Real-time or faster on modern GPUs
- Support CPU-only mode (slower but functional)

### NFR-2: Compatibility
- Cross-platform: Linux, macOS, Windows
- GPU support: NVIDIA CUDA (primary), CPU fallback
- Rust edition: 2021+

### NFR-3: Usability
- Clear error messages with actionable guidance
- Progress indicators for long operations
- Verbose mode (`-v`) for debugging

### NFR-4: Licensing
- Tool itself: MIT or Apache 2.0
- All model dependencies: MIT or Apache 2.0 only
- No non-commercial model weights

## CLI Interface

```
open-tts-rs [OPTIONS]

OPTIONS:
    -m, --model <MODEL>        TTS model: "ov" (OpenVoiceV2) or "of" (OpenF5-TTS) [default: ov]
    -r, --reference <REF>      Reference audio with transcript: "file.wav;transcript text"
    -g, --generate <TEXT>      Text to generate speech from
    -n, --name <NAME>          Name for the voice (for saving/loading)
    -o, --output <FILE>        Output audio file [default: output.wav]
    -v, --verbose              Enable verbose output
        --list-voices          List all saved voices
        --delete-voice <NAME>  Delete a saved voice
    -h, --help                 Print help information
    -V, --version              Print version information
```

## Usage Examples

### Clone a voice and generate speech
```bash
# Clone voice from reference and generate immediately
open-tts-rs -m ov -r "sample.wav;Hello, this is a sample of my voice." -g "Welcome to the demonstration."

# Clone, name, and save a voice
open-tts-rs -m of -r "recording.wav;This is my recorded message." -n my_voice

# Generate using a previously saved voice
open-tts-rs -n my_voice -g "Generate this text with my saved voice." -o speech.wav
```

### Voice management
```bash
# List all saved voices
open-tts-rs --list-voices

# Delete a voice
open-tts-rs --delete-voice old_voice
```

## Success Metrics

1. **Functionality**: Successfully clone voices and generate speech with both supported models
2. **Quality**: Generated speech is intelligible and preserves voice characteristics
3. **Performance**: Generation completes in reasonable time (< 2x real-time on GPU)
4. **Reliability**: < 1% failure rate on valid inputs

## Out of Scope (v1.0)

- Real-time streaming synthesis
- Voice mixing/averaging (Kokoro-style)
- Web UI or API server mode
- Training/fine-tuning capabilities
- Multi-language support (English-first)

## Future Considerations (v2.0+)

- Add Kokoro support for voice mixing
- Streaming audio output
- Server mode with REST API
- Batch processing from file
- Multi-language expansion
