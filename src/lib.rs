pub mod datasource;
pub mod datasources;
pub mod numeric;
pub mod search_engine;
pub mod search_result;
pub mod string;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        datasource::MockDataSource,
        datasources::open_food_facts::OpenFoodFactsDataSource,
        search_engine::{Field, FieldType, FieldValue, SearchEngine},
    };

    #[tokio::test]
    async fn it_works() {
        
    }
}
