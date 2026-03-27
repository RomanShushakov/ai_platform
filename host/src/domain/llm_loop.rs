use anyhow::{Context, Result, bail};
use tracing::info;
use uuid::Uuid;

use llm_client::ChatRequest;
use shared_types::{LlmMessage, LlmOutput, RetrievalQuery, UiChatResponse};

use crate::domain::{llm_backend::LlmBackend, retriever::Retriever, tool_provider::ToolProvider};

pub async fn run_chat_loop(
    request_id: Uuid,
    user_message: String,
    llm_backend: &dyn LlmBackend,
    tool_provider: &dyn ToolProvider,
    retriever: &dyn Retriever,
    retrieval_top_k: usize,
    max_steps: usize,
) -> Result<UiChatResponse> {
    let mut steps = Vec::new();

    let tools = tool_provider.list_tools().await?;
    steps.push(format!("Loaded {} tool(s)", tools.len()));

    let retrieval = retriever
        .retrieve(RetrievalQuery {
            query: user_message.clone(),
            top_k: retrieval_top_k,
        })
        .await?;

    steps.push(format!("Retrieved {} chunk(s)", retrieval.chunks.len()));

    let mut messages = Vec::new();

    if !retrieval.chunks.is_empty() {
        let mut context = String::from("Retrieved context:\n");

        for chunk in &retrieval.chunks {
            context.push_str(&format!(
                "\n[{} / {}]\n{}\n",
                chunk.doc_id, chunk.chunk_id, chunk.content
            ));
        }

        messages.push(LlmMessage::System {
            content: format!(
                "Use the retrieved context when it is relevant. \
                If the retrieved context is insufficient, \
                you may use tools or answer directly if appropriate.\n\n{}",
                context
            ),
        });
    }

    messages.push(LlmMessage::User {
        content: user_message,
    });

    for step_idx in 0..max_steps {
        info!(request_id = %request_id, step = step_idx, "calling llm");

        let llm_output = llm_backend
            .chat(ChatRequest {
                messages: messages.clone(),
                tools: tools.clone(),
            })
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

                let tool_result = tool_provider
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
