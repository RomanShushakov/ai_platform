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

use adapters::embeddings_retriever::EmbeddingsRetriever;
use adapters::inmemory_markdown_retriever::InMemoryMarkdownRetriever;
use adapters::noop_retriever::NoopRetriever;
use adapters::{
    http_tool_provider::HttpToolProvider, mcp_tool_provider::McpToolProvider,
    ollama_llm_backend::OllamaLlmBackend, openai_compat_llm_backend::OpenAiCompatLlmBackend,
    tools_client::ToolsClient,
};
use api::AppState;
use config::Config;
use domain::{llm_backend::LlmBackend, retriever::Retriever, tool_provider::ToolProvider};

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
                chat_path = %config.llm_chat_path,
                model = %config.llm_model,
                "using Ollama LLM backend"
            );
            Arc::new(OllamaLlmBackend::new(
                config.llm_base_url.clone(),
                config.llm_chat_path.clone(),
                config.llm_model.clone(),
            ))
        }
        "openai_compat" => {
            info!(
                base_url = %config.llm_base_url,
                model = %config.llm_model,
                "using OpenAI-compatible LLM backend"
            );
            Arc::new(OpenAiCompatLlmBackend::new(
                config.llm_base_url.clone(),
                config.llm_model.clone(),
            ))
        }
        other => anyhow::bail!("unsupported LLM_BACKEND value: {}", other),
    };

    let retriever: Arc<dyn Retriever> = match config.retrieval_backend.as_str() {
        "noop" => {
            info!("using noop retriever");
            Arc::new(NoopRetriever)
        }
        "inmemory_markdown" => {
            info!(
                knowledge_base_path = %config.knowledge_base_path,
                "using in-memory markdown retriever"
            );
            Arc::new(InMemoryMarkdownRetriever::load_from_dir(
                &config.knowledge_base_path,
            )?)
        }
        "embeddings_local" => {
            info!("using embeddings retriever");

            Arc::new(EmbeddingsRetriever::load(
                &config.rag_artifacts_path,
                config.embedding_base_url.clone(),
                config.embedding_model.clone(),
                config.retrieval_min_score,
                config.retrieval_relative_ratio,
            )?)
        }
        other => anyhow::bail!("unsupported RETRIEVAL_BACKEND value: {}", other),
    };

    let state = AppState {
        config: config.clone(),
        tool_provider,
        llm_backend,
        retriever,
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
