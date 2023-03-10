pub mod datasource;
pub mod datasources;
pub mod numeric;
pub mod search_engine;
pub mod search_engine_config;
pub mod search_result;
pub mod string;
use std::{path::PathBuf, sync::Arc};
pub mod graphql;
use async_graphql_poem::GraphQL;
use poem::EndpointExt;
use poem::{
    get,
    listener::TcpListener,
    middleware::AddData,
    web::{Data, Path},
    Route, Server,
};

use clap::{builder::Str, Parser};
use graphql::get_schema;
use search_engine_config::Config;

use crate::graphql::graphiql;
use crate::search_engine::SearchEngine;

/// Data integration engine
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to a config file containing the fields available on the search engine and the data sources. Defaults to `config.yaml`
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let fields_path = args.config.unwrap_or_else(|| PathBuf::from("config.yaml"));
    let configuration: Config = config::Config::builder()
        .add_source(config::File::from(fields_path))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap();
    println!("{:?}", configuration);
    let engine = configuration.to_search_engine().await;
    let result = engine.search("apple".to_string()).await;
    println!("{:?}", result);
    let arced_engine = Arc::new(engine);
    let schema = get_schema(arced_engine.clone()).unwrap();
    let app = Route::new()
        .at("/", get(graphiql).post(GraphQL::new(schema)))
        .at("/search/::query", http_search)
        .with(AddData::new(arced_engine));

    println!("GraphiQL IDE: http://localhost:8000");
    Server::new(TcpListener::bind("127.0.0.1:8000"))
        .run(app)
        .await
        .unwrap();
}
use poem::web::Json;

#[poem::handler]
async fn http_search(
    search_engine: Data<&Arc<SearchEngine>>,
    Path(query): Path<String>,
) -> Json<serde_json::Value> {
    let result = search_engine.search(query).await;
    Json(serde_json::to_value(result).unwrap())
}
