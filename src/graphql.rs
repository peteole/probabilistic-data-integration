use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
// use async_graphql_poem::GraphQL;
use poem::{handler, web::Html, IntoResponse};
#[handler]
pub async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/").finish())
}

use crate::{
    numeric::NumericFieldValue,
    search_engine::{SearchEngine, SearchResponse},
};
use async_graphql::{dynamic::*, Value};
type KeyValuePair = (String, f64);

pub fn get_schema(search_engine: Arc<SearchEngine>) -> Result<Schema, SchemaError> {
    let key_value_pair = Object::new("KeyValuePair")
        .field(Field::new(
            "key",
            TypeRef::named_nn(TypeRef::STRING),
            |ctx| {
                FieldFuture::new(async move {
                    let data = ctx.parent_value.try_downcast_ref::<KeyValuePair>()?;
                    Ok(Some(Value::from(data.0.clone())))
                })
            },
        ))
        .field(Field::new(
            "value",
            TypeRef::named_nn(TypeRef::FLOAT),
            |ctx| {
                FieldFuture::new(async move {
                    let data = ctx.parent_value.try_downcast_ref::<KeyValuePair>()?;
                    Ok(Some(Value::from(data.1)))
                })
            },
        ));
    let gql_numeric_field_value = get_numeric_field_value();
    let mut search_result_builder = Object::new("SearchResult");
    for (field_name, field) in search_engine.search_fields.iter() {
        let field_name = field_name.clone();
        let field = field.clone();
        match field.field_type.clone() {
            crate::search_engine::FieldType::Float { unit } => {
                search_result_builder = search_result_builder.field(
                    Field::new(
                        field_name.clone(),
                        TypeRef::named("NumericFieldValue"),
                        move |ctx| {
                            let field_name = field_name.clone();
                            FieldFuture::new(async move {
                                let data = ctx.parent_value.try_downcast_ref::<SearchResponse>()?;
                                let result = match data.fields.get(&field_name as &str) {
                                    Some(v) => v,
                                    None => return Ok(None),
                                };
                                match &result.1 {
                                    crate::search_engine::FieldValue::String(_s) => {
                                        return Ok(None)
                                    }
                                    crate::search_engine::FieldValue::Numeric(n) => {
                                        return Ok(Some(FieldValue::boxed_any(Box::new(n.clone()))))
                                    }
                                };
                            })
                        },
                    )
                    .description(format!("{} ({})", field.description, unit)),
                );
            }
            crate::search_engine::FieldType::String => {
                search_result_builder = search_result_builder.field(
                    Field::new(
                        field_name.clone(),
                        TypeRef::named_nn_list(key_value_pair.type_name()),
                        move |ctx| {
                            let field_name = field_name.clone();
                            FieldFuture::new(async move {
                                let data = ctx.parent_value.try_downcast_ref::<SearchResponse>()?;
                                let result = match data.fields.get(&field_name as &str) {
                                    Some(v) => v,
                                    None => return Ok(None),
                                };

                                match &result.1 {
                                    crate::search_engine::FieldValue::String(s) => {
                                        return match s {
                                            crate::string::StringFieldValue::Exact(e) => {
                                                return Ok(Some(FieldValue::list(vec![
                                                    FieldValue::boxed_any(Box::new((
                                                        e.clone(),
                                                        1.0,
                                                    ))),
                                                ])));
                                            }
                                            crate::string::StringFieldValue::Distribution(r) => {
                                                return Ok(Some(FieldValue::list(r.iter().map(
                                                    |(k, v)| {
                                                        FieldValue::boxed_any(Box::new((
                                                            k.clone(),
                                                            v.clone(),
                                                        )))
                                                    },
                                                ))));
                                            }
                                            _ => Ok(None),
                                        };
                                    }
                                    _ => return Ok(None),
                                };
                            })
                        },
                    )
                    .description(format!(
                        "{}. Result is a distribution, mapping each key to a probability",
                        field.description
                    )),
                )
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
                    let search_engine = ctx.data::<Arc<SearchEngine>>()?;
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
    let schema = Schema::build(root_builder.type_name(), None, None)
        .register(root_builder)
        .register(search_result_builder)
        .register(gql_numeric_field_value)
        .register(key_value_pair)
        .data(search_engine)
        .finish();
    return schema;
}

struct NumericFieldGetter<'a, T> {
    name: &'a str,
    reducer: fn(T) -> Option<f64>,
    description: &'a str,
}

fn numeric_value_field<T>(field_getter: NumericFieldGetter<T>) -> Field
where
    T: Clone + 'static,
{
    Field::new(
        field_getter.name,
        TypeRef::named(TypeRef::FLOAT),
        move |ctx| {
            FieldFuture::new(async move {
                let data = ctx.parent_value.try_downcast_ref::<T>()?;
                Ok((field_getter.reducer)(data.to_owned()).map(Value::from))
            })
        },
    )
    .description(field_getter.description)
}
pub fn get_numeric_field_value() -> Object {
    const TYPENAME: &str = "NumericFieldValue";
    Object::new(TYPENAME)
        .description("Distribution over a numeric field")
        .field(numeric_value_field(NumericFieldGetter {
            name: "mu",
            reducer: |nfv: NumericFieldValue| Some(nfv.mu()),
            description: "Mean of the field",
        }))
        .field(numeric_value_field(NumericFieldGetter {
            name: "sigma",
            reducer: |nfv: NumericFieldValue| Some(nfv.sigma()),
            description: "Standard deviation of the field",
        }))
        .field(numeric_value_field(NumericFieldGetter {
            name: "exact",
            reducer: |nfv: NumericFieldValue| match nfv {
                NumericFieldValue::Exact(e) => Some(e),
                _ => None,
            },
            description: "Exact value if distribution of type \"exact\", null else",
        }))
        .field(numeric_value_field(NumericFieldGetter {
            name: "normal_mean",
            reducer: |nfv: NumericFieldValue| match nfv {
                NumericFieldValue::Normal { mu: m, sigma: _ } => Some(m),
                _ => None,
            },
            description: "Mean of normal distribution if it is a normal distribution, null else",
        }))
        .field(numeric_value_field(NumericFieldGetter {
            name: "normal_sigma",
            reducer: |nfv: NumericFieldValue| match nfv {
                NumericFieldValue::Normal { sigma: s,.. } => Some(s),
                _ => None,
            },
            description: "Standard deviation of normal distribution if it is a normal distribution, null else",
        }))
        .field(numeric_value_field(NumericFieldGetter {
            name: "uniform_min",
            reducer: |nfv: NumericFieldValue| match nfv {
                NumericFieldValue::Uniform { min:m,max:_} => Some(m),
                _ => None,
            },
            description: "Minimum value of uniform distribution if it is a uniform distribution, null else",
        }))
        .field(numeric_value_field(NumericFieldGetter {
            name: "uniform_max",
            reducer: |nfv: NumericFieldValue| match nfv {
                NumericFieldValue::Uniform { min:_,max:m} => Some(m),
                _ => None,
            },
            description: "Minimum value of uniform distribution if it is a uniform distribution, null else",
        }))
        .field(
            Field::new("combination_components", TypeRef::named_nn_list(TYPENAME), |ctx|{
                FieldFuture::new(async move{
                    let data = ctx.parent_value.try_downcast_ref::<NumericFieldValue>()?;
                    match data{
                        NumericFieldValue::Combination { components, .. } => {
                            Ok(Some(FieldValue::list(components.iter().map(|c|FieldValue::borrowed_any(c)))))
                        },
                        _=>Ok(None),
                    }
                })
            })
            .description("Combination field values if the distribution is a combination of several other field values.")
        )
        .field(numeric_value_field(NumericFieldGetter {
            name: "combination_scaling_factor",
            reducer: |nfv: NumericFieldValue| match nfv {
                NumericFieldValue::Combination { scaling_factor,components:_, mu: _, sigma: _ } => Some(scaling_factor),
                _ => None,
            },
            description: "Inverse of integral of the product of all component probabilities.",
        }))
        .field(Field::new(
            "probability_density",
            TypeRef::named_nn_list_nn(TypeRef::FLOAT),
            move |ctx| {
                FieldFuture::new(async move {
                    let data = ctx.parent_value.try_downcast_ref::<NumericFieldValue>()?;
                    let query = ctx.args.try_get("x")?;
                    let list=query.list()?;
                    let y = list.iter().filter_map(|x|x.f64().ok()).
                        map(|x|data.get_value(x)).map(Value::from);
                    Ok(Some(FieldValue::list(y)))
                })
            },
        )
        .argument(InputValue::new("x",TypeRef::named_nn_list_nn(TypeRef::FLOAT)).description("Location to calculate probability density at"))
        .description("probability density at the given values")
        )
}
