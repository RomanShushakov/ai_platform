mod adapters;
mod api;
mod config;
mod domain;

use std::{net::SocketAddr, sync::Arc};

use adapters::{
    http_tool_provider::HttpToolProvider, mcp_tool_provider::McpToolProvider,
    tools_client::ToolsClient,
};
use api::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use llm_client::OllamaClient;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::{config::Config, domain::tool_provider::ToolProvider};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "host=debug,llm_client=debug,tower_http=debug".to_string()),
        )
        .init();

    let config = Config::from_env();

    let tool_provider: Arc<dyn ToolProvider> = match config.tool_provider.as_str() {
        "http" => {
            info!(base_url = %config.tools_base_url, "using HTTP tool provider");
            Arc::new(HttpToolProvider::new(ToolsClient::new(
                config.tools_base_url.clone(),
            )))
        }
        "mcp" => {
            info!(binary = %config.mcp_tools_binary, "using MCP tool provider");
            Arc::new(McpToolProvider::new(&config.mcp_tools_binary).await?)
        }
        other => anyhow::bail!("unsupported TOOL_PROVIDER value: {}", other),
    };

    let state = AppState {
        tool_provider,
        llm_client: Arc::new(OllamaClient::new(
            config.ollama_base_url.clone(),
            config.ollama_model.clone(),
        )),
        config: config.clone(),
    };

    let app = Router::new()
        .route("/health", get(api::health))
        .route("/chat", post(api::chat))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.host_port));
    info!("host listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
