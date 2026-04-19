use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use shared_types::{UiChatRequest, UiChatResponse};

use crate::domain::llm_backend::LlmBackend;
use crate::domain::retriever::Retriever;
use crate::{
    config::Config,
    domain::{llm_loop::run_chat_loop, tool_provider::ToolProvider},
};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub tool_provider: Arc<dyn ToolProvider>,
    pub llm_backend: Arc<dyn LlmBackend>,
    pub retriever: Arc<dyn Retriever>,
}

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok"
    }))
}

pub async fn chat(
    State(state): State<AppState>,
    Json(request): Json<UiChatRequest>,
) -> Result<Json<UiChatResponse>, (StatusCode, Json<Value>)> {
    let request_id = Uuid::new_v4();

    match run_chat_loop(
        request_id,
        request.message,
        state.llm_backend.as_ref(),
        state.tool_provider.as_ref(),
        state.retriever.as_ref(),
        state.config.retrieval_top_k,
        state.config.retrieval_use_threshold,
        state.config.max_llm_steps,
    )
    .await
    {
        Ok(response) => Ok(Json(response)),
        Err(err) => {
            error!(request_id = %request_id, error = ?err, "chat request failed");

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": err.to_string(),
                    "request_id": request_id
                })),
            ))
        }
    }
}
