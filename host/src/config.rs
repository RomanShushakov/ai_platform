#[derive(Debug, Clone)]
pub struct Config {
    pub host_port: u16,
    pub tools_base_url: String,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub max_llm_steps: usize,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host_port: std::env::var("HOST_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            tools_base_url: std::env::var("TOOLS_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            ollama_base_url: std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            ollama_model: std::env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama3".to_string()),
            max_llm_steps: std::env::var("MAX_LLM_STEPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),
        }
    }
}
