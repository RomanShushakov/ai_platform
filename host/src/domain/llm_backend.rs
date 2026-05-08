use anyhow::Result;
use async_trait::async_trait;

use llm_client::ChatRequest;
use shared_types::LlmOutput;

#[async_trait]
pub trait LlmBackend: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<LlmOutput>;

    async fn chat_direct(&self, message: String) -> Result<String> {
        let output = self
            .chat(ChatRequest {
                messages: vec![shared_types::LlmMessage::User { content: message }],
                tools: Vec::new(),
            })
            .await?;

        match output {
            LlmOutput::FinalText { text } => Ok(text),
            LlmOutput::ToolCall { name, .. } => {
                anyhow::bail!("direct chat received unexpected tool call: {}", name)
            }
        }
    }
}
