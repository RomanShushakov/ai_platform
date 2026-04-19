use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkEmbedding {
    pub doc_id: String,
    pub chunk_id: String,
    pub embedding: Vec<f32>,
}
