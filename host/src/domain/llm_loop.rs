use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use tracing::info;
use uuid::Uuid;

use llm_client::ChatRequest;
use shared_types::{LlmMessage, LlmOutput, RetrievalQuery, SourceRef, UiChatResponse};

use crate::domain::{llm_backend::LlmBackend, retriever::Retriever, tool_provider::ToolProvider};

pub async fn run_chat_loop(
    request_id: Uuid,
    user_message: String,
    llm_backend: &dyn LlmBackend,
    tool_provider: &dyn ToolProvider,
    retriever: &dyn Retriever,
    retrieval_top_k: usize,
    retrieval_use_threshold: f32,
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

    let raw_retrieval_confidence = retrieval
        .chunks
        .iter()
        .map(|c| c.score)
        .fold(0.0_f32, f32::max);

    let use_retrieval =
        !retrieval.chunks.is_empty() && raw_retrieval_confidence >= retrieval_use_threshold;

    let retrieval_confidence = if use_retrieval {
        Some(raw_retrieval_confidence)
    } else {
        None
    };

    let mut seen_sources = HashSet::new();
    let mut sources = Vec::new();

    if use_retrieval {
        for chunk in &retrieval.chunks {
            let key = format!("{}::{}", chunk.doc_id, chunk.title);

            if seen_sources.insert(key) {
                sources.push(SourceRef {
                    doc_id: chunk.doc_id.clone(),
                    title: chunk.title.clone(),
                });
            }
        }
    }

    steps.push(format!("Retrieved {} chunk(s)", retrieval.chunks.len()));

    steps.push(format!(
        "Retrieval confidence {:.3} ({})",
        raw_retrieval_confidence,
        if use_retrieval { "used" } else { "ignored" }
    ));

    let mut messages = Vec::new();

    if use_retrieval {
        let mut context = String::from(
            "Retrieved knowledge base context is provided below. \
             For documentation and policy questions, prefer answering \
             from this retrieved context before calling tools.\n",
        );

        for chunk in &retrieval.chunks {
            context.push_str(&format!(
                "\n[doc_id: {} | chunk_id: {} | title: {}]\n{}\n",
                chunk.doc_id, chunk.chunk_id, chunk.title, chunk.content
            ));
        }

        messages.push(LlmMessage::System { content: context });
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
                    sources,
                    retrieval_confidence,
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

    bail!("max llm steps ({}) reached without final answer", max_steps)
}
