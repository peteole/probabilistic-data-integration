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
    pub fn demo1() -> Self {
        Self {
            data: HashMap::from([(
                "apple".to_string(),
                SearchResult {
                    numeric_fields: HashMap::from([
                        ("weight".into(), NumericFieldValue::Exact(0.5)),
                        (
                            "energy_density".into(),
                            NumericFieldValue::Normal {
                                sigma: 200.0,
                                mean: 300.0,
                            },
                        ),
                    ]),
                    string_fields: HashMap::from([
                        ("color".into(), StringFieldValue::Exact("red".to_string())),
                        (
                            "origin".into(),
                            StringFieldValue::Distribution(HashMap::from([
                                ("de".into(), 0.4),
                                ("fr".into(), 0.6),
                            ])),
                        ),
                    ]),
                },
            )]),
        }
    }

    pub fn demo2() -> Self {
        Self {
            data: HashMap::from([(
                "apple".into(),
                SearchResult {
                    numeric_fields: HashMap::from([
                        ("weight".to_string(), NumericFieldValue::Exact(0.5)),
                        (
                            "energy_density".to_string(),
                            NumericFieldValue::Uniform {
                                min: 200.0,
                                max: 230.0,
                            },
                        ),
                    ]),
                    string_fields: HashMap::from([
                        ("color".into(), StringFieldValue::Exact("red".to_string())),
                        ("origin".into(), StringFieldValue::Exact("de".to_string())),
                    ]),
                },
            )]),
        }
    }
}
