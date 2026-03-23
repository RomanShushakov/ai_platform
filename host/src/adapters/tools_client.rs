use std::time::Duration;
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use shared_types::{ToolCallRequest, ToolDefinition, ToolResult};

#[derive(Clone)]
pub struct ToolsClient {
    http: Client,
    base_url: String,
}

impl ToolsClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build reqwest client");

        Self {
            http,
            base_url: base_url.into(),
        }
    }

    pub async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        let url = format!("{}/tools", self.base_url);

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("failed to call tools /tools endpoint")?;

        let status = resp.status();

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("tools list failed: status={}, body={}", status, body);
        }

        let tools = resp
            .json::<Vec<ToolDefinition>>()
            .await
            .context("failed to deserialize tools list")?;

        Ok(tools)
    }

    pub async fn call_tool(&self, name: String, arguments: Value) -> Result<ToolResult> {
        let url = format!("{}/tools/call", self.base_url);

        let request = ToolCallRequest { name, arguments };

        let resp = self
            .http
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("failed to call tool execution endpoint")?;

        let status = resp.status();

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("tool call failed: status={}, body={}", status, body);
        }

        let result = resp
            .json::<ToolResult>()
            .await
            .context("failed to deserialize tool result")?;

        Ok(result)
    }
}
