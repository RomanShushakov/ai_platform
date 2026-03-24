use anyhow::Result;
use async_trait::async_trait;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParams,
    service::{RoleClient, RunningService},
    transport::TokioChildProcess,
};
use serde_json::{Map, Value};
use tokio::process::Command;

use shared_types::{ToolDefinition, ToolResult};

use crate::domain::tool_provider::ToolProvider;

pub struct McpToolProvider {
    client: RunningService<RoleClient, ()>,
}

impl McpToolProvider {
    pub async fn new(binary_path: &str) -> Result<Self> {
        let mut cmd = Command::new(binary_path);
        cmd.env("TOOLS_TRANSPORT", "mcp-stdio");

        let client = ().serve(TokioChildProcess::new(cmd)?).await?;

        Ok(Self { client })
    }
}

#[async_trait]
impl ToolProvider for McpToolProvider {
    async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        let result = self.client.list_all_tools().await?;

        Ok(result
            .into_iter()
            .map(|t| ToolDefinition {
                name: t.name.to_string(),
                description: t.description.unwrap_or_default().to_string(),
                input_schema: Value::Object((*t.input_schema).clone()),
            })
            .collect())
    }

    async fn call_tool(&self, name: String, arguments: Value) -> Result<ToolResult> {
        let arguments = match arguments {
            Value::Object(map) => map,
            _ => Map::new(),
        };

        let params = CallToolRequestParams::new(name.clone()).with_arguments(arguments);

        let result = self.client.call_tool(params).await?;

        Ok(ToolResult {
            name,
            content: result.structured_content.unwrap_or(Value::Null),
        })
    }
}
