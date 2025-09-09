//! Filter expression tests for JSONPath core evaluator
//!
//! Tests for complex filter expressions, bracket notation, and advanced
//! JSONPath query patterns ensuring RFC 9535 compliance.

use serde_json::json;
use quyc::json_path::core_evaluator::evaluator::CoreJsonPathEvaluator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bracket_notation() {
        println!("Test: Bracket notation");
        let test_json = json!({
            "data": {
                "x": 42,
                "y": 24
            }
        });

        let evaluator = CoreJsonPathEvaluator::new("$.data['x']")
            .expect("Failed to create evaluator for bracket notation '$.data['x']'");
        let results = evaluator
            .evaluate(&test_json)
            .expect("Failed to evaluate bracket notation against JSON");
        println!("$.data['x'] -> {} results: {:?}", results.len(), results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!(42));
    }

    #[test]
    fn test_multi_selector_duplicates() {
        println!("Test: Multi-selector (should show duplicates)");
        let test_json = json!({
            "data": {
                "x": 42,
                "y": 24
            }
        });

        let evaluator = CoreJsonPathEvaluator::new("$.data['x','x','y','x']")
            .expect("Failed to create evaluator for multi-selector '$.data['x','x','y','x']'");
        let results = evaluator
            .evaluate(&test_json)
            .expect("Failed to evaluate multi-selector against JSON");
        println!(
            "$.data['x','x','y','x'] -> {} results: {:?}",
            results.len(),
            results
        );
        
        // RFC 9535 requires duplicate preservation
        assert_eq!(results.len(), 4);
        assert_eq!(results[0], json!(42)); // x
        assert_eq!(results[1], json!(42)); // x again
        assert_eq!(results[2], json!(24)); // y
        assert_eq!(results[3], json!(42)); // x again
    }

    #[test]
    fn test_filter_with_comparison() {
        let json = json!({
            "products": [
                {"name": "Product A", "price": 10},
                {"name": "Product B", "price": 25},
                {"name": "Product C", "price": 15}
            ]
        });

        let evaluator = CoreJsonPathEvaluator::new("$.products[?@.price > 12]")
            .expect("Failed to create evaluator for price filter");
        let results = evaluator
            .evaluate(&json)
            .expect("Failed to evaluate price filter");
        
        assert_eq!(results.len(), 2);
        // Should contain Product B and Product C
        assert!(results.iter().any(|v| v.get("name").unwrap() == "Product B"));
        assert!(results.iter().any(|v| v.get("name").unwrap() == "Product C"));
    }

    #[test]
    fn test_complex_filter_expression() {
        let json = json!({
            "library": {
                "books": [
                    {"title": "Book 1", "author": "Author A", "year": 2020},
                    {"title": "Book 2", "author": "Author B", "year": 2019},
                    {"title": "Book 3", "author": "Author A", "year": 2021}
                ]
            }
        });

        let evaluator = CoreJsonPathEvaluator::new("$.library.books[?@.author == 'Author A']")
            .expect("Failed to create evaluator for author filter");
        let results = evaluator
            .evaluate(&json)
            .expect("Failed to evaluate author filter");
        
        assert_eq!(results.len(), 2);
        for result in &results {
            assert_eq!(result.get("author").unwrap(), "Author A");
        }
    }
}