use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
// use async_graphql_poem::GraphQL;
use poem::{get, handler, web::Html, IntoResponse};
#[handler]
pub async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/").finish())
}

use async_graphql::{dynamic::*, Value};
use tag_search::{
    search_engine::{SearchEngine, SearchFields, SearchResponse},
    search_result::SearchResult,
};

pub fn get_schema(search_engine: SearchEngine) -> Result<Schema, SchemaError> {
    let mut search_result_builder = Object::new("SearchResult");
    // for i in ["a".to_string(),"b".to_string()] {
    //     search_result_builder = search_result_builder.field(Field::new(
    //         format!("field{}", i),
    //         TypeRef::named_nn(TypeRef::STRING),
    //         move |ctx| {
    //             FieldFuture::new(async move {
    //                 Ok(Some(Value::from(i)))
    //             })
    //         },
    //     ));
    // }
    for (field_name, field) in search_engine.search_fields.iter() {
        let field_name = field_name.clone();
        let field = field.clone();
        match field.field_type.clone() {
            tag_search::search_engine::FieldType::Float { unit } => {
                search_result_builder = search_result_builder.field(
                    Field::new(
                        field_name.clone(),
                        TypeRef::named_nn(TypeRef::FLOAT),
                        move |ctx| {
                            let field_name = field_name.clone();
                            FieldFuture::new(async move {
                                println!("field_name={}", field_name);
                                let data = ctx.parent_value.try_downcast_ref::<SearchResponse>()?;
                                let result = data
                                    .fields
                                    .get(&field_name as &str)
                                    .ok_or("field not found")?;
                                match &result.1 {
                                    tag_search::search_engine::FieldValue::String(s) => {
                                        return Ok(None)
                                    }
                                    tag_search::search_engine::FieldValue::Numeric(n) => {
                                        return Ok(Some(Value::from(n.mean())))
                                    }
                                };
                            })
                        },
                    )
                    .description(format!("{} ({})", field.description, unit)),
                );
            }
            tag_search::search_engine::FieldType::String => {
                search_result_builder = search_result_builder.field(Field::new(
                    field_name.clone(),
                    TypeRef::named_nn(TypeRef::STRING),
                    move |ctx| {
                        let field_name = field_name.clone();
                        FieldFuture::new(async move {
                            let data = ctx.parent_value.try_downcast_ref::<SearchResponse>()?;
                            let result = data
                                .fields
                                .get(&field_name as &str)
                                .ok_or("field not found")?;

                            match &result.1 {
                                tag_search::search_engine::FieldValue::String(s) => {
                                    return match s {
                                        tag_search::string::StringFieldValue::Exact(e) => {
                                            return Ok(Some(Value::from(e.clone())));
                                        }
                                        // tag_search::string::StringFieldValue::Distribution(r) => {
                                        //     return Ok(Some(FieldValue::list(r.iter())));
                                        // }
                                        ,
                                        _ => Ok(None),
                                    };
                                }
                                _ => return Ok(None),
                            };
                        })
                    },
                ));
            }
        }
    }

    let root_builder = Object::new("Query").field(
        Field::new(
            "search",
            TypeRef::named(search_result_builder.type_name()),
            move |ctx| {
                FieldFuture::new(async move {
                    let query = ctx.args.try_get("query")?;
                    let search_engine = ctx.data::<SearchEngine>()?;
                    match query.string() {
                        Ok(q) => {
                            let cloned = q.to_owned();
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
        .register(search_result_builder)
        .data(search_engine)
        .finish();
    return schema;
}
