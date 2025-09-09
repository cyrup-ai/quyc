//! Tests for evaluation engine implementation
//! 
//! Extracted from src/jsonpath/core_evaluator/evaluator/evaluation_engine.rs
//! Tests core evaluation logic including recursive descent processing

use serde_json::json;
use quyc_client::jsonpath::core_evaluator::evaluator::core_types::CoreJsonPathEvaluator;
use quyc_client::jsonpath::core_evaluator::evaluator::evaluation_engine::EvaluationEngine;
use quyc_client::jsonpath::parser::JsonSelector;

#[test]
fn test_simple_evaluation() {
    let evaluator = CoreJsonPathEvaluator::new("$.test").expect("Failed to create evaluator");
    let json = json!({"test": "value"});

    let results = EvaluationEngine::evaluate_expression(&evaluator, &json).expect("Failed to evaluate expression");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json!("value"));
}

#[test]
fn test_nested_evaluation() {
    let evaluator = CoreJsonPathEvaluator::new("$.store.book").expect("Failed to create evaluator");
    let json = json!({
        "store": {
            "book": [
                {"title": "Book 1"},
                {"title": "Book 2"}
            ]
        }
    });

    let results = EvaluationEngine::evaluate_expression(&evaluator, &json).expect("Failed to evaluate expression");
    assert_eq!(results.len(), 1);
    assert!(results[0].is_array());
}

#[test]
fn test_wildcard_evaluation() {
    let evaluator = CoreJsonPathEvaluator::new("$.store.*").expect("Failed to create evaluator");
    let json = json!({
        "store": {
            "book": [],
            "bicycle": {}
        }
    });

    let results = EvaluationEngine::evaluate_expression(&evaluator, &json).expect("Failed to evaluate expression");
    assert_eq!(results.len(), 2);
}

#[test]
fn test_recursive_descent() {
    let evaluator = CoreJsonPathEvaluator::new("$..title").expect("Failed to create evaluator");
    let json = json!({
        "store": {
            "book": [
                {"title": "Book 1"},
                {"title": "Book 2"}
            ]
        }
    });

    let results = EvaluationEngine::evaluate_expression(&evaluator, &json).expect("Failed to evaluate expression");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], json!("Book 1"));
    assert_eq!(results[1], json!("Book 2"));
}

#[test]
fn test_empty_results() {
    let evaluator = CoreJsonPathEvaluator::new("$.nonexistent").expect("Failed to create evaluator");
    let json = json!({"test": "value"});

    let results = EvaluationEngine::evaluate_expression(&evaluator, &json).expect("Failed to evaluate expression");
    assert!(results.is_empty());
}

#[test]
fn test_evaluate_multiple() {
    let expressions = vec!["$.a", "$.b", "$.c"];
    let json = json!({"a": 1, "b": 2, "c": 3});

    let results = EvaluationEngine::evaluate_multiple(&expressions, &json).expect("Failed to evaluate multiple expressions");
    assert_eq!(results.len(), 3);
    assert_eq!(results[0], vec![json!(1)]);
    assert_eq!(results[1], vec![json!(2)]);
    assert_eq!(results[2], vec![json!(3)]);
}

#[test]
fn test_is_expensive_evaluation() {
    let simple_selectors = vec![JsonSelector::Root];
    assert!(!EvaluationEngine::is_expensive_evaluation(&simple_selectors));

    let expensive_selectors = vec![JsonSelector::RecursiveDescent];
    assert!(EvaluationEngine::is_expensive_evaluation(&expensive_selectors));
}

#[test]
fn test_estimate_complexity() {
    let simple_selectors = vec![JsonSelector::Root];
    let complexity = EvaluationEngine::estimate_evaluation_complexity(&simple_selectors);
    assert_eq!(complexity, 1);

    let complex_selectors = vec![JsonSelector::RecursiveDescent, JsonSelector::Wildcard];
    let complexity = EvaluationEngine::estimate_evaluation_complexity(&complex_selectors);
    assert_eq!(complexity, 60); // 50 + 10
}

#[test]
fn test_invalid_expression() {
    let result = CoreJsonPathEvaluator::new("$.[invalid");
    assert!(result.is_err());
}

#[test]
fn test_complex_path() {
    let evaluator = CoreJsonPathEvaluator::new("$.store.book[0].title").expect("Failed to create evaluator");
    let json = json!({
        "store": {
            "book": [
                {"title": "First Book"},
                {"title": "Second Book"}
            ]
        }
    });

    let results = EvaluationEngine::evaluate_expression(&evaluator, &json).expect("Failed to evaluate expression");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json!("First Book"));
}