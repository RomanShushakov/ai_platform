use anyhow::Result;
use async_trait::async_trait;

use llm_client::ChatRequest;
use shared_types::LlmOutput;

#[async_trait]
pub trait LlmBackend: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<LlmOutput>;
}
