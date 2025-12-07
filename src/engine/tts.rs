//! TTS Engine implementation.

use std::path::Path;

use chrono::Utc;
use thiserror::Error;

use crate::backend::{Backend, BackendError, HealthResponse, SynthesizeRequest, VoiceInfo};
use crate::voice::{VoiceError, VoiceManager, VoiceMetadata};

/// Errors that can occur during TTS operations.
#[derive(Error, Debug)]
pub enum TTSError {
    #[error("Voice not found: {0}")]
    VoiceNotFound(String),

    #[error("Backend error: {0}")]
    BackendError(#[from] BackendError),

    #[error("Voice management error: {0}")]
    VoiceError(#[from] VoiceError),

    #[error("Audio file not found: {0}")]
    AudioNotFound(String),
}

/// The main TTS engine that orchestrates between components.
pub struct TTSEngine<B: Backend> {
    backend: B,
    voice_manager: VoiceManager,
}

impl<B: Backend> TTSEngine<B> {
    /// Create a new TTS engine.
    pub fn new(backend: B, voice_manager: VoiceManager) -> Self {
        Self {
            backend,
            voice_manager,
        }
    }

    /// Check backend health status.
    pub fn health_check(&self) -> Result<HealthResponse, TTSError> {
        Ok(self.backend.health()?)
    }

    /// Extract voice from reference audio and save it.
    ///
    /// This uploads the voice to the backend and saves metadata locally.
    pub fn extract_voice(
        &self,
        audio_path: &Path,
        transcript: &str,
        name: Option<String>,
    ) -> Result<VoiceInfo, TTSError> {
        // Verify audio file exists
        if !audio_path.exists() {
            return Err(TTSError::AudioNotFound(audio_path.display().to_string()));
        }

        // Extract voice on backend
        let voice_info = self
            .backend
            .extract_voice(audio_path, transcript, name.clone())?;

        // Save metadata locally
        let metadata = VoiceMetadata {
            name: voice_info.name.clone(),
            transcript: voice_info.transcript.clone(),
            model: voice_info.model.clone(),
            created_at: Utc::now().to_rfc3339(),
        };
        self.voice_manager.save_metadata(&metadata)?;

        Ok(voice_info)
    }

    /// Synthesize speech from text.
    ///
    /// If a voice name is provided, it must exist locally or on the backend.
    pub fn synthesize(
        &self,
        text: &str,
        voice_name: Option<String>,
        speed: f32,
    ) -> Result<Vec<u8>, TTSError> {
        // If voice specified, verify it exists locally
        if let Some(ref name) = voice_name
            && self.voice_manager.load_metadata(name).is_err()
        {
            return Err(TTSError::VoiceNotFound(name.clone()));
        }

        let request = SynthesizeRequest {
            text: text.to_string(),
            voice_name,
            speed,
        };

        Ok(self.backend.synthesize(&request)?)
    }

    /// List all available voices from the backend.
    pub fn list_voices(&self) -> Result<Vec<VoiceInfo>, TTSError> {
        let response = self.backend.list_voices()?;
        Ok(response.voices)
    }

    /// Delete a voice from both backend and local storage.
    pub fn delete_voice(&self, name: &str) -> Result<(), TTSError> {
        // Delete from backend
        self.backend.delete_voice(name)?;

        // Delete local metadata (ignore if not found locally)
        let _ = self.voice_manager.delete_local(name);

        Ok(())
    }
}
