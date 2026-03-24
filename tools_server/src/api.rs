use std::net::SocketAddr;

use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use serde_json::json;
use shared_types::{ToolCallRequest, ToolDefinition, ToolResult};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::tools;

pub async fn run_http_server() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/health", get(health))
        .route("/tools", get(list_tools))
        .route("/tools/call", post(call_tool))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!("tools_server listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

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
