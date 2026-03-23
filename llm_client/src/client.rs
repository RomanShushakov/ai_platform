use anyhow::Result;
use async_trait::async_trait;
use shared_types::{LlmMessage, LlmOutput, ToolDefinition};

use crate::models::ChatRequest;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<LlmOutput>;

    async fn chat_with_messages(
        &self,
        messages: Vec<LlmMessage>,
        tools: Vec<ToolDefinition>,
    ) -> Result<LlmOutput> {
        let request = ChatRequest { messages, tools };
        self.chat(request).await
    }
}
