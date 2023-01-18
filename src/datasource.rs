use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use serde::Deserialize;

use crate::{numeric::NumericFieldValue, search_result::SearchResult, string::StringFieldValue};

#[async_trait]
pub trait DataSource {
    async fn search(&self, query: String) -> Option<SearchResult>;
}

#[derive(Deserialize)]
pub struct MockDataSource {
    pub data: HashMap<String, SearchResult>,
}
#[async_trait]
impl DataSource for MockDataSource {
    async fn search(&self, query: String) -> Option<SearchResult> {
        return self.data.get(&query).cloned();
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
