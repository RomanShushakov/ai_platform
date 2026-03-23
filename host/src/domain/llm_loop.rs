use anyhow::{Context, Result, bail};
use llm_client::LlmClient;
use shared_types::{LlmMessage, LlmOutput, UiChatResponse};
use tracing::info;
use uuid::Uuid;

use crate::adapters::tools_client::ToolsClient;

pub async fn run_chat_loop(
    request_id: Uuid,
    user_message: String,
    llm_client: &dyn LlmClient,
    tools_client: &ToolsClient,
    max_steps: usize,
) -> Result<UiChatResponse> {
    let mut steps = Vec::new();

    let tools = tools_client.list_tools().await?;
    steps.push(format!("Loaded {} tool(s)", tools.len()));

    let mut messages = vec![LlmMessage::User {
        content: user_message,
    }];

    for step_idx in 0..max_steps {
        info!(request_id = %request_id, step = step_idx, "calling llm");

        let llm_output = llm_client
            .chat_with_messages(messages.clone(), tools.clone())
            .await
            .with_context(|| format!("llm call failed at step {}", step_idx))?;

        match llm_output {
            LlmOutput::FinalText { text } => {
                steps.push("Generated final answer".to_string());

                return Ok(UiChatResponse {
                    answer: text,
                    steps,
                    request_id,
                });
            }

            LlmOutput::ToolCall { name, arguments } => {
                steps.push(format!("LLM requested tool '{}'", name));

                let tool_result = tools_client
                    .call_tool(name.clone(), arguments)
                    .await
                    .with_context(|| format!("tool execution failed for '{}'", name))?;

                steps.push(format!("Executed tool '{}'", tool_result.name));

                info!(
                    request_id = %request_id,
                    tool_name = %tool_result.name,
                    tool_result = %tool_result.content,
                    "tool executed successfully",
                );

                messages.push(LlmMessage::ToolResult {
                    tool_name: tool_result.name,
                    content: tool_result.content,
                });
            }
        }
    }

    tracing::info!(
        request_id = %request_id,
        "final answer generated"
    );

    bail!("max llm steps ({}) reached without final answer", max_steps)
}
