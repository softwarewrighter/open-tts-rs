//! Backend communication with TTS model servers.
//!
//! Provides traits and implementations for communicating with the
//! Docker-based TTS backends (OpenVoice V2 and OpenF5-TTS).

mod client;
mod types;

pub use client::HttpBackend;
pub use types::{BackendError, HealthResponse, SynthesizeRequest, VoiceInfo, VoicesResponse};

use crate::cli::Model;

/// Trait for TTS backend communication.
///
/// This trait abstracts the HTTP communication with the TTS servers,
/// allowing for mock implementations in tests.
#[cfg_attr(test, mockall::automock)]
pub trait Backend: Send + Sync {
    /// Check backend health status.
    fn health(&self) -> Result<HealthResponse, BackendError>;

    /// Extract voice from reference audio.
    ///
    /// # Arguments
    /// * `audio_path` - Path to the reference audio file
    /// * `transcript` - Transcript of the audio content
    /// * `name` - Optional name to save the voice as
    fn extract_voice(
        &self,
        audio_path: &std::path::Path,
        transcript: &str,
        name: Option<String>,
    ) -> Result<VoiceInfo, BackendError>;

    /// Synthesize speech from text.
    ///
    /// # Arguments
    /// * `request` - Synthesis request parameters
    ///
    /// # Returns
    /// Raw WAV audio data
    fn synthesize(&self, request: &SynthesizeRequest) -> Result<Vec<u8>, BackendError>;

    /// List all saved voices.
    fn list_voices(&self) -> Result<VoicesResponse, BackendError>;

    /// Delete a saved voice.
    fn delete_voice(&self, name: &str) -> Result<(), BackendError>;
}

/// Create a backend for the specified model.
pub fn create_backend(model: Model, host: &str) -> HttpBackend {
    HttpBackend::new(model, host)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ===========================================
    // Backend trait tests with mocks (TDD - RED phase)
    // ===========================================

    #[test]
    fn test_mock_backend_health_success() {
        let mut mock = MockBackend::new();

        mock.expect_health().times(1).returning(|| {
            Ok(HealthResponse {
                status: "healthy".to_string(),
                model: "openvoice_v2".to_string(),
                cuda_available: true,
                gpu: Some("NVIDIA RTX 5060".to_string()),
                device: "cuda:0".to_string(),
            })
        });

        let result = mock.health();
        assert!(result.is_ok());

        let health = result.unwrap();
        assert_eq!(health.status, "healthy");
        assert!(health.cuda_available);
    }

    #[test]
    fn test_mock_backend_health_failure() {
        let mut mock = MockBackend::new();

        mock.expect_health().times(1).returning(|| {
            Err(BackendError::ConnectionFailed(
                "Connection refused".to_string(),
            ))
        });

        let result = mock.health();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BackendError::ConnectionFailed(_)
        ));
    }

    #[test]
    fn test_mock_backend_list_voices() {
        let mut mock = MockBackend::new();

        mock.expect_list_voices().times(1).returning(|| {
            Ok(VoicesResponse {
                voices: vec![
                    VoiceInfo {
                        name: "test_voice".to_string(),
                        transcript: "Hello world".to_string(),
                        model: "openvoice_v2".to_string(),
                        duration: None,
                    },
                    VoiceInfo {
                        name: "another_voice".to_string(),
                        transcript: "Another sample".to_string(),
                        model: "openf5_tts".to_string(),
                        duration: Some(5.2),
                    },
                ],
            })
        });

        let result = mock.list_voices();
        assert!(result.is_ok());

        let voices = result.unwrap();
        assert_eq!(voices.voices.len(), 2);
        assert_eq!(voices.voices[0].name, "test_voice");
    }

    #[test]
    fn test_mock_backend_extract_voice() {
        let mut mock = MockBackend::new();

        mock.expect_extract_voice()
            .withf(|path, transcript, name| {
                path == PathBuf::from("/tmp/test.wav").as_path()
                    && transcript == "Hello world"
                    && name.as_deref() == Some("my_voice")
            })
            .times(1)
            .returning(|_, _, _| {
                Ok(VoiceInfo {
                    name: "my_voice".to_string(),
                    transcript: "Hello world".to_string(),
                    model: "openvoice_v2".to_string(),
                    duration: Some(3.5),
                })
            });

        let result = mock.extract_voice(
            PathBuf::from("/tmp/test.wav").as_path(),
            "Hello world",
            Some("my_voice".to_string()),
        );

        assert!(result.is_ok());
        let voice = result.unwrap();
        assert_eq!(voice.name, "my_voice");
    }

    #[test]
    fn test_mock_backend_synthesize() {
        let mut mock = MockBackend::new();

        mock.expect_synthesize()
            .withf(|req| {
                req.text == "Hello world" && req.voice_name == Some("my_voice".to_string())
            })
            .times(1)
            .returning(|_| {
                // Return fake WAV data (RIFF header)
                Ok(b"RIFF\x00\x00\x00\x00WAVEfmt ".to_vec())
            });

        let request = SynthesizeRequest {
            text: "Hello world".to_string(),
            voice_name: Some("my_voice".to_string()),
            speed: 1.0,
            reference_audio: None,
            reference_transcript: None,
        };

        let result = mock.synthesize(&request);
        assert!(result.is_ok());

        let audio = result.unwrap();
        assert!(audio.starts_with(b"RIFF"));
    }

    #[test]
    fn test_mock_backend_delete_voice() {
        let mut mock = MockBackend::new();

        mock.expect_delete_voice()
            .with(mockall::predicate::eq("old_voice"))
            .times(1)
            .returning(|_| Ok(()));

        let result = mock.delete_voice("old_voice");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_backend_delete_voice_not_found() {
        let mut mock = MockBackend::new();

        mock.expect_delete_voice()
            .with(mockall::predicate::eq("nonexistent"))
            .times(1)
            .returning(|_| Err(BackendError::VoiceNotFound("nonexistent".to_string())));

        let result = mock.delete_voice("nonexistent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BackendError::VoiceNotFound(_)
        ));
    }

    // ===========================================
    // Model-to-backend mapping tests
    // ===========================================

    #[test]
    fn test_create_backend_openvoice() {
        let backend = create_backend(Model::OpenVoice, "localhost");
        assert_eq!(backend.base_url(), "http://localhost:9280");
    }

    #[test]
    fn test_create_backend_openf5() {
        let backend = create_backend(Model::OpenF5, "localhost");
        assert_eq!(backend.base_url(), "http://localhost:9288");
    }
}
