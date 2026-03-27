use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use uuid::Uuid;

use crate::domain::llm_backend::LlmBackend;
use crate::domain::retriever::Retriever;

use crate::{
    config::Config,
    domain::{llm_loop::run_chat_loop, tool_provider::ToolProvider},
};
use shared_types::{UiChatRequest, UiChatResponse};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub tool_provider: Arc<dyn ToolProvider>,
    pub llm_backend: Arc<dyn LlmBackend>,
    pub retriever: Arc<dyn Retriever>,
}

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok"
    }))
}

pub async fn chat(
    State(state): State<AppState>,
    Json(request): Json<UiChatRequest>,
) -> Result<Json<UiChatResponse>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4();

    match run_chat_loop(
        request_id,
        request.message,
        state.llm_backend.as_ref(),
        state.tool_provider.as_ref(),
        state.retriever.as_ref(),
        state.config.retrieval_top_k,
        state.config.max_llm_steps,
    )
    .await
    {
        Ok(response) => Ok(Json(response)),
        Err(err) => {
            error!(request_id = %request_id, error = ?err, "chat request failed");

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": err.to_string(),
                    "request_id": request_id
                })),
            ))
        }
    }
}
