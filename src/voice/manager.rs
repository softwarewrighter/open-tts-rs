//! Voice manager for local storage operations.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during voice management.
#[derive(Error, Debug)]
pub enum VoiceError {
    #[error("Voice not found: {0}")]
    NotFound(String),

    #[error("Invalid voice name: {0}")]
    InvalidName(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Metadata for a saved voice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceMetadata {
    pub name: String,
    pub transcript: String,
    pub model: String,
    pub created_at: String,
}

/// Manages local voice storage.
pub struct VoiceManager {
    voices_dir: PathBuf,
}

impl VoiceManager {
    /// Create a new VoiceManager with the default directory.
    pub fn new() -> Self {
        let voices_dir = dirs::home_dir()
            .expect("Could not find home directory")
            .join(".open-tts-rs")
            .join("voices");

        Self { voices_dir }
    }

    /// Create a new VoiceManager with a custom directory.
    pub fn with_dir(voices_dir: PathBuf) -> Self {
        Self { voices_dir }
    }

    /// Get the voices directory path.
    pub fn voices_dir(&self) -> PathBuf {
        self.voices_dir.clone()
    }

    /// Validate a voice name.
    fn validate_name(name: &str) -> Result<(), VoiceError> {
        if name.is_empty() {
            return Err(VoiceError::InvalidName("Name cannot be empty".to_string()));
        }

        // Prevent path traversal
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return Err(VoiceError::InvalidName(
                "Name cannot contain path separators".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the metadata file path for a voice.
    fn metadata_path(&self, name: &str) -> PathBuf {
        self.voices_dir.join(format!("{}.json", name))
    }

    /// Save voice metadata to local storage.
    pub fn save_metadata(&self, metadata: &VoiceMetadata) -> Result<(), VoiceError> {
        Self::validate_name(&metadata.name)?;

        // Ensure directory exists
        std::fs::create_dir_all(&self.voices_dir)?;

        let path = self.metadata_path(&metadata.name);
        let json = serde_json::to_string_pretty(metadata)?;
        std::fs::write(path, json)?;

        Ok(())
    }

    /// Load voice metadata from local storage.
    pub fn load_metadata(&self, name: &str) -> Result<VoiceMetadata, VoiceError> {
        Self::validate_name(name)?;

        let path = self.metadata_path(name);

        if !path.exists() {
            return Err(VoiceError::NotFound(name.to_string()));
        }

        let json = std::fs::read_to_string(path)?;
        let metadata = serde_json::from_str(&json)?;

        Ok(metadata)
    }

    /// Delete voice metadata from local storage.
    pub fn delete_local(&self, name: &str) -> Result<(), VoiceError> {
        Self::validate_name(name)?;

        let path = self.metadata_path(name);

        if !path.exists() {
            return Err(VoiceError::NotFound(name.to_string()));
        }

        std::fs::remove_file(path)?;

        Ok(())
    }

    /// List all locally stored voice metadata.
    pub fn list_local(&self) -> Result<Vec<VoiceMetadata>, VoiceError> {
        if !self.voices_dir.exists() {
            return Ok(Vec::new());
        }

        let mut voices = Vec::new();

        for entry in std::fs::read_dir(&self.voices_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                let json = std::fs::read_to_string(&path)?;
                if let Ok(metadata) = serde_json::from_str::<VoiceMetadata>(&json) {
                    voices.push(metadata);
                }
            }
        }

        Ok(voices)
    }
}

impl Default for VoiceManager {
    fn default() -> Self {
        Self::new()
    }
}
