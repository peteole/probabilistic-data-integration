use std::collections::HashMap;

use async_trait::async_trait;
use openapi::apis::configuration::Configuration;

use crate::{datasource::DataSource, numeric::NumericFieldValue, search_result::SearchResult};

pub struct OpenFoodFactsDataSource {
    config: Configuration,
}
#[async_trait]
impl DataSource for OpenFoodFactsDataSource {
    async fn search(&self, query: String) -> Option<SearchResult> {
        let raw_res = openapi::apis::read_requests_api::get_search(
            &self.config,
            openapi::apis::read_requests_api::GetSearchParams {
                categories_tags_en: Some(query),
                labels_tags_en: None,
                fields: Some("code,product_name,nutriscore_data,nutriments".into()),
            },
        )
        .await;
        //print!("{:?}", raw_res);
        match raw_res {
            Ok(res) => {
                //println!("{:?}", res);
                match res.products {
                    Some(products) => {
                        let product = match products.first() {
                            Some(p) => p.clone(),
                            None => return None,
                        };
                        if let Some(energy) = product.nutriments?.energy {
                            return Some(SearchResult {
                                numeric_fields: HashMap::from([(
                                    "energy_density".to_string(),
                                    NumericFieldValue::Normal {
                                        sigma: energy.into(),
                                        mean: (energy / 10.0).into(),
                                    },
                                )]),
                                string_fields: HashMap::new(),
                            });
                        }
                        //print!("Products: {:?}", products[0].nutriments);
                        //println!("{:?}", products[0].code);
                    }
                    None => {}
                }
                Some(SearchResult::default())
            }
            Err(e) => {
                println!("{:?}", e);
                None
            }
        }
    }
}
impl Default for OpenFoodFactsDataSource {
    fn default() -> Self {
        Self {
            config: Configuration {
                base_path: "https://world.openfoodfacts.org".to_owned(),
                ..Configuration::default()
            },
        }
    }
}
