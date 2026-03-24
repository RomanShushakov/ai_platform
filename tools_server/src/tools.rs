pub mod search_docs;
pub mod weather;

use anyhow::{Result, bail};
use shared_types::{ToolCallRequest, ToolDefinition, ToolResult};

pub fn all_definitions() -> Vec<ToolDefinition> {
    vec![weather::definition(), search_docs::definition()]
}

pub async fn execute(request: ToolCallRequest) -> Result<ToolResult> {
    match request.name.as_str() {
        "get_weather" => weather::execute(request.arguments).await,
        "search_docs" => search_docs::execute(request.arguments).await,
        other => bail!("unknown tool: {}", other),
    }
}
