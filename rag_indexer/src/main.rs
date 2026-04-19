use anyhow::Result;
use serde_json::json;
use std::{fs, path::Path};

use shared_types::RetrievedChunk;

mod chunking;
mod embedding_client;
mod types;

use chunking::{chunk_markdown, extract_title, visit_markdown_files};
use embedding_client::EmbeddingClient;
use types::ChunkEmbedding;

#[tokio::main]
async fn main() -> Result<()> {
    let kb_path =
        std::env::var("KNOWLEDGE_BASE_PATH").unwrap_or_else(|_| "knowledge_base".to_string());

    let output_dir =
        std::env::var("RAG_OUTPUT_DIR").unwrap_or_else(|_| "artifacts/rag".to_string());

    let ollama_url =
        std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());

    let embedding_model =
        std::env::var("EMBEDDING_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());

    println!("Loading knowledge base from {}", kb_path);

    let mut chunks = Vec::<RetrievedChunk>::new();

    visit_markdown_files(Path::new(&kb_path), &mut |path| {
        let content = fs::read_to_string(path)?;

        let doc_id = path.to_string_lossy().to_string();
        let title = extract_title(&content).unwrap_or_else(|| "untitled".to_string());

        let doc_chunks = chunk_markdown(&doc_id, &title, &content);
        chunks.extend(doc_chunks);

        Ok(())
    })?;

    println!("Loaded {} chunks", chunks.len());

    let client = EmbeddingClient::new(ollama_url, embedding_model);

    let mut embeddings: Vec<ChunkEmbedding> = Vec::<ChunkEmbedding>::new();

    for chunk in &chunks {
        println!("Embedding {}", chunk.chunk_id);

        let emb = client.embed(&chunk.content).await?;

        embeddings.push(ChunkEmbedding {
            doc_id: chunk.doc_id.clone(),
            chunk_id: chunk.chunk_id.clone(),
            embedding: emb,
        });
    }

    fs::create_dir_all(&output_dir)?;

    fs::write(
        format!("{}/chunks.json", output_dir),
        serde_json::to_string_pretty(&chunks)?,
    )?;

    fs::write(
        format!("{}/embeddings.json", output_dir),
        serde_json::to_string_pretty(&embeddings)?,
    )?;

    fs::write(
        format!("{}/manifest.json", output_dir),
        serde_json::to_string_pretty(&json!({
            "embedding_model": "nomic-embed-text",
            "chunks": chunks.len()
        }))?,
    )?;

    println!("Indexing complete → {}", output_dir);

    Ok(())
}
