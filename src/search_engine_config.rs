use serde::Deserialize;
use std::collections::HashMap;
use tag_search::{
    datasource::{DataSource, MockDataSource},
    datasources::{grpc_datasource::GrpcDataSource, open_food_facts::OpenFoodFactsDataSource},
    search_engine::{Field, SearchEngine},
};
type FieldsConfig = HashMap<String, Field>;

#[derive(Debug, Clone, Deserialize)]
pub enum DataSourceConfig {
    OpenFoodFacts,
    Mock1,
    Mock2,
    Grpc { address: String },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub fields: FieldsConfig,
    pub data_sources: Vec<DataSourceConfig>,
}

impl Config {
    pub async fn to_search_engine(self) -> SearchEngine {
        let mut new_ds = Vec::with_capacity(self.data_sources.len());
        for data_source in self.data_sources {
            let m: Box<dyn DataSource+Sync+Send> = match data_source {
                DataSourceConfig::OpenFoodFacts => Box::new(OpenFoodFactsDataSource::default()),
                DataSourceConfig::Mock1 => Box::new(MockDataSource::demo1()),
                DataSourceConfig::Mock2 => Box::new(MockDataSource::demo2()),
                DataSourceConfig::Grpc { address } => {
                    let ds = GrpcDataSource::new(address).await;
                    match ds {
                        Ok(ds) => Box::new(ds),
                        Err(e) => {
                            println!("Failed to connect to gRPC server: {}", e);
                            continue;
                        }
                    }
                }
            };
            new_ds.push(m);
        }
        SearchEngine {
            search_fields: self.fields,
            data_sources: new_ds,
        }
    }
}
