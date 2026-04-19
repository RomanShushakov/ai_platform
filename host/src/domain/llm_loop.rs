use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use tracing::info;
use uuid::Uuid;

use llm_client::ChatRequest;
use shared_types::{
    HybridLiveStatus, LlmMessage, LlmOutput, QueryRoute, RetrievalQuery, SourceRef, UiChatResponse,
};

use crate::domain::{
    llm_backend::LlmBackend,
    query_router::{analyze_hybrid_query, route_name, route_query},
    retriever::Retriever,
    tool_provider::ToolProvider,
};

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

    let route = route_query(&user_message);
    steps.push(format!("Query route: {}", route_name(&route)));

    let hybrid_analysis = analyze_hybrid_query(&user_message, &tools);

    if hybrid_analysis.has_doc_intent && hybrid_analysis.has_live_intent {
        match hybrid_analysis.live_status {
            HybridLiveStatus::ToolAvailable => {
                steps.push(
                    "Hybrid analysis: doc intent + live intent, matching live tool available"
                        .to_string(),
                );
            }
            HybridLiveStatus::MissingTool => {
                steps.push("Hybrid analysis: doc intent + live intent, but no matching live tool is available".to_string());
            }
            HybridLiveStatus::NotNeeded => {}
        }
    }

    let retrieval = match route {
        QueryRoute::ToolFirst => {
            steps.push("Skipped retrieval for tool-first query".to_string());
            shared_types::RetrievalResult::default()
        }
        QueryRoute::RetrievalFirst | QueryRoute::Hybrid => {
            retriever
                .retrieve(RetrievalQuery {
                    query: user_message.clone(),
                    top_k: retrieval_top_k,
                })
                .await?
        }
    };

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
        let mut context = "Retrieved knowledge base context is provided below. \
            For documentation and policy questions, prefer answering \
            from this retrieved context before calling tools.\n"
            .to_string();

        for chunk in &retrieval.chunks {
            context.push_str(&format!(
                "\n[doc_id: {} | chunk_id: {} | title: {}]\n{}\n",
                chunk.doc_id, chunk.chunk_id, chunk.title, chunk.content
            ));
        }

        messages.push(LlmMessage::System { content: context });
    }

    if hybrid_analysis.has_doc_intent
        && hybrid_analysis.has_live_intent
        && matches!(hybrid_analysis.live_status, HybridLiveStatus::MissingTool)
    {
        messages.push(LlmMessage::System {
        content: "The user asked a mixed question containing documentation/policy content and live operational status. \
            You may answer only the documentation/policy part from retrieved context. \
            Do not invent, assume, or guess any live/personal/operational status. \
            Explicitly state that the live status cannot be checked because no matching tool is configured."
            .to_string(),
    });
    }

    messages.push(LlmMessage::User {
        content: user_message,
    });

    for step_idx in 0..max_steps {
        info!(request_id = %request_id, step = step_idx, "calling llm");

        let llm_tools = if hybrid_analysis.has_doc_intent
            && hybrid_analysis.has_live_intent
            && matches!(hybrid_analysis.live_status, HybridLiveStatus::MissingTool)
        {
            Vec::new()
        } else {
            tools.clone()
        };

        let llm_output = llm_backend
            .chat(ChatRequest {
                messages: messages.clone(),
                tools: llm_tools.clone(),
            })
            .await
            .with_context(|| format!("llm call failed at step {}", step_idx))?;

        match llm_output {
            LlmOutput::FinalText { text } => {
                steps.push("Generated final answer".to_string());

                let mut safety_notes = Vec::new();

                if hybrid_analysis.has_doc_intent
                    && hybrid_analysis.has_live_intent
                    && matches!(hybrid_analysis.live_status, HybridLiveStatus::MissingTool)
                {
                    safety_notes.push(
                        "Live operational status was requested, but no matching tool is configured.".to_string()
                    );
                }

                return Ok(UiChatResponse {
                    answer: text,
                    steps,
                    request_id,
                    sources,
                    retrieval_confidence,
                    safety_notes,
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
