//! CLI argument parsing and validation.

mod args;

pub use args::{Args, Model, Reference, ReferenceParseError};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    // ===========================================
    // Reference::parse tests (TDD - RED phase)
    // ===========================================

    #[test]
    fn test_parse_reference_valid() {
        // Create a temp file that exists
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let input = format!("{path};Hello world");

        let result = Reference::parse(&input);

        assert!(result.is_ok());
        let reference = result.unwrap();
        assert_eq!(reference.transcript, "Hello world");
        assert_eq!(reference.audio_path, PathBuf::from(path));
    }

    #[test]
    fn test_parse_reference_missing_semicolon() {
        let input = "audio.wav no semicolon here";
        let result = Reference::parse(input);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ReferenceParseError::InvalidFormat(_)
        ));
    }

    #[test]
    fn test_parse_reference_empty_transcript() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let input = format!("{path};");

        let result = Reference::parse(&input);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ReferenceParseError::EmptyTranscript
        ));
    }

    #[test]
    fn test_parse_reference_file_not_found() {
        let input = "/nonexistent/path/audio.wav;Hello world";
        let result = Reference::parse(input);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ReferenceParseError::FileNotFound(_)
        ));
    }

    #[test]
    fn test_parse_reference_trims_whitespace() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let input = format!("  {path}  ;  Hello world  ");

        let result = Reference::parse(&input);

        assert!(result.is_ok());
        let reference = result.unwrap();
        assert_eq!(reference.transcript, "Hello world");
    }

    #[test]
    fn test_parse_reference_preserves_semicolons_in_transcript() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let input = format!("{path};Hello; world; with; semicolons");

        let result = Reference::parse(&input);

        assert!(result.is_ok());
        let reference = result.unwrap();
        assert_eq!(reference.transcript, "Hello; world; with; semicolons");
    }

    // ===========================================
    // Model enum tests
    // ===========================================

    #[test]
    fn test_model_default_is_openvoice() {
        let model = Model::default();
        assert_eq!(model, Model::OpenVoice);
    }

    #[test]
    fn test_model_display_openvoice() {
        let model = Model::OpenVoice;
        assert_eq!(model.as_str(), "ov");
    }

    #[test]
    fn test_model_display_openf5() {
        let model = Model::OpenF5;
        assert_eq!(model.as_str(), "of");
    }

    #[test]
    fn test_model_port_openvoice() {
        let model = Model::OpenVoice;
        assert_eq!(model.port(), 9280);
    }

    #[test]
    fn test_model_port_openf5() {
        let model = Model::OpenF5;
        assert_eq!(model.port(), 9288);
    }
}
