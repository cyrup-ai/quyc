//! Debug test for @ execution

use quyc::json_path::JsonArrayStream;
use bytes::Bytes;

#[test]
fn debug_at_execution() {
    env_logger::init();
    log::debug!("Testing @ execution in JSONPath expressions...");
    
    const TEST_JSON: &str = r#"{
      "store": {
        "books": [
          {
            "id": 1,
            "name": "Book One",
            "value": 10.5,
            "active": true,
            "metadata": {"category": "fiction", "pages": 300}
          },
          {
            "id": 2,
            "name": "Book Two", 
            "value": 25.0,
            "active": false,
            "metadata": {"category": "science", "pages": 450}
          },
          {
            "id": 3,
            "name": "Book Three",
            "value": 15.75,
            "active": true,
            "metadata": null
          }
        ]
      }
    }"#;
    
    let test_expressions = vec![
        ("$.store.books[?@.metadata == null]", 1, "Null property access"),
        ("$.store.books[?@.active == true]", 2, "Boolean property access"),
        ("$.store.books[?@.id > 1]", 2, "Numeric comparison"),
    ];
    
    for (expr, expected_count, description) in test_expressions {
        log::debug!("Testing: {} (expected: {})", expr, expected_count);
        let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
        let chunk = Bytes::from(TEST_JSON);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        log::debug!("Got {} results", results.len());
        
        if results.len() != expected_count {
            log::error!("MISMATCH! Expected {} but got {}", expected_count, results.len());
            for (i, result) in results.iter().enumerate() {
                log::debug!("Result {}: {:?}", i, result);
            }
        }
        
        assert_eq!(results.len(), expected_count, "{}", description);
    }
}