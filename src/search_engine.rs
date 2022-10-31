use futures::future::join_all;
use std::collections::HashMap;

use crate::{
    datasource::DataSource, numeric::NumericFieldValue, search_result::SearchResult,
    string::StringFieldValue,
};

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Float { unit: String },
}

#[derive(Debug, Clone)]
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
    pub data_sources: Vec<Box<dyn DataSource>>,
}

#[derive(Debug, Clone)]
pub struct SearchResponse {
    /// Map from the field key to its value
    pub fields: HashMap<String, (Field, FieldValue)>,
}

impl SearchEngine {
    pub fn verify(&self, result: &SearchResult) -> Result<(), String> {
        // Check that all fields in result are known and of correct type
        for (key, _) in &result.numeric_fields {
            if let Some(field) = self.search_fields.get(key) {
                match field.field_type {
                    FieldType::Float { .. } => {}
                    _ => {
                        return Err(format!(
                            "Field {} is of type {:?}, but got numeric value",
                            key, field.field_type
                        ))
                    }
                }
            } else {
                return Err(format!("Field {} is not known", key));
            }
        }
        for (key, _) in &result.string_fields {
            if let Some(field) = self.search_fields.get(key) {
                if field.field_type != FieldType::String {
                    return Err(format!("Field {} is not of type String", key));
                }
            } else {
                return Err(format!("Field {} is not known", key));
            }
        }
        Ok(())
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
            .filter(|r| match self.verify(r) {
                Ok(_) => true,
                Err(_) => false,
            })
            .collect();
        let merged = SearchResult::merge(&filtered_results);

        SearchResponse {
            fields: merged
                .numeric_fields
                .into_iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        (
                            self.search_fields.get(&k).unwrap().clone(),
                            FieldValue::Numeric(v),
                        ),
                    )
                })
                .chain(merged.string_fields.into_iter().map(|(k, v)| {
                    (
                        k.clone(),
                        (
                            self.search_fields.get(&k).unwrap().clone(),
                            FieldValue::String(v),
                        ),
                    )
                }))
                .collect(),
        }
    }
}
