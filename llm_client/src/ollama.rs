use anyhow::{bail, Result};
use async_trait::async_trait;
use reqwest::Client;
use shared_types::LlmOutput;

use crate::{client::LlmClient, models::ChatRequest};

#[derive(Debug, Clone)]
pub struct OllamaClient {
    http_client: Client,
    base_url: String,
    model: String,
}

impl OllamaClient {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            http_client: Client::new(),
            base_url: base_url.into(),
            model: model.into(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn model(&self) -> &str {
        &self.model
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn chat(&self, _request: ChatRequest) -> Result<LlmOutput> {
        let _ = &self.http_client;
        bail!("Ollama client not implemented yet")
    }
}
