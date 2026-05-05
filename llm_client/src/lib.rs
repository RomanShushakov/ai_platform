pub mod client;
pub mod models;
pub mod ollama;
pub mod openai_compat;

pub use client::LlmClient;
pub use models::ChatRequest;
pub use ollama::OllamaClient;
pub use openai_compat::OpenAiCompatClient;
