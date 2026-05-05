use anyhow::Result;
use reqwest::Client;
use serde_json::{Value, json};

pub struct EmbeddingClient {
    client: Client,
    backend: String,
    base_url: String,
    model: String,
}

impl EmbeddingClient {
    pub fn new(backend: String, base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            backend,
            base_url,
            model,
        }
    }

    async fn embed_ollama(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base_url.trim_end_matches('/'));

        let resp = self
            .client
            .post(url)
            .json(&json!({
                "model": self.model,
                "prompt": text
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        parse_ollama_embedding(resp)
    }

    async fn embed_openai_compat(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/v1/embeddings", self.base_url.trim_end_matches('/'));

        let resp = self
            .client
            .post(url)
            .json(&json!({
                "model": self.model,
                "input": text
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        parse_openai_compat_embedding(resp)
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match self.backend.as_str() {
            "ollama" => self.embed_ollama(text).await,
            "openai_compat" => self.embed_openai_compat(text).await,
            other => anyhow::bail!("unsupported EMBEDDING_BACKEND value: {}", other),
        }
    }
}

fn parse_ollama_embedding(resp: Value) -> Result<Vec<f32>> {
    let embedding = resp["embedding"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("invalid ollama embedding response"))?
        .iter()
        .map(|v| v.as_f64().unwrap_or(0.0) as f32)
        .collect();

    Ok(embedding)
}

fn parse_openai_compat_embedding(resp: Value) -> Result<Vec<f32>> {
    let embedding = resp["data"][0]["embedding"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("invalid OpenAI-compatible embedding response"))?
        .iter()
        .map(|v| v.as_f64().unwrap_or(0.0) as f32)
        .collect();

    Ok(embedding)
}
