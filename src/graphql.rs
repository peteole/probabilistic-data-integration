use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
// use async_graphql_poem::GraphQL;
use poem::{get, handler, web::Html, IntoResponse};
#[handler]
pub async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/").finish())
}

use async_graphql::{dynamic::*, Value};
use tag_search::{search_engine::SearchEngine, search_result::SearchResult};

pub fn get_schema(search_engine: &'static SearchEngine) -> Result<Schema, SchemaError> {
    let mut search_result_builder = Object::new("SearchResult");
    for (field_name, field) in search_engine.search_fields.iter() {
        match field.field_type.clone() {
            tag_search::search_engine::FieldType::Float { unit } => {
                search_result_builder = search_result_builder.field(Field::new(
                    field_name.clone(),
                    TypeRef::named_nn(TypeRef::FLOAT),
                    move |ctx| {
                        FieldFuture::new(async move {
                            let data = ctx.parent_value.try_downcast_ref::<SearchResult>()?;
                            let result = data.numeric_fields.get(&field_name as &str);
                            Ok(result.map(|r| Value::from(r.mean())))
                        })
                    },
                ));
            }
            tag_search::search_engine::FieldType::String => {
                search_result_builder = search_result_builder.field(Field::new(
                    field_name.clone(),
                    TypeRef::named_nn(TypeRef::STRING),
                    move |ctx| {
                        FieldFuture::new(async move {
                            let data = ctx.parent_value.try_downcast_ref::<SearchResult>()?;
                            let result = data.string_fields.get(&field_name as &str);
                            match result {
                                Some(r) => match r {
                                    tag_search::string::StringFieldValue::Exact(e) => {
                                        return Ok(Some(Value::from(e.clone())));
                                    }
                                    // tag_search::string::StringFieldValue::Distribution(r) => {
                                    //     return Ok(Some(FieldValue::list(r.iter())));
                                    // }
                                    ,
                                    _ => Ok(None),
                                },
                                None => Ok(None),
                            }
                        })
                    },
                ));
            }
        }
    }

    let root_builder = Object::new("Query").field(
        Field::new(
            "search",
            TypeRef::named_nn(search_result_builder.type_name()),
            move |ctx| {
                FieldFuture::new(async move {
                    let query = ctx.args.try_get("query")?;
                    match query.string() {
                        Ok(q) => {
                            let cloned=q.to_owned();
                            let result = search_engine.search(cloned.into()).await;
                            Ok(Some(FieldValue::boxed_any(Box::new(result))))
                        }
                        Err(_) => Err(async_graphql::Error::new("")),
                    }
                })
            },
        )
        .argument(InputValue::new("query", TypeRef::named_nn(TypeRef::STRING))),
    );

    // let query = Object::new("Query").field(Field::new(
    //     "value",
    //     TypeRef::named_nn(TypeRef::STRING),
    //     |ctx| FieldFuture::new(async move { Ok(Some(Value::from("Hello world"))) }),
    // ));

    let schema = Schema::build(root_builder.type_name(), None, None)
        .register(root_builder)
        .finish();
    return schema;
}
