mod api;
mod tools;

use std::net::SocketAddr;

use axum::{
    Router,
    routing::{get, post},
};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "tools_server=debug,tower_http=debug".to_string()),
        )
        .init();

    let app = Router::new()
        .route("/health", get(api::health))
        .route("/tools", get(api::list_tools))
        .route("/tools/call", post(api::call_tool))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!("tools_server listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
