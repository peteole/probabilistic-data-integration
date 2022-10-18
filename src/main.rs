use tag_search::{SearchEngine, MockDataSource};
fn main() {
    let engine = SearchEngine {
        data_sources: vec![Box::new(MockDataSource {}),Box::new(MockDataSource {})],
    };
    let result = engine.search("apple".to_string());
    println!("{:?}", result);
}