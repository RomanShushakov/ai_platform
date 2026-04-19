use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{Value, json};
use std::time::Duration;
use tracing::info;

use shared_types::{LlmMessage, LlmOutput, ToolDefinition};

use crate::{client::LlmClient, models::ChatRequest};

fn build_system_prompt(tools: &[ToolDefinition]) -> String {
    let tools_json = serde_json::to_string_pretty(tools).unwrap();

    format!(
        r#"
            You are an AI assistant with access to tools.

            TOOLS:
            {}

            RULES:
            - You MUST respond in valid JSON only
            - No extra text
            - Two possible formats:

            1) Final answer:
            {{ "type": "final", "text": "your answer" }}

            2) Tool call:
            {{ "type": "tool_call", "name": "tool_name", "arguments": {{ ... }} }}

            - Use tools when needed
            - Use exact tool names
            "#,
        tools_json
    )
}

fn convert_messages(messages: Vec<LlmMessage>) -> Vec<Value> {
    messages
        .into_iter()
        .map(|msg| match msg {
            LlmMessage::User { content } => json!({
                "role": "user",
                "content": content
            }),
            LlmMessage::Assistant { content } => json!({
                "role": "assistant",
                "content": content
            }),
            LlmMessage::System { content } => json!({
                "role": "system",
                "content": content
            }),
            LlmMessage::ToolResult { tool_name, content } => json!({
                "role": "user",
                "content": format!(
                    "Tool '{}' returned:\n{}",
                    tool_name,
                    content
                )
            }),
        })
        .collect()
}

fn parse_llm_output(content: &str) -> Result<LlmOutput> {
    let value: Value =
        serde_json::from_str(content).context("failed to parse LLM JSON response")?;

    let t = value["type"].as_str().context("missing type field")?;

    match t {
        "final" => {
            let text = value["text"].as_str().context("missing text field")?;

            Ok(LlmOutput::FinalText {
                text: text.to_string(),
            })
        }
        "tool_call" => {
            let name = value["name"].as_str().context("missing tool name")?;

            let arguments = value["arguments"].clone();

            Ok(LlmOutput::ToolCall {
                name: name.to_string(),
                arguments,
            })
        }
        other => anyhow::bail!("unknown response type: {}", other),
    }
}

#[derive(Debug, Clone)]
pub struct OllamaClient {
    http_client: Client,
    base_url: String,
    chat_path: String,
    model: String,
}

impl OllamaClient {
    pub fn new(
        base_url: impl Into<String>,
        chat_path: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("failed to build ollama reqwest client");

        Self {
            http_client,
            base_url: base_url.into(),
            chat_path: chat_path.into(),
            model: model.into(),
        }
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn chat(&self, request: ChatRequest) -> Result<LlmOutput> {
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), self.chat_path);

        let system_prompt = build_system_prompt(&request.tools);

        let messages = convert_messages(request.messages);

        let mut all_messages = vec![json!({
            "role": "system",
            "content": system_prompt
        })];

        all_messages.extend(messages);

        let body = json!({
            "model": self.model,
            "messages": all_messages,
            "stream": false
        });

        let resp = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("failed to call ollama")?;

        let status = resp.status();

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("ollama error: {} {}", status, text);
        }

        let value: Value = resp.json().await?;

        let content = value["message"]["content"]
            .as_str()
            .context("missing message content")?;

        info!("LLM raw response: {}", content);

        parse_llm_output(content)
    }
}
