use serde_json::json;
use quyc_client::jsonpath::core_evaluator::evaluator::{
    CoreJsonPathEvaluator, EvaluationEngine, TimeoutHandler, TimeoutConfig
};
use quyc_client::jsonpath::core_evaluator::evaluator::descendant_operations::DescendantOperations;

#[test]
fn test_evaluator_integration() {
    let evaluator = CoreJsonPathEvaluator::new("$.store.book[0].title").unwrap();
    let json = json!({
        "store": {
            "book": [
                {"title": "Book 1"},
                {"title": "Book 2"}
            ]
        }
    });

    let results = evaluator.evaluate(&json).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json!("Book 1"));
}

#[test]
fn test_timeout_integration() {
    let evaluator = CoreJsonPathEvaluator::new("$.simple").unwrap();
    let json = json!({"simple": "value"});

    let config = TimeoutConfig {
        timeout_duration: std::time::Duration::from_millis(100),
        log_timeouts: false,
    };

    let results = evaluator.evaluate_with_config(&json, config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json!("value"));
}#[test]
fn test_property_operations_integration() {
    let evaluator = CoreJsonPathEvaluator::new("$.test").unwrap();
    let json = json!({"nested": {"prop": "value"}});

    let results = evaluator
        .evaluate_property_path(&json, "nested.prop")
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json!("value"));

    let recursive_results = evaluator.find_property_recursive(&json, "prop");
    assert_eq!(recursive_results.len(), 1);
    assert_eq!(recursive_results[0], json!("value"));
}

#[test]
fn test_descendant_operations_integration() {
    let json = json!({
        "store": {
            "book": [
                {"title": "Book 1"},
                {"title": "Book 2"}
            ]
        }
    });

    let mut results = Vec::new();
    DescendantOperations::collect_all_descendants_owned(&json, &mut results);

    // Should collect all nested values
    assert!(!results.is_empty());
    assert!(results.iter().any(|v| v == &json!("Book 1")));
    assert!(results.iter().any(|v| v == &json!("Book 2")));
}#[test]
fn test_evaluation_engine_integration() {
    let evaluator = CoreJsonPathEvaluator::new("$..title").unwrap();
    let json = json!({
        "title": "Root Title",
        "store": {
            "book": [
                {"title": "Book 1"},
                {"title": "Book 2"}
            ]
        }
    });

    let results = EvaluationEngine::evaluate_expression(&evaluator, &json).unwrap();
    assert_eq!(results.len(), 3);
    assert!(results.contains(&json!("Root Title")));
    assert!(results.contains(&json!("Book 1")));
    assert!(results.contains(&json!("Book 2")));
}

#[test]
fn test_complex_expression_integration() {
    let evaluator = CoreJsonPathEvaluator::new("$.store.*").unwrap();
    let json = json!({
        "store": {
            "book": [{"title": "Book"}],
            "bicycle": {"color": "red"}
        }
    });

    let results = evaluator.evaluate(&json).unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|v| v.is_array()));
    assert!(results.iter().any(|v| v.is_object()));
}#[test]
fn test_error_handling_integration() {
    let result = CoreJsonPathEvaluator::new("$.[invalid");
    assert!(result.is_err());
}

#[test]
fn test_empty_results_integration() {
    let evaluator = CoreJsonPathEvaluator::new("$.nonexistent").unwrap();
    let json = json!({"existing": "value"});

    let results = evaluator.evaluate(&json).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_multiple_expressions_integration() {
    let expressions = vec!["$.a", "$.b", "$.c"];
    let json = json!({"a": 1, "b": 2, "c": 3});

    let results = EvaluationEngine::evaluate_multiple(&expressions, &json).unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0], vec![json!(1)]);
    assert_eq!(results[1], vec![json!(2)]);
    assert_eq!(results[2], vec![json!(3)]);
}

#[test]
fn test_complexity_estimation_integration() {
    let evaluator = CoreJsonPathEvaluator::new("$..book[*].title").unwrap();
    let complexity = EvaluationEngine::estimate_evaluation_complexity(evaluator.selectors());

    // Should be high complexity due to recursive descent and wildcard
    assert!(complexity > 50);
}

#[test]
fn test_timeout_estimation_integration() {
    let simple_expr = "$.simple";
    let complex_expr = "$..book[*]";

    let simple_timeout = TimeoutHandler::recommended_timeout(simple_expr);
    let complex_timeout = TimeoutHandler::recommended_timeout(complex_expr);

    assert!(complex_timeout > simple_timeout);
}