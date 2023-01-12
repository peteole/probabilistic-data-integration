use std::collections::HashMap;

use async_trait::async_trait;

use crate::{datasource::DataSource, numeric::NumericFieldValue, search_result::SearchResult};

pub mod grpc_ds {
    tonic::include_proto!("datasource");
}

pub fn convert_string_field_value(
    value: grpc_ds::string_field_value::Value,
) -> crate::string::StringFieldValue {
    match value {
        grpc_ds::string_field_value::Value::Exact(value) => {
            crate::string::StringFieldValue::Exact(value)
        }
        grpc_ds::string_field_value::Value::Distribution(v) => {
            crate::string::StringFieldValue::Distribution(
                v.values.into_iter().map(|v| (v.0, v.1.into())).collect(),
            )
        }
    }
}

pub fn convert_numeric_field_value(
    value: grpc_ds::numeric_field_value::Value,
) -> crate::numeric::NumericFieldValue {
    match value {
        grpc_ds::numeric_field_value::Value::Exact(value) => {
            crate::numeric::NumericFieldValue::Exact(value.value.into())
        }
        grpc_ds::numeric_field_value::Value::Normal(n) => {
            crate::numeric::NumericFieldValue::Normal {
                sigma: n.sigma.into(),
                mean: n.mean.into(),
            }
        }
        grpc_ds::numeric_field_value::Value::Uniform(u) => {
            crate::numeric::NumericFieldValue::Uniform {
                min: u.min.into(),
                max: u.max.into(),
            }
        }
        grpc_ds::numeric_field_value::Value::Combination(v) => {
            crate::numeric::NumericFieldValue::Combination {
                components: v
                    .values
                    .into_iter()
                    .map(|v| convert_numeric_field_value(v.value.unwrap()))
                    .collect(),
                scaling_factor: v.scaling_factor.into(),
            }
        }
    }
}

pub struct GrpcDataSource {
    client: grpc_ds::data_source_client::DataSourceClient<tonic::transport::Channel>,
}

#[async_trait]
impl DataSource for GrpcDataSource {
    async fn search(&self, query: String) -> Option<SearchResult> {
        let response = self
            .client
            .clone()
            .search(grpc_ds::SearchRequest { query })
            .await
            .ok()
            .unwrap();
        let result = response.into_inner();
        Some(SearchResult {
            numeric_fields: result
                .numeric_fields
                .into_iter()
                .map(|v| (v.0, convert_numeric_field_value(v.1.value.unwrap())))
                .collect(),
            string_fields: result
                .string_fields
                .into_iter()
                .map(|v| (v.0, convert_string_field_value(v.1.value.unwrap())))
                .collect(),
        })
    }
}

impl GrpcDataSource{
    pub async fn new(address: String) -> Self {
        let client = grpc_ds::data_source_client::DataSourceClient::connect(address).await.unwrap();
        Self { client }
    }
}