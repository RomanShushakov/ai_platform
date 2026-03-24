mod api;
mod mcp_server;
mod tools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = std::env::var("TOOLS_TRANSPORT").unwrap_or_else(|_| "http".to_string());

    eprintln!("TOOLS_SERVER MODE = {}", transport);

    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "tools_server=debug,tower_http=debug".to_string()),
        )
        .with_writer(std::io::stderr)
        .init();

    match transport.as_str() {
        "mcp-stdio" => mcp_server::run_mcp_stdio().await,
        _ => api::run_http_server().await,
    }
}
