
use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use serde::Deserialize;

use crate::{search_result::SearchResult, datasource::DataSource};
#[derive(Deserialize)]
pub struct MockDataSource {
    pub data: HashMap<String, SearchResult>,
}
#[async_trait]
impl DataSource for MockDataSource {
    async fn search(&self, query: String) -> Option<SearchResult> {
        if let Some(res) =self.data.get(&query) {
            return Some(res.clone())
        }
        for (key, value) in self.data.iter() {
            if regex::Regex::new(&key).map(|r| r.is_match(&query)).unwrap_or(false){
                return Some(value.clone())
            }
        }
        None
    }
}

impl MockDataSource {
    pub fn load_from_file(data_file: impl Into<PathBuf>) -> Self {
        config::Config::builder()
            .add_source(config::File::from(data_file.into()))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}