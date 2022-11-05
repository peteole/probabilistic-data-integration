pub mod datasource;
pub mod datasources;
pub mod numeric;
pub mod search_engine;
pub mod search_result;
pub mod string;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        datasource::MockDataSource,
        datasources::openFoodFacts::OpenFoodFactsDataSource,
        search_engine::{Field, FieldType, FieldValue, SearchEngine},
    };

    #[tokio::test]
    async fn it_works() {
        let engine = SearchEngine {
            data_sources: vec![
                Box::new(MockDataSource::demo1()),
                Box::new(MockDataSource::demo2()),
                Box::new(OpenFoodFactsDataSource::default()),
            ],
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
                    "energy_density".to_string(),
                    Field {
                        description: "Energy of the object".to_string(),
                        field_type: FieldType::Float {
                            unit: "Joule/100g".to_string(),
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
            ]),
        };
        let result = engine.search("apple".to_string()).await;
        println!("{:?}", result);
        let (_desc, value) = result.fields.get("energy_density").unwrap();
        match value {
            FieldValue::Numeric(v) => {
                println!("Mean: {}", v.mean());
                println!("Sigma: {}", v.sigma());
                println!("Sigma: {:?}", v.get_distribution(200));
            }
            _ => panic!("Expected numeric value"),
        }
    }
}
