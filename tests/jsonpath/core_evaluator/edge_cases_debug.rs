//! Edge cases and debugging tests for JSONPath core evaluator
//!
//! Tests for edge cases, duplicate preservation, error handling,
//! and debugging utilities for JSONPath evaluation.

use serde_json::json;
use quyc::json_path::core_evaluator::evaluator::CoreJsonPathEvaluator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_preservation_debug() {
        println!("=== Testing duplicate preservation ===");

        let test_json = json!({
            "data": {
                "x": 42,
                "y": 24
            }
        });

        // Test 1: Direct property access
        println!("Test 1: Direct property access");
        let evaluator = CoreJsonPathEvaluator::new("$.data.x")
            .expect("Failed to create evaluator for direct property access '$.data.x'");
        let results = evaluator
            .evaluate(&test_json)
            .expect("Failed to evaluate direct property access against JSON");
        println!("$.data.x -> {} results: {:?}", results.len(), results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!(42));
    }

    #[test]
    fn test_empty_results() {
        let json = json!({"data": {"x": 1}});
        
        let evaluator = CoreJsonPathEvaluator::new("$.nonexistent")
            .expect("Failed to create evaluator for nonexistent property");
        let results = evaluator
            .evaluate(&json)
            .expect("Failed to evaluate nonexistent property");
        
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_null_values() {
        let json = json!({
            "data": {
                "value": null,
                "number": 42
            }
        });

        let evaluator = CoreJsonPathEvaluator::new("$.data.value")
            .expect("Failed to create evaluator for null value");
        let results = evaluator
            .evaluate(&json)
            .expect("Failed to evaluate null value");
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!(null));
    }

    #[test]
    fn test_deeply_nested_structure() {
        let json = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "target": "deep_value"
                        }
                    }
                }
            }
        });

        let evaluator = CoreJsonPathEvaluator::new("$.level1.level2.level3.level4.target")
            .expect("Failed to create evaluator for deep nesting");
        let results = evaluator
            .evaluate(&json)
            .expect("Failed to evaluate deep nesting");
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!("deep_value"));
    }

    #[test]
    fn test_mixed_array_types() {
        let json = json!({
            "mixed": [1, "string", true, null, {"key": "value"}]
        });

        let evaluator = CoreJsonPathEvaluator::new("$.mixed[*]")
            .expect("Failed to create evaluator for mixed array");
        let results = evaluator
            .evaluate(&json)
            .expect("Failed to evaluate mixed array");
        
        assert_eq!(results.len(), 5);
        assert_eq!(results[0], json!(1));
        assert_eq!(results[1], json!("string"));
        assert_eq!(results[2], json!(true));
        assert_eq!(results[3], json!(null));
        assert_eq!(results[4], json!({"key": "value"}));
    }

    #[test]
    fn test_performance_timing() {
        let large_json = json!({
            "items": (0..1000).map(|i| json!({"id": i, "value": format!("item_{}", i)})).collect::<Vec<_>>()
        });

        let start = std::time::Instant::now();
        let evaluator = CoreJsonPathEvaluator::new("$.items[*].id")
            .expect("Failed to create evaluator for performance test");
        let results = evaluator
            .evaluate(&large_json)
            .expect("Failed to evaluate performance test");
        let elapsed = start.elapsed();

        println!("Performance test: {} results in {:?}", results.len(), elapsed);
        assert_eq!(results.len(), 1000);
        
        // Ensure reasonable performance (should complete in under 1 second)
        assert!(elapsed.as_secs() < 1, "Performance test took too long: {:?}", elapsed);
    }
}