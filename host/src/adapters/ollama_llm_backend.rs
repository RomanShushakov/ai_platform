use anyhow::Result;
use async_trait::async_trait;

use crate::domain::llm_backend::LlmBackend;

use llm_client::{ChatRequest, LlmClient, OllamaClient};
use shared_types::LlmOutput;

#[derive(Clone)]
pub struct OllamaLlmBackend {
    client: OllamaClient,
}

impl OllamaLlmBackend {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: OllamaClient::new(base_url, model),
        }
    }
}

#[async_trait]
impl LlmBackend for OllamaLlmBackend {
    async fn chat(&self, request: ChatRequest) -> Result<LlmOutput> {
        self.client.chat(request).await
    }
}
