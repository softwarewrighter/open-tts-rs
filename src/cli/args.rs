//! CLI argument definitions and parsing.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use thiserror::Error;

/// Voice cloning and text-to-speech CLI.
#[derive(Parser, Debug)]
#[command(name = "open-tts-rs")]
#[command(about = "Voice cloning and text-to-speech using open-source models")]
#[command(version)]
pub struct Args {
    /// TTS model to use: "ov" (OpenVoice V2) or "of" (OpenF5-TTS)
    #[arg(short, long, value_enum, default_value = "ov")]
    pub model: Model,

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

    /// Backend host address
    #[arg(long, default_value = "localhost")]
    pub host: String,

    /// Speech speed multiplier (0.5 to 2.0)
    #[arg(short, long, default_value = "1.0")]
    pub speed: f32,
}

/// TTS model selection.
#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq)]
pub enum Model {
    /// OpenVoice V2 (MIT license, fast)
    #[default]
    #[value(name = "ov")]
    OpenVoice,

    /// OpenF5-TTS (Apache 2.0, atmospheric cloning)
    #[value(name = "of")]
    OpenF5,
}

impl Model {
    /// Returns the CLI argument string for this model.
    pub fn as_str(&self) -> &'static str {
        match self {
            Model::OpenVoice => "ov",
            Model::OpenF5 => "of",
        }
    }

    /// Returns the backend server port for this model.
    pub fn port(&self) -> u16 {
        match self {
            Model::OpenVoice => 9280,
            Model::OpenF5 => 9288,
        }
    }

    /// Returns the human-readable name of the model.
    pub fn name(&self) -> &'static str {
        match self {
            Model::OpenVoice => "OpenVoice V2",
            Model::OpenF5 => "OpenF5-TTS",
        }
    }
}

/// Parsed reference audio with transcript.
#[derive(Debug, Clone)]
pub struct Reference {
    /// Path to the audio file.
    pub audio_path: PathBuf,
    /// Transcript of the audio content.
    pub transcript: String,
}

/// Errors that can occur when parsing a reference string.
#[derive(Error, Debug)]
pub enum ReferenceParseError {
    #[error("Invalid format: {0}. Expected 'file.wav;transcript text'")]
    InvalidFormat(String),

    #[error("Audio file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Transcript cannot be empty")]
    EmptyTranscript,
}

impl Reference {
    /// Parse a reference from "file.wav;transcript" format.
    ///
    /// # Arguments
    /// * `input` - String in format "path/to/audio.wav;transcript text"
    ///
    /// # Returns
    /// * `Ok(Reference)` if parsing succeeds
    /// * `Err(ReferenceParseError)` if parsing fails
    ///
    /// # Examples
    /// ```
    /// use open_tts_rs::cli::Reference;
    /// let reference = Reference::parse("audio.wav;Hello world");
    /// ```
    pub fn parse(input: &str) -> Result<Self, ReferenceParseError> {
        // Split on first semicolon only (transcript may contain semicolons)
        let parts: Vec<&str> = input.splitn(2, ';').collect();

        if parts.len() != 2 {
            return Err(ReferenceParseError::InvalidFormat(
                "Missing semicolon separator".to_string(),
            ));
        }

        let audio_path = PathBuf::from(parts[0].trim());
        let transcript = parts[1].trim().to_string();

        // Validate file exists
        if !audio_path.exists() {
            return Err(ReferenceParseError::FileNotFound(audio_path));
        }

        // Validate transcript is not empty
        if transcript.is_empty() {
            return Err(ReferenceParseError::EmptyTranscript);
        }

        Ok(Self {
            audio_path,
            transcript,
        })
    }
}
