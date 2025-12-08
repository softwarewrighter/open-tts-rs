//! Voice management for local voice storage and backend coordination.
//!
//! This module handles saving, loading, and managing voice references
//! that are synchronized with the TTS backend servers.

mod manager;

pub use manager::{VoiceError, VoiceManager, VoiceMetadata};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // ===========================================
    // VoiceManager tests (TDD - RED phase)
    // ===========================================

    #[test]
    fn test_voice_manager_default_directory() {
        let manager = VoiceManager::new();
        let expected = dirs::home_dir()
            .unwrap()
            .join(".open-tts-rs")
            .join("voices");
        assert_eq!(manager.voices_dir(), expected);
    }

    #[test]
    fn test_voice_manager_custom_directory() {
        let custom_path = PathBuf::from("/tmp/custom-voices");
        let manager = VoiceManager::with_dir(custom_path.clone());
        assert_eq!(manager.voices_dir(), custom_path);
    }

    #[test]
    fn test_voice_manager_list_empty() {
        let temp_dir = TempDir::new().unwrap();
        let manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());

        let voices = manager.list_local().unwrap();
        assert!(voices.is_empty());
    }

    #[test]
    fn test_voice_manager_save_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());

        let metadata = VoiceMetadata {
            name: "test_voice".to_string(),
            transcript: "Hello world".to_string(),
            model: "openvoice_v2".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            audio_path: None,
        };

        manager.save_metadata(&metadata).unwrap();

        let loaded = manager.load_metadata("test_voice").unwrap();
        assert_eq!(loaded.name, "test_voice");
        assert_eq!(loaded.transcript, "Hello world");
    }

    #[test]
    fn test_voice_manager_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());

        let result = manager.load_metadata("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_voice_manager_delete_voice() {
        let temp_dir = TempDir::new().unwrap();
        let manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());

        let metadata = VoiceMetadata {
            name: "to_delete".to_string(),
            transcript: "Delete me".to_string(),
            model: "openvoice_v2".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            audio_path: None,
        };

        manager.save_metadata(&metadata).unwrap();
        assert!(manager.load_metadata("to_delete").is_ok());

        manager.delete_local("to_delete").unwrap();
        assert!(manager.load_metadata("to_delete").is_err());
    }

    #[test]
    fn test_voice_manager_list_after_save() {
        let temp_dir = TempDir::new().unwrap();
        let manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());

        let metadata1 = VoiceMetadata {
            name: "voice_a".to_string(),
            transcript: "First".to_string(),
            model: "openvoice_v2".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            audio_path: None,
        };

        let metadata2 = VoiceMetadata {
            name: "voice_b".to_string(),
            transcript: "Second".to_string(),
            model: "openf5_tts".to_string(),
            created_at: "2024-01-02T00:00:00Z".to_string(),
            audio_path: None,
        };

        manager.save_metadata(&metadata1).unwrap();
        manager.save_metadata(&metadata2).unwrap();

        let voices = manager.list_local().unwrap();
        assert_eq!(voices.len(), 2);
        assert!(voices.iter().any(|v| v.name == "voice_a"));
        assert!(voices.iter().any(|v| v.name == "voice_b"));
    }

    #[test]
    fn test_voice_manager_validates_name() {
        let temp_dir = TempDir::new().unwrap();
        let manager = VoiceManager::with_dir(temp_dir.path().to_path_buf());

        // Invalid names with path separators
        let metadata = VoiceMetadata {
            name: "../evil".to_string(),
            transcript: "Malicious".to_string(),
            model: "openvoice_v2".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            audio_path: None,
        };

        let result = manager.save_metadata(&metadata);
        assert!(result.is_err());
    }
}
