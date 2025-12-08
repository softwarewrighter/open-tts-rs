//! TTS Engine orchestrator.
//!
//! This module provides the main engine that coordinates between
//! the CLI, VoiceManager, and Backend to perform TTS operations.

mod tts;

pub use tts::{TTSEngine, TTSError};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{BackendError, HealthResponse, MockBackend, VoiceInfo, VoicesResponse};
    use crate::voice::{VoiceManager, VoiceMetadata};
    use tempfile::TempDir;

    // ===========================================
    // TTSEngine tests (TDD)
    // ===========================================

    #[test]
    fn test_engine_health_check_success() {
        let temp_dir = TempDir::new().unwrap();
        let voice_manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());
        let mut mock_backend = MockBackend::new();

        mock_backend.expect_health().times(1).returning(|| {
            Ok(HealthResponse {
                status: "healthy".to_string(),
                model: "openvoice_v2".to_string(),
                cuda_available: true,
                gpu: Some("NVIDIA RTX 5060".to_string()),
                device: "cuda:0".to_string(),
            })
        });

        let engine = TTSEngine::new(mock_backend, voice_manager);
        let result = engine.health_check();

        assert!(result.is_ok());
        let health = result.unwrap();
        assert_eq!(health.status, "healthy");
    }

    #[test]
    fn test_engine_health_check_failure() {
        let temp_dir = TempDir::new().unwrap();
        let voice_manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());
        let mut mock_backend = MockBackend::new();

        mock_backend.expect_health().times(1).returning(|| {
            Err(BackendError::ConnectionFailed(
                "Connection refused".to_string(),
            ))
        });

        let engine = TTSEngine::new(mock_backend, voice_manager);
        let result = engine.health_check();

        assert!(result.is_err());
    }

    #[test]
    fn test_engine_extract_voice_and_save() {
        let temp_dir = TempDir::new().unwrap();
        let voice_manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());
        let mut mock_backend = MockBackend::new();

        // Create a test audio file
        let audio_path = temp_dir.path().join("test.wav");
        std::fs::write(&audio_path, b"RIFF fake wav data").unwrap();

        mock_backend
            .expect_extract_voice()
            .times(1)
            .returning(|_, _, _| {
                Ok(VoiceInfo {
                    name: "my_voice".to_string(),
                    transcript: "Hello world".to_string(),
                    model: "openvoice_v2".to_string(),
                    duration: Some(3.5),
                })
            });

        let engine = TTSEngine::new(mock_backend, voice_manager);
        let result = engine.extract_voice(&audio_path, "Hello world", Some("my_voice".to_string()));

        assert!(result.is_ok());
        let voice = result.unwrap();
        assert_eq!(voice.name, "my_voice");

        // Verify metadata was saved locally
        let manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());
        let metadata = manager.load_metadata("my_voice").unwrap();
        assert_eq!(metadata.transcript, "Hello world");
    }

    #[test]
    fn test_engine_synthesize_with_voice() {
        let temp_dir = TempDir::new().unwrap();
        let voice_manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());
        let mut mock_backend = MockBackend::new();

        // Save voice metadata first
        let metadata = VoiceMetadata {
            name: "test_voice".to_string(),
            transcript: "Reference transcript".to_string(),
            model: "openvoice_v2".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            audio_path: None,
        };
        voice_manager.save_metadata(&metadata).unwrap();

        mock_backend
            .expect_synthesize()
            .times(1)
            .returning(|_| Ok(b"RIFF wav audio data".to_vec()));

        let engine = TTSEngine::new(mock_backend, voice_manager);
        let result = engine.synthesize("Generate this text", Some("test_voice".to_string()), 1.0);

        assert!(result.is_ok());
        let audio = result.unwrap();
        assert!(audio.starts_with(b"RIFF"));
    }

    #[test]
    fn test_engine_synthesize_voice_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let voice_manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());
        let mock_backend = MockBackend::new();

        let engine = TTSEngine::new(mock_backend, voice_manager);
        let result = engine.synthesize("Generate this text", Some("nonexistent".to_string()), 1.0);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TTSError::VoiceNotFound(_)));
    }

    #[test]
    fn test_engine_list_voices() {
        let temp_dir = TempDir::new().unwrap();
        let voice_manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());
        let mut mock_backend = MockBackend::new();

        // Save local voice
        let metadata = VoiceMetadata {
            name: "local_voice".to_string(),
            transcript: "Local".to_string(),
            model: "openvoice_v2".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            audio_path: None,
        };
        voice_manager.save_metadata(&metadata).unwrap();

        mock_backend.expect_list_voices().times(1).returning(|| {
            Ok(VoicesResponse {
                voices: vec![VoiceInfo {
                    name: "backend_voice".to_string(),
                    transcript: "Backend".to_string(),
                    model: "openvoice_v2".to_string(),
                    duration: Some(2.0),
                }],
            })
        });

        let engine = TTSEngine::new(mock_backend, voice_manager);
        let result = engine.list_voices();

        assert!(result.is_ok());
        let voices = result.unwrap();
        // Should include backend voice
        assert!(voices.iter().any(|v| v.name == "backend_voice"));
    }

    #[test]
    fn test_engine_delete_voice() {
        let temp_dir = TempDir::new().unwrap();
        let voice_manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());
        let mut mock_backend = MockBackend::new();

        // Save local voice
        let metadata = VoiceMetadata {
            name: "to_delete".to_string(),
            transcript: "Delete me".to_string(),
            model: "openvoice_v2".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            audio_path: None,
        };
        voice_manager.save_metadata(&metadata).unwrap();

        mock_backend
            .expect_delete_voice()
            .times(1)
            .returning(|_| Ok(()));

        let engine = TTSEngine::new(mock_backend, voice_manager);
        let result = engine.delete_voice("to_delete");

        assert!(result.is_ok());

        // Verify local metadata was also deleted
        let manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());
        assert!(manager.load_metadata("to_delete").is_err());
    }

    #[test]
    fn test_engine_synthesize_default_voice() {
        let temp_dir = TempDir::new().unwrap();
        let voice_manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());
        let mut mock_backend = MockBackend::new();

        mock_backend
            .expect_synthesize()
            .times(1)
            .returning(|_| Ok(b"RIFF wav audio data".to_vec()));

        let engine = TTSEngine::new(mock_backend, voice_manager);
        // No voice specified - should use default/last voice
        let result = engine.synthesize("Generate this text", None, 1.0);

        assert!(result.is_ok());
    }
}
