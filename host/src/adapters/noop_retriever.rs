use anyhow::Result;
use async_trait::async_trait;

use crate::domain::retriever::Retriever;

use shared_types::{RetrievalQuery, RetrievalResult};

#[derive(Clone, Default)]
pub struct NoopRetriever;

#[async_trait]
impl Retriever for NoopRetriever {
    async fn retrieve(&self, _query: RetrievalQuery) -> Result<RetrievalResult> {
        Ok(RetrievalResult::default())
    }
}
