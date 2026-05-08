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
            LlmMessage::User { content } => json!({ "role": "user", "content": content }),
            LlmMessage::Assistant { content } => json!({ "role": "assistant", "content": content }),
            LlmMessage::System { content } => json!({ "role": "system", "content": content }),
            LlmMessage::ToolResult { tool_name, content } => json!({
                "role": "user",
                "content": format!("Tool '{}' returned:\n{}", tool_name, content)
            }),
        })
        .collect()
}

fn parse_llm_output(content: &str) -> Result<LlmOutput> {
    let value: Value = match serde_json::from_str(content) {
        Ok(value) => value,
        Err(_) => {
            return Ok(LlmOutput::FinalText {
                text: content.trim().to_string(),
            });
        }
    };

    let t = value["type"].as_str().context("missing type field")?;

    match t {
        "final" => Ok(LlmOutput::FinalText {
            text: value["text"]
                .as_str()
                .context("missing text field")?
                .to_string(),
        }),
        "tool_call" => Ok(LlmOutput::ToolCall {
            name: value["name"]
                .as_str()
                .context("missing tool name")?
                .to_string(),
            arguments: value["arguments"].clone(),
        }),
        other => anyhow::bail!("unknown response type: {}", other),
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiCompatClient {
    http_client: Client,
    base_url: String,
    model: String,
}

impl OpenAiCompatClient {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("failed to build OpenAI-compatible reqwest client");

        Self {
            http_client,
            base_url: base_url.into(),
            model: model.into(),
        }
    }

    pub async fn chat_direct(&self, message: String) -> Result<String> {
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": message
                }
            ],
            "temperature": 0.2,
            "max_tokens": 128
        });

        let resp = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("failed to call OpenAI-compatible direct chat")?;

        let status = resp.status();

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI-compatible direct chat error: {} {}", status, text);
        }

        let value: serde_json::Value = resp.json().await?;

        let content = value["choices"][0]["message"]["content"]
            .as_str()
            .context("missing direct chat message content")?;

        Ok(content.to_string())
    }
}

#[async_trait]
impl LlmClient for OpenAiCompatClient {
    async fn chat(&self, request: ChatRequest) -> Result<LlmOutput> {
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        let system_prompt = build_system_prompt(&request.tools);
        let mut all_messages = vec![json!({
            "role": "system",
            "content": system_prompt
        })];

        all_messages.extend(convert_messages(request.messages));

        let body = json!({
            "model": self.model,
            "messages": all_messages,
            "temperature": 0,
            "max_tokens": 512,
            "stream": false
        });

        let resp = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("failed to call OpenAI-compatible LLM")?;

        let status = resp.status();

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI-compatible LLM error: {} {}", status, text);
        }

        let value: Value = resp.json().await?;

        let content = value["choices"][0]["message"]["content"]
            .as_str()
            .context("missing choices[0].message.content")?;

        info!("LLM raw response: {}", content);

        parse_llm_output(content)
    }
}
