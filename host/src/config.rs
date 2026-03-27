#[derive(Debug, Clone)]
pub struct Config {
    pub host_port: u16,
    pub tool_provider: String,
    pub tools_base_url: String,
    pub mcp_tools_binary: String,
    pub llm_backend: String,
    pub llm_base_url: String,
    pub llm_model: String,
    pub max_llm_steps: usize,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host_port: std::env::var("HOST_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),

            tool_provider: std::env::var("TOOL_PROVIDER").unwrap_or_else(|_| "mcp".to_string()),

            tools_base_url: std::env::var("TOOLS_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),

            mcp_tools_binary: std::env::var("MCP_TOOLS_BINARY")
                .unwrap_or_else(|_| "target/debug/tools-server".to_string()),

            llm_backend: std::env::var("LLM_BACKEND").unwrap_or_else(|_| "ollama".to_string()),

            llm_base_url: std::env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),

            llm_model: std::env::var("LLM_MODEL").unwrap_or_else(|_| "llama3".to_string()),

            max_llm_steps: std::env::var("MAX_LLM_STEPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),
        }
    }
}
