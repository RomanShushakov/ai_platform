use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedChunk {
    pub doc_id: String,
    pub chunk_id: String,
    pub title: String,
    pub content: String,
    pub score: f32,
    pub source: Option<String>,
}
