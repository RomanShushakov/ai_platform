use anyhow::Result;
use async_trait::async_trait;

use crate::domain::llm_backend::LlmBackend;

use llm_client::{ChatRequest, LlmClient, OpenAiCompatClient};
use shared_types::LlmOutput;

#[derive(Clone)]
pub struct OpenAiCompatLlmBackend {
    client: OpenAiCompatClient,
}

impl OpenAiCompatLlmBackend {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: OpenAiCompatClient::new(base_url, model),
        }
    }
}

#[async_trait]
impl LlmBackend for OpenAiCompatLlmBackend {
    async fn chat(&self, request: ChatRequest) -> Result<LlmOutput> {
        self.client.chat(request).await
    }

    async fn chat_direct(&self, message: String) -> Result<String> {
        self.client.chat_direct(message).await
    }
}
