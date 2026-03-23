use shared_types::{LlmMessage, ToolDefinition};

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub messages: Vec<LlmMessage>,
    pub tools: Vec<ToolDefinition>,
}
