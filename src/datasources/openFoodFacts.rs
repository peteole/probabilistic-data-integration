use async_trait::async_trait;
use openapi::apis::configuration::Configuration;

use crate::{datasource::DataSource, search_result::SearchResult};

pub struct OpenFoodFactsDataSource {
    config: Configuration,
}
#[async_trait]
impl DataSource for OpenFoodFactsDataSource {
    async fn search(&self, query: String) -> Option<SearchResult> {
        let raw_res =
            openapi::apis::read_requests_api::get_search(&self.config,openapi::apis::read_requests_api::GetSearchParams { categories_tags_en: None, labels_tags_en: None, fields: Some("code,product_name".into()) }).await;
        print!("{:?}", raw_res);
        match raw_res {
            Ok(res) => {
                //println!("{:?}", res);
                match res.products{
                    Some(products) => {
                        print!("Products: {:?}", products);
                    },
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
            config: Configuration::default(),
        }
    }
}