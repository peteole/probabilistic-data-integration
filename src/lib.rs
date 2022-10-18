use std::collections::HashMap;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Float,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub key: String,
    pub description: String,
    pub unit: String,
    pub field_type: FieldType,
}
#[derive(Debug, Clone)]
pub enum NumericFieldValue {
    Normal { sigma: f64, mean: f64 },
    Exact(f64),
    Uniform { min: f64, max: f64 },
    Combination(Vec<NumericFieldValue>),
}
#[derive(Debug, Clone)]
pub enum StringFieldValue {
    Exact(String),
    /// Possible values mapped to their probability. If the sum of the probabilities is not 1, the remaining probability is assigned to the "other" value.
    Distribution(HashMap<String, f64>),
}

#[derive(Debug, Clone)]
pub enum FieldValue {
    String(StringFieldValue),
    Numeric(NumericFieldValue),
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Map from the field key to its value
    fields: HashMap<String, FieldValue>,
}
impl Default for SearchResult {
    fn default() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
}

impl SearchResult {
    pub fn join(a: &Self, b: &Self) -> Result<Self, String> {
        let mut fields = HashMap::new();
        for (key, value) in a.fields.iter() {
            fields.insert(key.clone(), value.clone());
        }
        for (key, value) in b.fields.iter() {
            let value_to_insert = if let Some(existing_value) = &fields.get(key) {
                match existing_value {
                    FieldValue::Numeric(n) => {
                        let other_numeric = match value {
                            FieldValue::Numeric(n) => n,
                            _ => {
                                return Err(format!(
                                    "Cannot merge field {} because it is not of same type",
                                    key
                                ))
                            }
                        };
                        FieldValue::Numeric(NumericFieldValue::Combination(vec![
                            n.clone(),
                            other_numeric.clone(),
                        ]))
                    }
                    FieldValue::String(_s) => {
                        return Err(format!("Not implemented"));
                    }
                }
            } else {
                value.clone()
            };
            fields.insert(key.clone(), value_to_insert);
        }
        Ok(SearchResult { fields })
    }
}

pub trait DataSource {
    fn search(&self, query: String) -> Option<SearchResult>;
}

pub struct SearchEngine {
    pub data_sources: Vec<Box<dyn DataSource>>,
}

impl SearchEngine {
    pub fn search(&self, query: String) -> Option<SearchResult> {
        let mut result: Option<SearchResult> = None;
        for data_source in &self.data_sources {
            if let Some(engine_result) = data_source.search(query.clone()) {
                if let Some(existing_result) = result {
                    result = Some(SearchResult::join(&existing_result, &engine_result).unwrap());
                } else {
                    result = Some(engine_result);
                }
            }
        }
        result
    }
}

pub struct MockDataSource {}
impl DataSource for MockDataSource {
    fn search(&self, query: String) -> Option<SearchResult> {
        if query == "apple" {
            return Some(SearchResult {
                fields: HashMap::from([
                    (
                        "weight".to_string(),
                        FieldValue::Numeric(NumericFieldValue::Exact(0.5)),
                    ),
                    (
                        "calories".to_string(),
                        FieldValue::Numeric(NumericFieldValue::Normal {
                            sigma: 100.0,
                            mean: 300.0,
                        }),
                    ),
                ]),
            });
        } else {
            None
        }
    }
}

