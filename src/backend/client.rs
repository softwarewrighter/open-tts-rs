//! HTTP client for backend communication.

use std::path::Path;

use crate::cli::Model;

use super::Backend;
use super::types::{BackendError, HealthResponse, SynthesizeRequest, VoiceInfo, VoicesResponse};

/// HTTP-based backend client.
pub struct HttpBackend {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl HttpBackend {
    /// Create a new HTTP backend client.
    pub fn new(model: Model, host: &str) -> Self {
        let port = model.port();
        let base_url = format!("http://{host}:{port}");

        Self {
            base_url,
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Get the base URL for this backend.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Backend for HttpBackend {
    fn health(&self) -> Result<HealthResponse, BackendError> {
        let url = format!("{}/health", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| BackendError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(BackendError::RequestFailed(format!(
                "Status: {}",
                response.status()
            )));
        }

        response
            .json()
            .map_err(|e| BackendError::InvalidResponse(e.to_string()))
    }

    fn extract_voice(
        &self,
        audio_path: &Path,
        transcript: &str,
        name: Option<String>,
    ) -> Result<VoiceInfo, BackendError> {
        let url = format!("{}/extract_voice", self.base_url);

        // Read audio file
        let audio_data = std::fs::read(audio_path)
            .map_err(|_| BackendError::FileNotFound(audio_path.display().to_string()))?;

        let file_name = audio_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav");

        // Build multipart form
        let file_part = reqwest::blocking::multipart::Part::bytes(audio_data)
            .file_name(file_name.to_string())
            .mime_str("audio/wav")
            .map_err(|e| BackendError::RequestFailed(e.to_string()))?;

        let mut form = reqwest::blocking::multipart::Form::new()
            .part("audio", file_part)
            .text("transcript", transcript.to_string());

        if let Some(n) = name {
            form = form.text("name", n);
        }

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .map_err(|e| BackendError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(BackendError::RequestFailed(format!(
                "Status: {}",
                response.status()
            )));
        }

        response
            .json()
            .map_err(|e| BackendError::InvalidResponse(e.to_string()))
    }

    fn synthesize(&self, request: &SynthesizeRequest) -> Result<Vec<u8>, BackendError> {
        let url = format!("{}/synthesize", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(request)
            .send()
            .map_err(|e| BackendError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(BackendError::RequestFailed(format!(
                "Status: {}",
                response.status()
            )));
        }

        response
            .bytes()
            .map(|b| b.to_vec())
            .map_err(|e| BackendError::InvalidResponse(e.to_string()))
    }

    fn list_voices(&self) -> Result<VoicesResponse, BackendError> {
        let url = format!("{}/voices", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| BackendError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(BackendError::RequestFailed(format!(
                "Status: {}",
                response.status()
            )));
        }

        response
            .json()
            .map_err(|e| BackendError::InvalidResponse(e.to_string()))
    }

    fn delete_voice(&self, name: &str) -> Result<(), BackendError> {
        let url = format!("{}/voices/{name}", self.base_url);

        let response = self
            .client
            .delete(&url)
            .send()
            .map_err(|e| BackendError::ConnectionFailed(e.to_string()))?;

        if response.status().as_u16() == 404 {
            return Err(BackendError::VoiceNotFound(name.to_string()));
        }

        if !response.status().is_success() {
            return Err(BackendError::RequestFailed(format!(
                "Status: {}",
                response.status()
            )));
        }

        Ok(())
    }
}
