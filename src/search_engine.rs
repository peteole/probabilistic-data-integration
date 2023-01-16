use futures::future::join_all;
use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    datasource::DataSource, numeric::NumericFieldValue, search_result::SearchResult,
    string::StringFieldValue,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum FieldType {
    String,
    Float { unit: String },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Field {
    pub description: String,
    pub field_type: FieldType,
}

#[derive(Debug, Clone)]
pub enum FieldValue {
    String(StringFieldValue),
    Numeric(NumericFieldValue),
}

pub struct SearchEngine {
    pub search_fields: HashMap<String, Field>,
    pub data_sources: Vec<Box<dyn DataSource+Sync+Send>>,
}

#[derive(Debug, Clone)]
pub struct SearchResponse {
    /// Map from the field key to its value
    pub fields: HashMap<String, (Field, FieldValue)>,
}

impl SearchEngine {
    pub fn verify(&self, result: &SearchResult) -> SearchResult {
        // Check that all fields in result are known and of correct type
        let mut cloned_result = result.clone();
        for (key, _) in &result.numeric_fields {
            if let Some(field) = self.search_fields.get(key) {
                match field.field_type {
                    FieldType::Float { .. } => {}
                    _ => {
                        cloned_result.numeric_fields.remove(key);
                    }
                }
            } else {
                cloned_result.numeric_fields.remove(key);
            }
        }
        for (key, _) in &result.string_fields {
            if let Some(field) = self.search_fields.get(key) {
                if field.field_type != FieldType::String {
                    cloned_result.string_fields.remove(key);
                }
            } else {
                cloned_result.string_fields.remove(key);
            }
        }
        cloned_result
    }
    pub async fn search(&self, query: String) -> SearchResponse {
        let futures = self
            .data_sources
            .iter()
            .map(|source| source.search(query.clone()));
        let results = join_all(futures).await;
        let filtered_results: Vec<SearchResult> = results
            .into_iter()
            .filter_map(|r| r)
            .map(|r| self.verify(&r))
            .collect();
        let merged = SearchResult::merge(&filtered_results);

        SearchResponse {
            fields: merged
                .numeric_fields
                .into_iter()
                .filter_map(|(k, v)| {
                    self.search_fields
                        .get(&k)
                        .map(|field| (k.clone(), (field.clone(), FieldValue::Numeric(v))))
                })
                .chain(merged.string_fields.into_iter().filter_map(|(k, v)| {
                    self.search_fields
                        .get(&k)
                        .map(|field| (k.clone(), (field.clone(), FieldValue::String(v))))
                }))
                .collect(),
        }
    }
}
