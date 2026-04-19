use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::{collections::HashMap, fs};

use shared_types::RetrievedChunk;
use shared_types::{RetrievalQuery, RetrievalResult};

use crate::domain::retriever::Retriever;

#[derive(Debug, Deserialize)]
struct ChunkEmbedding {
    doc_id: String,
    chunk_id: String,
    embedding: Vec<f32>,
}

#[derive(Clone)]
pub struct EmbeddingsRetriever {
    chunks: Vec<RetrievedChunk>,
    embeddings: HashMap<String, Vec<f32>>,
    client: reqwest::Client,
    base_url: String,
    model: String,
    min_score: f32,
    relative_ratio: f32,
}

impl EmbeddingsRetriever {
    pub fn load(
        artifacts_path: &str,
        base_url: String,
        model: String,
        min_score: f32,
        relative_ratio: f32,
    ) -> Result<Self> {
        let chunks: Vec<RetrievedChunk> = serde_json::from_str(&fs::read_to_string(format!(
            "{}/chunks.json",
            artifacts_path
        ))?)?;

        let embeddings_vec: Vec<ChunkEmbedding> = serde_json::from_str(&fs::read_to_string(
            format!("{}/embeddings.json", artifacts_path),
        )?)?;

        let embeddings = embeddings_vec
            .into_iter()
            .map(|e| (format!("{}::{}", e.doc_id, e.chunk_id), e.embedding))
            .collect();

        Ok(Self {
            chunks,
            embeddings,
            client: reqwest::Client::new(),
            base_url,
            model,
            min_score,
            relative_ratio,
        })
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base_url);

        let resp = self
            .client
            .post(url)
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": text
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let emb = resp["embedding"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("invalid embedding response"))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(emb)
    }
}

#[async_trait]
impl Retriever for EmbeddingsRetriever {
    async fn retrieve(&self, query: RetrievalQuery) -> Result<RetrievalResult> {
        let query_emb = self.embed_query(&query.query).await?;

        let mut scored = Vec::new();

        for chunk in &self.chunks {
            let key = format!("{}::{}", chunk.doc_id, chunk.chunk_id);

            if let Some(chunk_emb) = self.embeddings.get(&key) {
                let score = cosine_similarity(&query_emb, chunk_emb);

                if score >= self.min_score {
                    let mut c: RetrievedChunk = chunk.clone();
                    c.score = score;
                    scored.push(c);
                }
            }
        }

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        let best_score = scored.first().map(|c| c.score).unwrap_or(0.0);
        let cutoff = best_score * self.relative_ratio;

        let filtered: Vec<_> = scored
            .into_iter()
            .filter(|c| c.score >= self.min_score && c.score >= cutoff)
            .take(query.top_k)
            .collect();

        Ok(RetrievalResult { chunks: filtered })
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a.sqrt() * norm_b.sqrt())
}
