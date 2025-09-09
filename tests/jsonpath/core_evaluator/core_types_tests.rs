//! Tests for core types implementation
//! 
//! Extracted from src/jsonpath/core_evaluator/evaluator/core_types.rs
//! Tests core JSONPath evaluator types

use quyc_client::jsonpath::core_evaluator::evaluator::core_types::CoreJsonPathEvaluator;

#[test]
fn test_evaluator_creation() {
    let evaluator = CoreJsonPathEvaluator::new("$.store.book[0]").expect("Failed to create evaluator");
    assert_eq!(evaluator.expression(), "$.store.book[0]");
    assert!(!evaluator.selectors().is_empty());
}

#[test]
fn test_evaluator_invalid_expression() {
    let result = CoreJsonPathEvaluator::new("$.[invalid");
    assert!(result.is_err());
}

#[test]
fn test_evaluator_clone() {
    let evaluator = CoreJsonPathEvaluator::new("$.test").expect("Failed to create evaluator");
    let cloned = evaluator.clone();
    assert_eq!(evaluator.expression(), cloned.expression());
    assert_eq!(evaluator.selectors().len(), cloned.selectors().len());
}

#[test]
fn test_temp_evaluator_creation() {
    let temp = CoreJsonPathEvaluator::create_temp_evaluator("$.temp").expect("Failed to create temp evaluator");
    assert_eq!(temp.expression(), "$.temp");
}

#[test]
fn test_evaluator_debug() {
    let evaluator = CoreJsonPathEvaluator::new("$.debug").expect("Failed to create evaluator");
    let debug_str = format!("{:?}", evaluator);
    assert!(debug_str.contains("CoreJsonPathEvaluator"));
    assert!(debug_str.contains("$.debug"));
}