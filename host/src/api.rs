use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use llm_client::OllamaClient;
use shared_types::{UiChatRequest, UiChatResponse};
use tracing::error;
use uuid::Uuid;

use crate::{adapters::tools_client::ToolsClient, config::Config, domain::llm_loop::run_chat_loop};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub tools_client: ToolsClient,
    pub llm_client: Arc<OllamaClient>,
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
        state.llm_client.as_ref(),
        &state.tools_client,
        state.config.max_llm_steps,
    )
    .await
    {
        Ok(response) => Ok(Json(response)),
        Err(err) => {
            error!(request_id = %request_id, error = %err, "chat request failed");

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
