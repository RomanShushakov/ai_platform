use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use shared_types::{ToolDefinition, ToolResult};

#[async_trait]
pub trait ToolProvider: Send + Sync {
    async fn list_tools(&self) -> Result<Vec<ToolDefinition>>;

    async fn call_tool(&self, name: String, arguments: Value) -> Result<ToolResult>;
}
