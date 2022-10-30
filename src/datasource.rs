use std::collections::HashMap;

use crate::{numeric::NumericFieldValue, search_result::SearchResult, string::StringFieldValue};

pub trait DataSource {
    fn search(&self, query: String) -> Option<SearchResult>;
}

pub struct MockDataSource {}
impl DataSource for MockDataSource {
    fn search(&self, query: String) -> Option<SearchResult> {
        if query == "apple" {
            return Some(SearchResult {
                numeric_fields: HashMap::from([
                    ("weight".to_string(), NumericFieldValue::Exact(0.5)),
                    (
                        "calories".to_string(),
                        NumericFieldValue::Normal {
                            sigma: 100.0,
                            mean: 300.0,
                        },
                    ),
                ]),
                string_fields: HashMap::from([
                    (
                        "color".to_string(),
                        StringFieldValue::Exact("red".to_string()),
                    ),
                    (
                        "taste".to_string(),
                        StringFieldValue::Exact("sweet".to_string()),
                    ),
                ]),
            });
        } else {
            None
        }
    }
}
