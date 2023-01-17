pub mod datasource;
pub mod datasources;
pub mod numeric;
pub mod search_engine;
pub mod search_engine_config;
pub mod search_result;
pub mod string;
use std::{ path::PathBuf};
pub mod graphql;
use async_graphql_poem::GraphQL;
use poem::{get, listener::TcpListener, Route, Server};

use clap::Parser;
use graphql::get_schema;
use search_engine_config::Config;


use crate::graphql::graphiql;

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

    let schema = get_schema(engine).unwrap();
    let app = Route::new().at("/", get(graphiql).post(GraphQL::new(schema)));

    println!("GraphiQL IDE: http://localhost:8000");
    Server::new(TcpListener::bind("127.0.0.1:8000"))
        .run(app)
        .await
        .unwrap();
}
