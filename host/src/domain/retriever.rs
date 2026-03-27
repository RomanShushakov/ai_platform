use anyhow::Result;
use async_trait::async_trait;

use shared_types::{RetrievalQuery, RetrievalResult};

#[async_trait]
pub trait Retriever: Send + Sync {
    async fn retrieve(&self, query: RetrievalQuery) -> Result<RetrievalResult>;
}
