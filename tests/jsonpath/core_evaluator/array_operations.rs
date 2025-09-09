//! Array operation tests for JSONPath core evaluator
//!
//! Tests for array wildcards, indexing, negative indexing, and array-specific
//! JSONPath operations ensuring proper RFC 9535 compliance.

use serde_json::json;
use quyc::json_path::core_evaluator::evaluator::CoreJsonPathEvaluator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_wildcard() {
        let evaluator = CoreJsonPathEvaluator::new("$.store.book[*]")
            .expect("Failed to create evaluator for array wildcard '$.store.book[*]'");
        let json = json!({
            "store": {
                "book": [
                    {"title": "Book 1"},
                    {"title": "Book 2"}
                ]
            }
        });
        let results = evaluator
            .evaluate(&json)
            .expect("Failed to evaluate array wildcard against JSON");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_negative_indexing_fix() {
        println!("=== Testing negative indexing fix ===");

        let array_json = json!({
            "items": [10, 20, 30, 40]
        });

        // Test negative index [-1]
        println!("Test: Negative index [-1]");
        let evaluator = CoreJsonPathEvaluator::new("$.items[-1]")
            .expect("Failed to create evaluator for negative index '$.items[-1]'");
        let results = evaluator
            .evaluate(&array_json)
            .expect("Failed to evaluate negative index [-1] against JSON");
        println!("$.items[-1] -> {} results: {:?}", results.len(), results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!(40)); // Should be last element

        // Test negative index [-2]
        println!("Test: Negative index [-2]");
        let evaluator = CoreJsonPathEvaluator::new("$.items[-2]")
            .expect("Failed to create evaluator for negative index '$.items[-2]'");
        let results = evaluator
            .evaluate(&array_json)
            .expect("Failed to evaluate negative index [-2] against JSON");
        println!("$.items[-2] -> {} results: {:?}", results.len(), results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!(30)); // Should be second-to-last element
    }

    #[test]
    fn test_array_union_selector() {
        println!("Test: Array union selector");
        let array_json = json!({
            "items": [10, 20, 30, 40]
        });
        let evaluator = CoreJsonPathEvaluator::new("$.items[0,1,0,2]")
            .expect("Failed to create evaluator for array union selector '$.items[0,1,0,2]'");
        let results = evaluator
            .evaluate(&array_json)
            .expect("Failed to evaluate array union selector against JSON");
        println!(
            "$.items[0,1,0,2] -> {} results: {:?}",
            results.len(),
            results
        );
        // Should preserve duplicates per RFC 9535
        assert_eq!(results.len(), 4);
        assert_eq!(results[0], json!(10)); // items[0]
        assert_eq!(results[1], json!(20)); // items[1]
        assert_eq!(results[2], json!(10)); // items[0] again
        assert_eq!(results[3], json!(30)); // items[2]
    }

    #[test]
    fn test_array_slice_operations() {
        let array_json = json!({
            "data": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        });

        // Test basic slice [1:4]
        let evaluator = CoreJsonPathEvaluator::new("$.data[1:4]")
            .expect("Failed to create evaluator for array slice");
        let results = evaluator
            .evaluate(&array_json)
            .expect("Failed to evaluate array slice");
        assert_eq!(results.len(), 3);
        assert_eq!(results, vec![json!(2), json!(3), json!(4)]);
    }
}