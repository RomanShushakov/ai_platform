#[derive(Debug, Clone)]
pub struct Config {
    pub host_port: u16,
    pub tool_provider: String,
    pub tools_base_url: String,
    pub mcp_tools_binary: String,
    pub llm_backend: String,
    pub llm_base_url: String,
    pub llm_chat_path: String,
    pub llm_model: String,
    pub embedding_base_url: String,
    pub retrieval_backend: String,
    pub retrieval_top_k: usize,
    pub knowledge_base_path: String,
    pub max_llm_steps: usize,
    pub rag_artifacts_path: String,
    pub embedding_model: String,
    pub retrieval_min_score: f32,
    pub retrieval_relative_ratio: f32,
    pub retrieval_use_threshold: f32,
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

            llm_chat_path: std::env::var("LLM_CHAT_PATH")
                .unwrap_or_else(|_| "/api/chat".to_string()),

            llm_model: std::env::var("LLM_MODEL").unwrap_or_else(|_| "llama3".to_string()),

            embedding_base_url: std::env::var("EMBEDDING_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),

            retrieval_backend: std::env::var("RETRIEVAL_BACKEND")
                .unwrap_or_else(|_| "noop".to_string()),

            retrieval_top_k: std::env::var("RETRIEVAL_TOP_K")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),

            knowledge_base_path: std::env::var("KNOWLEDGE_BASE_PATH")
                .unwrap_or_else(|_| "knowledge_base".to_string()),

            max_llm_steps: std::env::var("MAX_LLM_STEPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),

            rag_artifacts_path: std::env::var("RAG_ARTIFACTS_PATH")
                .unwrap_or_else(|_| "artifacts/rag".to_string()),

            embedding_model: std::env::var("EMBEDDING_MODEL")
                .unwrap_or_else(|_| "nomic-embed-text".to_string()),

            retrieval_min_score: std::env::var("RETRIEVAL_MIN_SCORE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.45),

            retrieval_relative_ratio: std::env::var("RETRIEVAL_RELATIVE_RATIO")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.95),

            retrieval_use_threshold: std::env::var("RETRIEVAL_USE_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.12),
        }
    }
}
