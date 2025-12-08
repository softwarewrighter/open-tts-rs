//! HTTP client for backend communication.

use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::cli::Model;

use super::Backend;
use super::types::{BackendError, HealthResponse, SynthesizeRequest, VoiceInfo, VoicesResponse};

/// HTTP-based backend client.
pub struct HttpBackend {
    base_url: String,
    client: reqwest::blocking::Client,
    model: Model,
}

impl HttpBackend {
    /// Create a new HTTP backend client.
    pub fn new(model: Model, host: &str) -> Self {
        let port = model.port();
        let base_url = format!("http://{host}:{port}");

        Self {
            base_url,
            client: reqwest::blocking::Client::new(),
            model,
        }
    }

    /// Get the base URL for this backend.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Upload a file to Gradio backend, returns the server path.
    fn gradio_upload(&self, audio_path: &Path) -> Result<String, BackendError> {
        let url = format!("{}/gradio_api/upload", self.base_url);

        let audio_data = std::fs::read(audio_path)
            .map_err(|_| BackendError::FileNotFound(audio_path.display().to_string()))?;

        let file_name = audio_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav");

        let file_part = reqwest::blocking::multipart::Part::bytes(audio_data)
            .file_name(file_name.to_string())
            .mime_str("audio/wav")
            .map_err(|e| BackendError::RequestFailed(e.to_string()))?;

        let form = reqwest::blocking::multipart::Form::new().part("files", file_part);

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .map_err(|e| BackendError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(BackendError::RequestFailed(format!(
                "Upload failed: {}",
                response.status()
            )));
        }

        let paths: Vec<String> = response
            .json()
            .map_err(|e| BackendError::InvalidResponse(e.to_string()))?;

        paths
            .into_iter()
            .next()
            .ok_or_else(|| BackendError::InvalidResponse("No path returned".to_string()))
    }

    /// Call Gradio generate endpoint and wait for result.
    fn gradio_generate(
        &self,
        text: &str,
        audio_path: Option<&str>,
        transcript: Option<&str>,
    ) -> Result<Vec<u8>, BackendError> {
        let url = format!("{}/gradio_api/call/generate", self.base_url);

        // Build the data array for Gradio
        // Order: [target_text, prompt_audio, prompt_text, cfg, timesteps, normalize]
        let audio_value = match audio_path {
            Some(path) => serde_json::json!({
                "path": path,
                "meta": {"_type": "gradio.FileData"}
            }),
            None => serde_json::Value::Null,
        };

        let body = serde_json::json!({
            "data": [
                text,
                audio_value,
                transcript.unwrap_or(""),
                2.0,  // CFG value
                10,   // Inference timesteps
                false // Text normalization
            ]
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| BackendError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(BackendError::RequestFailed(format!(
                "Generate call failed: {}",
                response.status()
            )));
        }

        #[derive(serde::Deserialize)]
        struct EventResponse {
            event_id: String,
        }

        let event: EventResponse = response
            .json()
            .map_err(|e| BackendError::InvalidResponse(e.to_string()))?;

        // Poll for result with timeout
        let poll_url = format!(
            "{}/gradio_api/call/generate/{}",
            self.base_url, event.event_id
        );
        let mut attempts = 0;
        let max_attempts = 60; // 60 seconds max

        loop {
            thread::sleep(Duration::from_secs(1));
            attempts += 1;

            let poll_response = self
                .client
                .get(&poll_url)
                .send()
                .map_err(|e| BackendError::ConnectionFailed(e.to_string()))?;

            let text = poll_response
                .text()
                .map_err(|e| BackendError::InvalidResponse(e.to_string()))?;

            // Parse SSE response
            if text.contains("event: complete") {
                // Extract the data line
                for line in text.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        let parsed: serde_json::Value = serde_json::from_str(data)
                            .map_err(|e| BackendError::InvalidResponse(e.to_string()))?;

                        if let Some(url) = parsed
                            .as_array()
                            .and_then(|a| a.first())
                            .and_then(|v| v.get("url"))
                            .and_then(|u| u.as_str())
                        {
                            // Download the audio file
                            return self.download_audio(url);
                        }
                    }
                }
                return Err(BackendError::InvalidResponse(
                    "No audio URL in response".to_string(),
                ));
            }

            if text.contains("event: error") {
                return Err(BackendError::BackendError("Generation failed".to_string()));
            }

            if attempts >= max_attempts {
                return Err(BackendError::RequestFailed(
                    "Generation timed out".to_string(),
                ));
            }
        }
    }

    /// Download audio from URL.
    fn download_audio(&self, url: &str) -> Result<Vec<u8>, BackendError> {
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|e| BackendError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(BackendError::RequestFailed(format!(
                "Download failed: {}",
                response.status()
            )));
        }

        response
            .bytes()
            .map(|b| b.to_vec())
            .map_err(|e| BackendError::InvalidResponse(e.to_string()))
    }
}

impl Backend for HttpBackend {
    fn health(&self) -> Result<HealthResponse, BackendError> {
        if self.model.is_gradio() {
            // For Gradio backends, check /config endpoint
            let url = format!("{}/config", self.base_url);
            let response = self
                .client
                .get(&url)
                .send()
                .map_err(|e| BackendError::ConnectionFailed(e.to_string()))?;

            if response.status().is_success() {
                return Ok(HealthResponse {
                    status: "healthy".to_string(),
                    model: self.model.name().to_string(),
                    cuda_available: true,
                    gpu: None,
                    device: "cuda".to_string(),
                });
            }
            return Err(BackendError::RequestFailed(format!(
                "Status: {}",
                response.status()
            )));
        }

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
        if self.model.is_gradio() {
            // For Gradio backends, just verify the file exists
            // Voice cloning happens at synthesis time
            if !audio_path.exists() {
                return Err(BackendError::FileNotFound(audio_path.display().to_string()));
            }

            return Ok(VoiceInfo {
                name: name.unwrap_or_else(|| "default".to_string()),
                transcript: transcript.to_string(),
                model: self.model.name().to_string(),
                duration: None,
            });
        }

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
        if self.model.is_gradio() {
            // For Gradio backends, upload reference audio and generate
            let server_path = match &request.reference_audio {
                Some(path) => Some(self.gradio_upload(path)?),
                None => None,
            };

            return self.gradio_generate(
                &request.text,
                server_path.as_deref(),
                request.reference_transcript.as_deref(),
            );
        }

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
        if self.model.is_gradio() {
            // Gradio backends don't persist voices
            return Ok(VoicesResponse { voices: vec![] });
        }

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
        if self.model.is_gradio() {
            // Gradio backends don't persist voices
            return Err(BackendError::VoiceNotFound(name.to_string()));
        }

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
