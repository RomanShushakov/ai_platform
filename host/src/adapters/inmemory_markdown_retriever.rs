use std::{collections::HashSet, fs, path::Path};

use anyhow::{Context, Result};
use async_trait::async_trait;
use shared_types::{RetrievalQuery, RetrievalResult, RetrievedChunk};

use crate::domain::retriever::Retriever;

#[derive(Clone, Default)]
pub struct InMemoryMarkdownRetriever {
    chunks: Vec<RetrievedChunk>,
}

impl InMemoryMarkdownRetriever {
    pub fn load_from_dir(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let mut chunks = Vec::new();

        visit_markdown_files(root, &mut |path| {
            let content = fs::read_to_string(path)
                .with_context(|| format!("failed to read markdown file: {}", path.display()))?;

            let doc_id = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            let title = extract_title(&content).unwrap_or_else(|| {
                path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "untitled".to_string())
            });

            let doc_chunks = chunk_markdown(&doc_id, &title, &content);
            chunks.extend(doc_chunks);

            Ok::<(), anyhow::Error>(())
        })?;

        Ok(Self { chunks })
    }
}

#[async_trait]
impl Retriever for InMemoryMarkdownRetriever {
    async fn retrieve(&self, query: RetrievalQuery) -> Result<RetrievalResult> {
        let q_terms = tokenize(&query.query);

        let mut scored: Vec<(f32, &RetrievedChunk)> = self
            .chunks
            .iter()
            .map(|chunk| {
                let score = score_chunk(&q_terms, chunk);
                (score, chunk)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.0.total_cmp(&a.0));

        let chunks = scored
            .into_iter()
            .take(query.top_k)
            .map(|(score, chunk)| {
                let mut c = chunk.clone();
                c.score = score;
                c
            })
            .collect();

        Ok(RetrievalResult { chunks })
    }
}

fn visit_markdown_files(root: &Path, f: &mut impl FnMut(&Path) -> Result<()>) -> Result<()> {
    for entry in fs::read_dir(root)
        .with_context(|| format!("failed to read directory: {}", root.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            visit_markdown_files(&path, f)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            f(&path)?;
        }
    }

    Ok(())
}

fn extract_title(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("# ")
            .map(|title| title.trim().to_string())
    })
}

fn chunk_markdown(doc_id: &str, title: &str, content: &str) -> Vec<RetrievedChunk> {
    let paragraphs: Vec<&str> = content
        .split("\n\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut chunk_index = 0usize;
    let target_size = 500usize;

    for para in paragraphs {
        if !current.is_empty() && current.len() + para.len() + 2 > target_size {
            chunks.push(RetrievedChunk {
                doc_id: doc_id.to_string(),
                chunk_id: format!("chunk-{}", chunk_index),
                title: title.to_string(),
                content: current.trim().to_string(),
                score: 0.0,
                source: Some(doc_id.to_string()),
            });
            chunk_index += 1;
            current.clear();
        }

        current.push_str(para);
        current.push_str("\n\n");
    }

    if !current.trim().is_empty() {
        chunks.push(RetrievedChunk {
            doc_id: doc_id.to_string(),
            chunk_id: format!("chunk-{}", chunk_index),
            title: title.to_string(),
            content: current.trim().to_string(),
            score: 0.0,
            source: Some(doc_id.to_string()),
        });
    }

    chunks
}

fn tokenize(s: &str) -> HashSet<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .filter(|t| t.len() > 2)
        .map(|t| t.to_string())
        .collect()
}

fn score_chunk(query_terms: &HashSet<String>, chunk: &RetrievedChunk) -> f32 {
    if query_terms.is_empty() {
        return 0.0;
    }

    let text = format!("{} {}", chunk.title, chunk.content).to_lowercase();

    let mut score = 0.0;
    for term in query_terms {
        if text.contains(term) {
            score += 1.0;
        }
    }

    score
}
