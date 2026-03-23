use axum::{Json, http::StatusCode};
use serde_json::json;
use shared_types::{ToolCallRequest, ToolDefinition, ToolResult};

use crate::tools;

pub async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok"
    }))
}

pub async fn list_tools() -> Json<Vec<ToolDefinition>> {
    Json(tools::all_definitions())
}

pub async fn call_tool(
    Json(request): Json<ToolCallRequest>,
) -> Result<Json<ToolResult>, (StatusCode, Json<serde_json::Value>)> {
    match tools::execute(request).await {
        Ok(result) => Ok(Json(result)),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": err.to_string()
            })),
        )),
    }
}
