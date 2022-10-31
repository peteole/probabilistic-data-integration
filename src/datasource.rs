use std::collections::HashMap;

use async_trait::async_trait;

use crate::{numeric::NumericFieldValue, search_result::SearchResult, string::StringFieldValue};

#[async_trait]
pub trait DataSource {
    async fn search(&self, query: String) -> Option<SearchResult>;
}

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
    pub fn demo1() -> Self {
        Self {
            data: HashMap::from([(
                "apple".to_string(),
                SearchResult {
                    numeric_fields: HashMap::from([
                        ("weight".into(), NumericFieldValue::Exact(0.5)),
                        (
                            "calories".into(),
                            NumericFieldValue::Normal {
                                sigma: 100.0,
                                mean: 300.0,
                            },
                        ),
                    ]),
                    string_fields: HashMap::from([
                        ("color".into(), StringFieldValue::Exact("red".to_string())),
                        ("taste".into(), StringFieldValue::Exact("sweet".to_string())),
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
                            "calories".to_string(),
                            NumericFieldValue::Uniform {
                                min: 280.0,
                                max: 290.0,
                            },
                        ),
                    ]),
                    string_fields: HashMap::from([
                        ("color".into(), StringFieldValue::Exact("red".to_string())),
                        ("taste".into(), StringFieldValue::Exact("sweet".to_string())),
                    ]),
                },
            )]),
        }
    }
}
