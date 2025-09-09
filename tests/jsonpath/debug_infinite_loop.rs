//! Debug test for infinite loop patterns

use quyc::json_path::{CoreJsonPathEvaluator, JsonPathParser};
use serde_json::json;
use std::time::{Duration, Instant};

#[test]
fn debug_infinite_loop_patterns() {
    env_logger::init();
    log::debug!("Testing JSONPath patterns that may cause infinite loops...");

    let bookstore_json = json!({
        "store": {
            "book": [
                {
                    "category": "reference",
                    "author": "Nigel Rees",
                    "title": "Sayings of the Century",
                    "price": 8.95
                },
                {
                    "category": "fiction",
                    "author": "Evelyn Waugh",
                    "title": "Sword of Honour",
                    "price": 12.99
                },
                {
                    "category": "fiction",
                    "author": "Herman Melville",
                    "title": "Moby Dick",
                    "isbn": "0-553-21311-3",
                    "price": 8.99
                }
            ]
        }
    });

    let problematic_patterns = vec![
        "$..*",           // Deep recursive descent
        "$..book..*",     // Nested recursive descent
        "$..[*]..*",      // Multiple recursive patterns
        "$..book[*]..*",  // Complex recursive with array access
    ];

    for pattern in problematic_patterns {
        log::debug!("Testing pattern: {}", pattern);
        
        let start_time = Instant::now();
        let timeout = Duration::from_secs(5);
        
        match JsonPathParser::compile(pattern) {
            Ok(compiled) => {
                let mut evaluator = CoreJsonPathEvaluator::new();
                
                // Test with timeout to prevent infinite loops
                let result = std::panic::catch_unwind(|| {
                    evaluator.evaluate(&compiled, &bookstore_json)
                });
                
                let elapsed = start_time.elapsed();
                
                if elapsed > timeout {
                    log::error!("Pattern {} took too long: {:?}", pattern, elapsed);
                    panic!("Pattern execution exceeded timeout");
                } else {
                    log::debug!("Pattern {} completed in {:?}", pattern, elapsed);
                    
                    match result {
                        Ok(values) => {
                            log::debug!("Found {} values", values.len());
                            assert!(values.len() < 1000, "Too many results, possible infinite expansion");
                        }
                        Err(_) => {
                            log::error!("Pattern {} caused panic", pattern);
                            panic!("Pattern caused panic during evaluation");
                        }
                    }
                }
            }
            Err(e) => {
                log::debug!("Pattern {} failed to compile: {:?}", pattern, e);
            }
        }
    }
}