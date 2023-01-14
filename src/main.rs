pub mod datasource;
pub mod datasources;
pub mod numeric;
pub mod search_engine;
pub mod search_engine_config;
pub mod search_result;
pub mod string;
use std::path::PathBuf;

use clap::Parser;
use search_engine_config::Config;

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
}
