use anyhow::{Result, bail};
use async_trait::async_trait;

use crate::domain::llm_backend::LlmBackend;

use llm_client::ChatRequest;
use shared_types::LlmOutput;

#[derive(Clone)]
pub struct VllmLlmBackend {
    base_url: String,
    model: String,
}

impl VllmLlmBackend {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
        }
    }
}

#[async_trait]
impl LlmBackend for VllmLlmBackend {
    async fn chat(&self, _request: ChatRequest) -> Result<LlmOutput> {
        let _ = (&self.base_url, &self.model);
        bail!("vLLM backend is not implemented yet")
    }
}
