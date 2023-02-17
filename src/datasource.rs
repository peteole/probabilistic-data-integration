

use async_trait::async_trait;

use crate::search_result::SearchResult;
#[async_trait]
pub trait DataSource {
    async fn search(&self, query: String) -> Option<SearchResult>;
}
