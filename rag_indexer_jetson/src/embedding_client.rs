use anyhow::Result;
use reqwest::Client;
use serde_json::{Value, json};

pub struct EmbeddingClient {
    client: Client,
    base_url: String,
    model: String,
}

impl EmbeddingClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            model,
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base_url);

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

        let embedding = resp["embedding"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("invalid embedding response"))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }
}
