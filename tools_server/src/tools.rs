pub mod weather;

use anyhow::{Result, bail};
use shared_types::{ToolCallRequest, ToolDefinition, ToolResult};

pub fn all_definitions() -> Vec<ToolDefinition> {
    vec![
        weather::definition(),
    ]
}

pub async fn execute(request: ToolCallRequest) -> Result<ToolResult> {
    match request.name.as_str() {
        "get_weather" => weather::execute(request.arguments).await,
        other => bail!("unknown tool: {}", other),
    }
}
