use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use shared_types::{ToolDefinition, ToolResult};

use crate::adapters::tools_client::ToolsClient;
use crate::domain::tool_provider::ToolProvider;

#[derive(Clone)]
pub struct HttpToolProvider {
    client: ToolsClient,
}

impl HttpToolProvider {
    pub fn new(client: ToolsClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ToolProvider for HttpToolProvider {
    async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        self.client.list_tools().await
    }

    async fn call_tool(&self, name: String, arguments: Value) -> Result<ToolResult> {
        self.client.call_tool(name, arguments).await
    }
}
