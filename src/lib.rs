use std::collections::HashMap;
pub mod datasource;
pub mod numeric;
pub mod search_engine;
pub mod search_result;
pub mod string;
use numeric::*;
#[cfg(test)]
mod tests {
    use crate::{
        datasource::MockDataSource,
        search_engine::{Field, FieldType, SearchEngine},
        *,
    };

    #[test]
    fn it_works() {
        let engine = SearchEngine {
            data_sources: vec![Box::new(MockDataSource {}), Box::new(MockDataSource {})],
            search_fields: HashMap::from([
                (
                    "weight".to_string(),
                    Field {
                        description: "Weight of the object".to_string(),
                        field_type: FieldType::Float {
                            unit: "kg".to_string(),
                        },
                    },
                ),
                (
                    "calories".to_string(),
                    Field {
                        description: "Calories of the object".to_string(),
                        field_type: FieldType::Float {
                            unit: "kcal".to_string(),
                        },
                    },
                ),
                (
                    "color".to_string(),
                    Field {
                        description: "Color of the object".to_string(),
                        field_type: FieldType::String,
                    },
                ),
                (
                    "taste".to_string(),
                    Field {
                        description: "Taste of the object".to_string(),
                        field_type: FieldType::String,
                    },
                ),
            ]),
        };
        let result = engine.search("apple".to_string());
        println!("{:?}", result);
    }
}
