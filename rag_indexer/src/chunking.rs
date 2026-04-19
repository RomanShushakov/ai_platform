use std::{fs, path::Path};

use anyhow::{Context, Result};

use shared_types::RetrievedChunk;

pub fn visit_markdown_files(root: &Path, f: &mut impl FnMut(&Path) -> Result<()>) -> Result<()> {
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

pub fn extract_title(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("# ")
            .map(|title| title.trim().to_string())
    })
}

pub fn chunk_markdown(doc_id: &str, title: &str, content: &str) -> Vec<RetrievedChunk> {
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
