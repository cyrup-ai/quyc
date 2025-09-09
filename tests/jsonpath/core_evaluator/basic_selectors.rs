//! Basic selector tests for JSONPath core evaluator
//!
//! Tests for root selector, property access, and fundamental navigation patterns
//! ensuring RFC 9535 compliance for basic JSONPath operations.

use serde_json::json;
use quyc::json_path::core_evaluator::evaluator::CoreJsonPathEvaluator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_selector() {
        let evaluator = CoreJsonPathEvaluator::new("$")
            .expect("Failed to create evaluator for root selector '$'");
        let json = json!({"test": "value"});
        let results = evaluator
            .evaluate(&json)
            .expect("Failed to evaluate root selector against JSON");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json);
    }

    #[test]
    fn test_property_access() {
        let evaluator = CoreJsonPathEvaluator::new("$.store")
            .expect("Failed to create evaluator for property access '$.store'");
        let json = json!({"store": {"name": "test"}});
        let results = evaluator
            .evaluate(&json)
            .expect("Failed to evaluate property access against JSON");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!({"name": "test"}));
    }

    #[test]
    fn test_nested_property_access() {
        let evaluator = CoreJsonPathEvaluator::new("$.store.name")
            .expect("Failed to create evaluator for nested property access");
        let json = json!({"store": {"name": "test_store"}});
        let results = evaluator
            .evaluate(&json)
            .expect("Failed to evaluate nested property access");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!("test_store"));
    }

    #[test]
    fn test_simple_bicycle_access() {
        println!("\n=== DEBUG: Testing simple pattern that's timing out ===");

        let json_value = json!({
            "store": {
                "book": ["a", "b", "c", "d"],
                "bicycle": {"color": "red", "price": 19.95}
            }
        });

        let pattern = "$.store.bicycle";
        println!("Testing pattern: {}", pattern);

        match CoreJsonPathEvaluator::new(pattern) {
            Ok(evaluator) => {
                let start = std::time::Instant::now();
                match evaluator.evaluate(&json_value) {
                    Ok(results) => {
                        let elapsed = start.elapsed();
                        println!("✅ SUCCESS: Got {} results in {:?}", results.len(), elapsed);
                        for (i, result) in results.iter().enumerate() {
                            println!("  [{}]: {}", i, result);
                        }
                        assert_eq!(results.len(), 1);
                        assert_eq!(results[0], json!({"color": "red", "price": 19.95}));
                    }
                    Err(e) => {
                        println!("❌ ERROR: {}", e);
                        panic!("Evaluation failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ CREATION ERROR: {}", e);
                panic!("Evaluator creation failed: {}", e);
            }
        }
    }
}