use axum::{
    Router,
    routing::{get, post},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

mod adapters;
mod api;
mod config;
mod domain;

use crate::{
    config::Config,
    domain::{llm_backend::LlmBackend, tool_provider::ToolProvider},
};
use adapters::{
    http_tool_provider::HttpToolProvider, mcp_tool_provider::McpToolProvider,
    ollama_llm_backend::OllamaLlmBackend, tools_client::ToolsClient,
    vllm_llm_backend::VllmLlmBackend,
};
use api::AppState;

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

    let llm_backend: Arc<dyn LlmBackend> = match config.llm_backend.as_str() {
        "ollama" => {
            info!(
                base_url = %config.llm_base_url,
                model = %config.llm_model,
                "using Ollama LLM backend"
            );
            Arc::new(OllamaLlmBackend::new(
                config.llm_base_url.clone(),
                config.llm_model.clone(),
            ))
        }
        "vllm" => {
            info!(
                base_url = %config.llm_base_url,
                model = %config.llm_model,
                "using vLLM LLM backend"
            );
            Arc::new(VllmLlmBackend::new(
                config.llm_base_url.clone(),
                config.llm_model.clone(),
            ))
        }
        other => anyhow::bail!("unsupported LLM_BACKEND value: {}", other),
    };

    let state = AppState {
        config: config.clone(),
        tool_provider,
        llm_backend,
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
