use std::collections::HashMap;

use poem::{
    get, handler,
    listener::TcpListener,
    middleware::AddData,
    web::{Data, Json},
    EndpointExt, Route, Server,
};
use poem_openapi::{param::Path, OpenApi, ApiResponse, Object};
use tag_search::search_engine::{SearchEngine, SearchResponse, Field, FieldValue};

use crate::search_engine;

struct SearchEngineApi {
    engine: search_engine::SearchEngine,
}
#[derive(Object)]
struct JsonSearchResponse{
    fields: HashMap<String, Field>,
}
#[derive(ApiResponse)]
enum Response {
    #[oai(status = 200)]
    Ok(Json<search_engine::SearchResponse>),
    #[oai(status = 404)]
    BadRequest,
}

#[OpenApi]
impl SearchEngineApi {
    #[oai(path = "/search/:query", method = "post")]
    async fn search(&self, engine: Data<&SearchEngine>, query: Path<String>) -> Response {
        engine.search(query.0).await
    }
}
