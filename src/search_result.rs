use std::collections::{HashMap, HashSet};

use crate::{numeric::NumericFieldValue, string::StringFieldValue};

#[derive(Debug, Clone)]
pub struct SearchResult{
    /// Map from the field key to its value
    pub numeric_fields: HashMap<String, NumericFieldValue>,
    pub string_fields: HashMap<String, StringFieldValue>,
}
impl Default for SearchResult {
    fn default() -> Self {
        Self {
            numeric_fields: HashMap::new(),
            string_fields: HashMap::new(),
        }
    }
}

impl<'b> SearchResult {
    pub fn merge(results: &'b Vec<Self>) -> Self {
        let mut string_keys = HashSet::new();
        for (key, _) in results
            .into_iter()
            .flat_map(|result| result.string_fields.clone())
        {
            string_keys.insert(key.clone());
        }
        let mut numeric_keys = HashSet::new();
        for (key, _) in results
            .into_iter()
            .flat_map(|result| result.numeric_fields.clone())
        {
            numeric_keys.insert(key.clone());
        }
        let mut string_fields = HashMap::new();
        let mut numeric_fields = HashMap::new();
        for key in numeric_keys {
            let values= results
                .into_iter()
                .filter_map(|result| result.numeric_fields.get(&key))
                .map(|value| value.to_owned())
                .collect();
            numeric_fields.insert(key, NumericFieldValue::merge(values));
        }
        for key in string_keys {
            let values = results
                .into_iter()
                .filter_map(|result| result.string_fields.get(&key))
                .map(|value| value.to_owned())
                .collect();
            string_fields.insert(key, StringFieldValue::merge(values));
        }
        SearchResult {
            string_fields,
            numeric_fields,
        }
    }
}
