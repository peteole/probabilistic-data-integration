use schemars::{schema_for};
pub mod datasource;
pub mod datasources;
pub mod numeric;
pub mod search_engine;
pub mod search_engine_config;
pub mod search_result;
pub mod string;
use std::io::prelude::*;

pub fn main() {
    let schema = schema_for!(search_engine::SearchResponse);
    let mut schema_file=std::fs::File::create("schema.json").unwrap();
    schema_file.write_all(serde_json::to_string_pretty(&schema).unwrap().as_bytes()).unwrap();
}