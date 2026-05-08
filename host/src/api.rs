use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use shared_types::{UiChatRequest, UiChatResponse};

use crate::domain::llm_backend::LlmBackend;
use crate::domain::llm_loop::run_direct_chat;
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
    pub llm_lora_backend: Option<Arc<dyn LlmBackend>>,
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

    let selected_llm_backend = match request.llm_profile.as_deref().unwrap_or("base") {
        "base" | "default" => state.llm_backend.clone(),
        "lora" => {
            if let Some(llm_lora_backend) = state.llm_lora_backend.clone() {
                llm_lora_backend
            } else {
                error!(request_id = %request_id, "llm_profile=lora requested, but LLM_LORA_BASE_URL is not configured");

                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "llm_profile=lora requested, but LLM_LORA_BASE_URL is not configured".to_string(),
                        "request_id": request_id
                    })),
                ));
            }
        }
        other => {
            error!(request_id = %request_id, "{}", format!("unsupported llm_profile value: {}", other));

            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("unsupported llm_profile value: {}", other),
                    "request_id": request_id
                })),
            ));
        }
    };

    let response_mode = request.response_mode.as_deref().unwrap_or("agent");

    match response_mode {
        "agent" => {
            match run_chat_loop(
                request_id,
                request.message,
                selected_llm_backend.as_ref(),
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
                    error!(request_id = %request_id, error = ?err, "chat agent request failed");

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
        "direct" => {
            match run_direct_chat(request_id, request.message, selected_llm_backend.as_ref()).await
            {
                Ok(response) => Ok(Json(response)),
                Err(err) => {
                    error!(request_id = %request_id, error = ?err, "chat direct request failed");

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
        other => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("unsupported response_mode value: {}", other),
                "request_id": request_id
            })),
        )),
    }
}
