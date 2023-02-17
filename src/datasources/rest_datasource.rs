use async_trait::async_trait;

use crate::datasource::DataSource;
use crate::search_result::SearchResult;

pub struct RestDatasource {
    base_url: String,
}
impl RestDatasource {
    pub fn new(base_url: String) -> Self {
        RestDatasource { base_url }
    }
}

#[async_trait]
impl DataSource for RestDatasource {
    async fn search(&self, query: String) -> Option<SearchResult> {
        let response = reqwest::get(format!("{}/search?query={}", self.base_url, query))
            .await
            .ok()?;
        response.json().await.ok()
    }
}
