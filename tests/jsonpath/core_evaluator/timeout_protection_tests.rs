//! Tests for timeout protection implementation
//! 
//! Extracted from src/jsonpath/core_evaluator/timeout_protection.rs
//! Tests timeout protection for JSONPath evaluation

use serde_json::json;
use quyc_client::jsonpath::core_evaluator::timeout_protection::TimeoutProtectedEvaluator;

#[test]
fn test_timeout_protection() {
    let json = json!({
        "deeply": {
            "nested": {
                "structure": {
                    "with": {
                        "many": {
                            "levels": "value"
                        }
                    }
                }
            }
        }
    });

    // This should complete quickly
    let result = TimeoutProtectedEvaluator::evaluate_with_custom_timeout(
        "$.deeply.nested.structure.with.many.levels",
        &json,
        100,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dangerous_expression_detection() {
    assert!(TimeoutProtectedEvaluator::is_dangerous_expression("$..*"));
    assert!(TimeoutProtectedEvaluator::is_dangerous_expression("$..*..*"));
    assert!(!TimeoutProtectedEvaluator::is_dangerous_expression("$.store.book[*]"));
}

#[test]
fn test_recommended_timeout() {
    assert_eq!(TimeoutProtectedEvaluator::recommended_timeout_ms("$..*"), 5000);
    assert_eq!(TimeoutProtectedEvaluator::recommended_timeout_ms("$..book"), 2000);
    assert_eq!(TimeoutProtectedEvaluator::recommended_timeout_ms("$[?@.price]"), 1500);
    assert_eq!(TimeoutProtectedEvaluator::recommended_timeout_ms("$.store"), 500);
}