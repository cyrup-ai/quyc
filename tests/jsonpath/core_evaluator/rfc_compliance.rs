//! RFC 9535 compliance tests for JSONPath core evaluator
//!
//! Tests ensuring strict compliance with RFC 9535 JSONPath specification,
//! including normative examples and edge case behaviors.

use serde_json::json;
use quyc::json_path::core_evaluator::evaluator::CoreJsonPathEvaluator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc_9535_basic_examples() {
        // RFC 9535 Example 1: Root selector
        let json = json!({"hello": "world"});
        let evaluator = CoreJsonPathEvaluator::new("$")
            .expect("Failed to create evaluator for RFC root example");
        let results = evaluator.evaluate(&json)
            .expect("Failed to evaluate RFC root example");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json);
    }

    #[test]
    fn test_rfc_9535_property_selector() {
        // RFC 9535 Example: Property selector
        let json = json!({"store": {"book": [{"title": "Book 1"}]}});
        let evaluator = CoreJsonPathEvaluator::new("$.store")
            .expect("Failed to create evaluator for RFC property example");
        let results = evaluator.evaluate(&json)
            .expect("Failed to evaluate RFC property example");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!({"book": [{"title": "Book 1"}]}));
    }

    #[test]
    fn test_rfc_9535_array_index() {
        // RFC 9535 Example: Array index selector
        let json = json!({"numbers": [1, 2, 3, 4, 5]});
        let evaluator = CoreJsonPathEvaluator::new("$.numbers[2]")
            .expect("Failed to create evaluator for RFC array index example");
        let results = evaluator.evaluate(&json)
            .expect("Failed to evaluate RFC array index example");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!(3));
    }

    #[test]
    fn test_rfc_9535_wildcard_selector() {
        // RFC 9535 Example: Wildcard selector
        let json = json!({"data": {"a": 1, "b": 2, "c": 3}});
        let evaluator = CoreJsonPathEvaluator::new("$.data.*")
            .expect("Failed to create evaluator for RFC wildcard example");
        let results = evaluator.evaluate(&json)
            .expect("Failed to evaluate RFC wildcard example");
        assert_eq!(results.len(), 3);
        assert!(results.contains(&json!(1)));
        assert!(results.contains(&json!(2)));
        assert!(results.contains(&json!(3)));
    }

    #[test]
    fn test_rfc_9535_union_selector() {
        // RFC 9535 Example: Union selector with duplicate preservation
        let json = json!({"data": {"x": 1, "y": 2, "z": 3}});
        let evaluator = CoreJsonPathEvaluator::new("$.data['x','y','x']")
            .expect("Failed to create evaluator for RFC union example");
        let results = evaluator.evaluate(&json)
            .expect("Failed to evaluate RFC union example");
        
        // RFC 9535 requires duplicate preservation
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], json!(1)); // x
        assert_eq!(results[1], json!(2)); // y  
        assert_eq!(results[2], json!(1)); // x again
    }

    #[test]
    fn test_rfc_9535_recursive_descent() {
        // RFC 9535 Example: Recursive descent
        let json = json!({
            "store": {
                "book": [
                    {"author": "Author 1", "title": "Book 1"},
                    {"author": "Author 2", "title": "Book 2"}
                ],
                "bicycle": {"color": "red"}
            }
        });
        
        let evaluator = CoreJsonPathEvaluator::new("$..author")
            .expect("Failed to create evaluator for RFC recursive descent example");
        let results = evaluator.evaluate(&json)
            .expect("Failed to evaluate RFC recursive descent example");
        
        assert_eq!(results.len(), 2);
        assert!(results.contains(&json!("Author 1")));
        assert!(results.contains(&json!("Author 2")));
    }

    #[test]
    fn test_rfc_9535_error_handling() {
        // Test that invalid expressions are properly rejected
        let result = CoreJsonPathEvaluator::new("$[invalid");
        assert!(result.is_err(), "Invalid JSONPath should be rejected");
        
        let result = CoreJsonPathEvaluator::new("$..");
        assert!(result.is_err(), "Incomplete recursive descent should be rejected");
    }
}