# Technical Design Document: open-tts-rs

## 1. Module Structure

```
src/
├── main.rs                 # Entry point, CLI dispatch
├── cli/
│   ├── mod.rs
│   ├── args.rs             # Clap argument definitions
│   └── commands.rs         # Command handlers
├── core/
│   ├── mod.rs
│   ├── engine.rs           # TTSEngine orchestrator
│   ├── session.rs          # Session state (current voice)
│   └── error.rs            # Error types
├── backend/
│   ├── mod.rs              # TTSBackend trait
│   ├── openvoice.rs        # OpenVoice V2 implementation
│   └── openf5.rs           # OpenF5-TTS implementation
├── voice/
│   ├── mod.rs
│   ├── embedding.rs        # VoiceEmbedding type
│   ├── manager.rs          # VoiceManager (CRUD operations)
│   └── storage.rs          # File I/O for voices
└── audio/
    ├── mod.rs
    ├── reader.rs           # Multi-format audio reading
    ├── writer.rs           # Audio output writing
    └── buffer.rs           # AudioBuffer type
```

## 2. Core Types

### 2.1 CLI Arguments

```rust
// src/cli/args.rs
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "open-tts-rs")]
#[command(about = "Voice cloning and text-to-speech CLI")]
#[command(version)]
pub struct Args {
    /// TTS model to use
    #[arg(short, long, value_enum, default_value = "ov")]
    pub model: ModelArg,

    /// Reference audio with transcript: "file.wav;transcript text"
    #[arg(short, long)]
    pub reference: Option<String>,

    /// Text to generate speech from
    #[arg(short, long)]
    pub generate: Option<String>,

    /// Name for saving/loading voice
    #[arg(short, long)]
    pub name: Option<String>,

    /// Output audio file
    #[arg(short, long, default_value = "output.wav")]
    pub output: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// List all saved voices
    #[arg(long)]
    pub list_voices: bool,

    /// Delete a saved voice
    #[arg(long)]
    pub delete_voice: Option<String>,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum ModelArg {
    /// OpenVoice V2 (MIT license, fast)
    #[default]
    Ov,
    /// OpenF5-TTS (Apache 2.0, atmospheric cloning)
    Of,
}
```

### 2.2 Reference Parsing

```rust
// src/cli/args.rs
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Reference {
    pub audio_path: PathBuf,
    pub transcript: String,
}

impl Reference {
    /// Parse reference from "file.wav;transcript" format
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let parts: Vec<&str> = input.splitn(2, ';').collect();

        if parts.len() != 2 {
            return Err(ParseError::InvalidFormat(
                "Expected format: 'file.wav;transcript text'".into()
            ));
        }

        let audio_path = PathBuf::from(parts[0].trim());
        let transcript = parts[1].trim().to_string();

        if !audio_path.exists() {
            return Err(ParseError::FileNotFound(audio_path));
        }

        if transcript.is_empty() {
            return Err(ParseError::EmptyTranscript);
        }

        Ok(Self { audio_path, transcript })
    }
}
```

### 2.3 Voice Embedding

```rust
// src/voice/embedding.rs
use serde::{Deserialize, Serialize};

/// Voice embedding extracted from reference audio.
/// The actual data format depends on the backend model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceEmbedding {
    /// Which model created this embedding
    pub model: ModelType,
    /// Raw embedding data (model-specific format)
    pub data: Vec<u8>,
    /// Original transcript used for extraction
    pub transcript: String,
    /// Metadata
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    OpenVoiceV2,
    OpenF5TTS,
}

impl VoiceEmbedding {
    pub fn new(model: ModelType, data: Vec<u8>, transcript: String) -> Self {
        Self {
            model,
            data,
            transcript,
            created_at: chrono::Utc::now(),
        }
    }
}
```

### 2.4 Audio Buffer

```rust
// src/audio/buffer.rs

/// In-memory audio representation
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    /// Interleaved samples normalized to [-1.0, 1.0]
    pub samples: Vec<f32>,
    /// Sample rate in Hz (e.g., 24000, 44100)
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
}

impl AudioBuffer {
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        Self { samples, sample_rate, channels }
    }

    /// Duration in seconds
    pub fn duration(&self) -> f32 {
        self.samples.len() as f32 / (self.sample_rate as f32 * self.channels as f32)
    }

    /// Convert to mono by averaging channels
    pub fn to_mono(&self) -> Self {
        if self.channels == 1 {
            return self.clone();
        }

        let mono_samples: Vec<f32> = self.samples
            .chunks(self.channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
            .collect();

        Self {
            samples: mono_samples,
            sample_rate: self.sample_rate,
            channels: 1,
        }
    }

    /// Resample to target sample rate
    pub fn resample(&self, target_rate: u32) -> Result<Self, AudioError> {
        // Use rubato crate for high-quality resampling
        todo!("Implement resampling with rubato")
    }
}
```

## 3. Backend Trait Design

```rust
// src/backend/mod.rs
use crate::audio::AudioBuffer;
use crate::voice::VoiceEmbedding;
use std::path::Path;

/// Trait for TTS model backends
pub trait TTSBackend: Send + Sync {
    /// Extract voice characteristics from reference audio
    fn extract_voice(
        &self,
        audio: &AudioBuffer,
        transcript: &str,
    ) -> Result<VoiceEmbedding, BackendError>;

    /// Generate speech from text using extracted voice
    fn synthesize(
        &self,
        text: &str,
        voice: &VoiceEmbedding,
    ) -> Result<AudioBuffer, BackendError>;

    /// Check if backend is available (dependencies installed)
    fn is_available(&self) -> bool;

    /// Get human-readable backend name
    fn name(&self) -> &'static str;

    /// Get required sample rate for this backend
    fn required_sample_rate(&self) -> u32;
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("Backend not available: {0}")]
    NotAvailable(String),

    #[error("Voice extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("Synthesis failed: {0}")]
    SynthesisFailed(String),

    #[error("Model incompatibility: voice was created with {expected}, but using {actual}")]
    ModelMismatch { expected: String, actual: String },

    #[error("Python error: {0}")]
    PythonError(String),
}
```

## 4. OpenVoice V2 Backend

```rust
// src/backend/openvoice.rs
use super::{TTSBackend, BackendError};
use pyo3::prelude::*;
use pyo3::types::PyModule;

pub struct OpenVoiceBackend {
    // PyO3 Python instance
    initialized: bool,
}

impl OpenVoiceBackend {
    pub fn new() -> Result<Self, BackendError> {
        // Initialize Python and check for OpenVoice
        Python::with_gil(|py| {
            // Try to import openvoice
            match py.import("openvoice") {
                Ok(_) => Ok(Self { initialized: true }),
                Err(e) => Err(BackendError::NotAvailable(
                    format!("OpenVoice not installed: {}", e)
                )),
            }
        })
    }
}

impl TTSBackend for OpenVoiceBackend {
    fn extract_voice(
        &self,
        audio: &AudioBuffer,
        transcript: &str,
    ) -> Result<VoiceEmbedding, BackendError> {
        Python::with_gil(|py| {
            // Load OpenVoice ToneColorConverter
            let openvoice = py.import("openvoice.api")?;
            let tone_color_converter = openvoice.getattr("ToneColorConverter")?;

            // Extract speaker embedding
            // ... implementation details

            todo!("Complete OpenVoice integration")
        })
    }

    fn synthesize(
        &self,
        text: &str,
        voice: &VoiceEmbedding,
    ) -> Result<AudioBuffer, BackendError> {
        // OpenVoice uses a base TTS + tone color conversion approach
        Python::with_gil(|py| {
            // 1. Generate base speech with MeloTTS
            // 2. Apply tone color conversion with extracted embedding

            todo!("Complete synthesis implementation")
        })
    }

    fn is_available(&self) -> bool {
        self.initialized
    }

    fn name(&self) -> &'static str {
        "OpenVoice V2"
    }

    fn required_sample_rate(&self) -> u32 {
        24000
    }
}
```

## 5. OpenF5-TTS Backend

```rust
// src/backend/openf5.rs
use super::{TTSBackend, BackendError};
use pyo3::prelude::*;

pub struct OpenF5Backend {
    initialized: bool,
}

impl OpenF5Backend {
    pub fn new() -> Result<Self, BackendError> {
        Python::with_gil(|py| {
            match py.import("f5_tts") {
                Ok(_) => Ok(Self { initialized: true }),
                Err(e) => Err(BackendError::NotAvailable(
                    format!("F5-TTS not installed: {}", e)
                )),
            }
        })
    }
}

impl TTSBackend for OpenF5Backend {
    fn extract_voice(
        &self,
        audio: &AudioBuffer,
        transcript: &str,
    ) -> Result<VoiceEmbedding, BackendError> {
        // F5-TTS doesn't extract embeddings in the traditional sense
        // Instead, it uses the reference audio directly during synthesis
        // We'll store the audio itself (or a reference to it) as the "embedding"

        todo!("Complete F5-TTS voice extraction")
    }

    fn synthesize(
        &self,
        text: &str,
        voice: &VoiceEmbedding,
    ) -> Result<AudioBuffer, BackendError> {
        Python::with_gil(|py| {
            // F5-TTS flow-matching synthesis
            // Uses reference audio + transcript to condition generation

            todo!("Complete F5-TTS synthesis")
        })
    }

    fn is_available(&self) -> bool {
        self.initialized
    }

    fn name(&self) -> &'static str {
        "OpenF5-TTS"
    }

    fn required_sample_rate(&self) -> u32 {
        24000
    }
}
```

## 6. Voice Manager

```rust
// src/voice/manager.rs
use super::embedding::VoiceEmbedding;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct VoiceManager {
    storage_dir: PathBuf,
    index: VoiceIndex,
}

#[derive(Debug, Serialize, Deserialize)]
struct VoiceIndex {
    voices: HashMap<String, VoiceMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VoiceMetadata {
    model: String,
    created: chrono::DateTime<chrono::Utc>,
    transcript: String,
    file: String,
}

impl VoiceManager {
    pub fn new() -> Result<Self, Error> {
        let storage_dir = dirs::data_dir()
            .ok_or(Error::NoDataDir)?
            .join("open-tts-rs")
            .join("voices");

        std::fs::create_dir_all(&storage_dir)?;

        let index = Self::load_index(&storage_dir)?;

        Ok(Self { storage_dir, index })
    }

    /// Save a voice with the given name
    pub fn save(&mut self, name: &str, voice: &VoiceEmbedding) -> Result<(), Error> {
        let filename = format!("{}.bin", name);
        let filepath = self.storage_dir.join(&filename);

        // Serialize and write embedding
        let data = bincode::serialize(voice)?;
        std::fs::write(&filepath, data)?;

        // Update index
        self.index.voices.insert(name.to_string(), VoiceMetadata {
            model: format!("{:?}", voice.model),
            created: voice.created_at,
            transcript: voice.transcript.clone(),
            file: filename,
        });

        self.save_index()?;
        Ok(())
    }

    /// Load a voice by name
    pub fn load(&self, name: &str) -> Result<VoiceEmbedding, Error> {
        let metadata = self.index.voices.get(name)
            .ok_or_else(|| Error::VoiceNotFound(name.to_string()))?;

        let filepath = self.storage_dir.join(&metadata.file);
        let data = std::fs::read(&filepath)?;
        let voice: VoiceEmbedding = bincode::deserialize(&data)?;

        Ok(voice)
    }

    /// List all saved voices
    pub fn list(&self) -> Vec<(&str, &VoiceMetadata)> {
        self.index.voices.iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect()
    }

    /// Delete a voice by name
    pub fn delete(&mut self, name: &str) -> Result<(), Error> {
        let metadata = self.index.voices.remove(name)
            .ok_or_else(|| Error::VoiceNotFound(name.to_string()))?;

        let filepath = self.storage_dir.join(&metadata.file);
        std::fs::remove_file(filepath)?;

        self.save_index()?;
        Ok(())
    }
}
```

## 7. Main Engine

```rust
// src/core/engine.rs
use crate::backend::{TTSBackend, OpenVoiceBackend, OpenF5Backend};
use crate::voice::{VoiceEmbedding, VoiceManager};
use crate::audio::{AudioBuffer, AudioReader, AudioWriter};
use crate::cli::args::{ModelArg, Reference};

pub struct TTSEngine {
    backend: Box<dyn TTSBackend>,
    voice_manager: VoiceManager,
    current_voice: Option<VoiceEmbedding>,
}

impl TTSEngine {
    pub fn new(model: ModelArg) -> Result<Self, Error> {
        let backend: Box<dyn TTSBackend> = match model {
            ModelArg::Ov => Box::new(OpenVoiceBackend::new()?),
            ModelArg::Of => Box::new(OpenF5Backend::new()?),
        };

        let voice_manager = VoiceManager::new()?;

        Ok(Self {
            backend,
            voice_manager,
            current_voice: None,
        })
    }

    /// Load voice from reference audio
    pub fn load_reference(&mut self, reference: &Reference) -> Result<(), Error> {
        // Read audio file
        let audio = AudioReader::read(&reference.audio_path)?;

        // Resample if needed
        let audio = audio.resample(self.backend.required_sample_rate())?;

        // Extract voice embedding
        let voice = self.backend.extract_voice(&audio, &reference.transcript)?;

        self.current_voice = Some(voice);
        Ok(())
    }

    /// Load a saved voice by name
    pub fn load_named_voice(&mut self, name: &str) -> Result<(), Error> {
        let voice = self.voice_manager.load(name)?;

        // Verify model compatibility
        // ...

        self.current_voice = Some(voice);
        Ok(())
    }

    /// Save current voice with a name
    pub fn save_voice(&mut self, name: &str) -> Result<(), Error> {
        let voice = self.current_voice.as_ref()
            .ok_or(Error::NoVoiceLoaded)?;

        self.voice_manager.save(name, voice)?;
        Ok(())
    }

    /// Generate speech from text
    pub fn generate(&self, text: &str) -> Result<AudioBuffer, Error> {
        let voice = self.current_voice.as_ref()
            .ok_or(Error::NoVoiceLoaded)?;

        let audio = self.backend.synthesize(text, voice)?;
        Ok(audio)
    }
}
```

## 8. Command Dispatch

```rust
// src/main.rs
use clap::Parser;
use open_tts_rs::cli::Args;
use open_tts_rs::core::TTSEngine;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Setup logging
    if args.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("open_tts_rs=debug")
            .init();
    }

    // Handle utility commands
    if args.list_voices {
        return cmd_list_voices();
    }
    if let Some(name) = &args.delete_voice {
        return cmd_delete_voice(name);
    }

    // Main TTS workflow
    let mut engine = TTSEngine::new(args.model)?;

    // Load voice (from reference or saved name)
    if let Some(ref_str) = &args.reference {
        let reference = Reference::parse(ref_str)?;
        engine.load_reference(&reference)?;

        // Save if name provided
        if let Some(name) = &args.name {
            engine.save_voice(name)?;
            println!("Voice saved as '{}'", name);
        }
    } else if let Some(name) = &args.name {
        engine.load_named_voice(name)?;
    }

    // Generate speech
    if let Some(text) = &args.generate {
        let audio = engine.generate(text)?;
        AudioWriter::write(&args.output, &audio)?;
        println!("Generated: {}", args.output.display());
    }

    Ok(())
}
```

## 9. Error Handling

```rust
// src/core/error.rs
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Audio file not found: {0}")]
    AudioNotFound(PathBuf),

    #[error("Invalid reference format. Expected 'file.wav;transcript text'")]
    InvalidReferenceFormat,

    #[error("Empty transcript provided")]
    EmptyTranscript,

    #[error("Voice '{0}' not found. Use --list-voices to see available voices.")]
    VoiceNotFound(String),

    #[error("No voice loaded. Provide --reference or --name first.")]
    NoVoiceLoaded,

    #[error("Cannot use voice created with {0} with model {1}")]
    ModelMismatch(String, String),

    #[error("Backend error: {0}")]
    Backend(#[from] crate::backend::BackendError),

    #[error("Audio error: {0}")]
    Audio(#[from] crate::audio::AudioError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
}
```

## 10. Progress Indication

```rust
// src/cli/progress.rs
use indicatif::{ProgressBar, ProgressStyle};

pub fn extraction_progress() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message("Extracting voice characteristics...");
    pb
}

pub fn synthesis_progress() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap()
    );
    pb.set_message("Generating speech...");
    pb
}
```

## 11. Python Integration Strategy

### Option A: PyO3 (Recommended)

```rust
// Direct Python embedding
use pyo3::prelude::*;

fn call_openvoice(audio_path: &str, transcript: &str) -> PyResult<Vec<u8>> {
    Python::with_gil(|py| {
        let code = r#"
import numpy as np
from openvoice.api import ToneColorConverter
from openvoice import se_extractor

# Extract speaker embedding
se = se_extractor.get_se(audio_path, tone_color_converter, vad=True)
return se.tobytes()
"#;
        let locals = [("audio_path", audio_path)].into_py_dict(py)?;
        py.run(code, None, Some(&locals))?;
        // Extract result...
        Ok(vec![])
    })
}
```

### Option B: Subprocess Fallback

```rust
// Shell out to Python scripts
use std::process::Command;

fn call_openvoice_subprocess(audio_path: &str, transcript: &str) -> Result<Vec<u8>> {
    let output = Command::new("python")
        .args([
            "-m", "open_tts_rs.scripts.extract_voice",
            "--audio", audio_path,
            "--transcript", transcript,
        ])
        .output()?;

    if !output.status.success() {
        return Err(Error::Backend(String::from_utf8_lossy(&output.stderr).into()));
    }

    Ok(output.stdout)
}
```

## 12. Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_parsing() {
        let input = "test.wav;Hello world";
        let ref_ = Reference::parse(input).unwrap();
        assert_eq!(ref_.transcript, "Hello world");
    }

    #[test]
    fn test_reference_parsing_invalid() {
        let input = "no_semicolon";
        assert!(Reference::parse(input).is_err());
    }

    #[test]
    fn test_voice_roundtrip() {
        let voice = VoiceEmbedding::new(
            ModelType::OpenVoiceV2,
            vec![1, 2, 3, 4],
            "test".into(),
        );

        let serialized = bincode::serialize(&voice).unwrap();
        let deserialized: VoiceEmbedding = bincode::deserialize(&serialized).unwrap();

        assert_eq!(voice.model, deserialized.model);
        assert_eq!(voice.data, deserialized.data);
    }
}
```
