//! Backend request/response types.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur when communicating with the backend.
#[derive(Error, Debug)]
pub enum BackendError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Voice not found: {0}")]
    VoiceNotFound(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Backend error: {0}")]
    BackendError(String),
}

/// Health check response from backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub model: String,
    pub cuda_available: bool,
    pub gpu: Option<String>,
    pub device: String,
}

/// Voice information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    pub name: String,
    pub transcript: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,
}

/// Response from list voices endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicesResponse {
    pub voices: Vec<VoiceInfo>,
}

/// Request for speech synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizeRequest {
    pub text: String,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub voice_name: Option<String>,
    #[serde(default = "default_speed")]
    pub speed: f32,
    /// Reference audio path (for Gradio backends like VoxCPM)
    #[serde(skip)]
    pub reference_audio: Option<std::path::PathBuf>,
    /// Reference transcript (for Gradio backends like VoxCPM)
    #[serde(skip)]
    pub reference_transcript: Option<String>,
}

fn default_speed() -> f32 {
    1.0
}

impl SynthesizeRequest {
    /// Create a new synthesis request.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            voice_name: None,
            speed: 1.0,
            reference_audio: None,
            reference_transcript: None,
        }
    }

    /// Set the voice name.
    pub fn with_voice(mut self, name: impl Into<String>) -> Self {
        self.voice_name = Some(name.into());
        self
    }

    /// Set the speech speed.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Set reference audio path (for Gradio backends).
    pub fn with_reference_audio(mut self, path: std::path::PathBuf) -> Self {
        self.reference_audio = Some(path);
        self
    }

    /// Set reference transcript (for Gradio backends).
    pub fn with_reference_transcript(mut self, transcript: impl Into<String>) -> Self {
        self.reference_transcript = Some(transcript.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesize_request_builder() {
        let request = SynthesizeRequest::new("Hello world")
            .with_voice("my_voice")
            .with_speed(1.5);

        assert_eq!(request.text, "Hello world");
        assert_eq!(request.voice_name, Some("my_voice".to_string()));
        assert_eq!(request.speed, 1.5);
    }

    #[test]
    fn test_synthesize_request_defaults() {
        let request = SynthesizeRequest::new("Hello");

        assert_eq!(request.text, "Hello");
        assert_eq!(request.voice_name, None);
        assert_eq!(request.speed, 1.0);
    }

    #[test]
    fn test_health_response_deserialize() {
        let json = r#"{
            "status": "healthy",
            "model": "openvoice_v2",
            "cuda_available": true,
            "gpu": "NVIDIA RTX 5060",
            "device": "cuda:0"
        }"#;

        let response: HealthResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "healthy");
        assert!(response.cuda_available);
        assert_eq!(response.gpu, Some("NVIDIA RTX 5060".to_string()));
    }

    #[test]
    fn test_voices_response_deserialize() {
        let json = r#"{
            "voices": [
                {"name": "voice1", "transcript": "Hello", "model": "ov"},
                {"name": "voice2", "transcript": "World", "model": "of", "duration": 5.5}
            ]
        }"#;

        let response: VoicesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.voices.len(), 2);
        assert_eq!(response.voices[1].duration, Some(5.5));
    }
}
