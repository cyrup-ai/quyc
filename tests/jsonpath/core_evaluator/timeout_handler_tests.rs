//! Tests for timeout handler implementation
//! 
//! Extracted from src/jsonpath/core_evaluator/evaluator/timeout_handler.rs
//! Tests timeout protection for JSONPath evaluation

use std::time::Duration;
use serde_json::json;
use quyc_client::jsonpath::core_evaluator::evaluator::core_types::CoreJsonPathEvaluator;
use quyc_client::jsonpath::core_evaluator::evaluator::timeout_handler::{TimeoutHandler, TimeoutConfig};

#[test]
fn test_timeout_config_default() {
    let config = TimeoutConfig::default();
    assert_eq!(config.timeout_duration, Duration::from_millis(1500));
    assert_eq!(config.log_timeouts, true);
}

#[test]
fn test_timeout_config_custom() {
    let config = TimeoutConfig {
        timeout_duration: Duration::from_millis(500),
        log_timeouts: false,
    };
    assert_eq!(config.timeout_duration, Duration::from_millis(500));
    assert_eq!(config.log_timeouts, false);
}

#[test]
fn test_is_potentially_slow() {
    assert!(TimeoutHandler::is_potentially_slow("$..book"));
    assert!(TimeoutHandler::is_potentially_slow("$.store.*"));
    assert!(TimeoutHandler::is_potentially_slow("$.store.book[?(@.price < 10)]"));
    assert!(TimeoutHandler::is_potentially_slow("$.store.book[1:3]"));
    assert!(TimeoutHandler::is_potentially_slow("$.a[0].b[1].c[2].d[3]"));

    assert!(!TimeoutHandler::is_potentially_slow("$.store.book"));
    assert!(!TimeoutHandler::is_potentially_slow("$.simple"));
}

#[test]
fn test_estimate_complexity() {
    assert_eq!(TimeoutHandler::estimate_complexity("$.simple"), 1);
    assert_eq!(TimeoutHandler::estimate_complexity("$..book"), 51); // 1 + 50
    assert_eq!(TimeoutHandler::estimate_complexity("$.store.*"), 11); // 1 + 10
    assert_eq!(TimeoutHandler::estimate_complexity("$.book[?(@.price)]"), 23); // 1 + 20 + 2
    assert_eq!(TimeoutHandler::estimate_complexity("$.book[1:3]"), 8); // 1 + 5 + 2
}

#[test]
fn test_recommended_timeout() {
    assert_eq!(TimeoutHandler::recommended_timeout("$.simple"), Duration::from_millis(100));
    assert_eq!(TimeoutHandler::recommended_timeout("$.store.*"), Duration::from_millis(500));
    assert_eq!(TimeoutHandler::recommended_timeout("$..book"), Duration::from_millis(1000));
    assert_eq!(TimeoutHandler::recommended_timeout("$..*[?(@.price > 10)]"), Duration::from_millis(2000));
}

#[test]
fn test_timeout_handler_simple_evaluation() {
    let evaluator = CoreJsonPathEvaluator::new("$.test").expect("Failed to create evaluator");
    let json = json!({"test": "value"});

    let result = TimeoutHandler::evaluate_with_timeout(&evaluator, &json, None);
    assert!(result.is_ok());
}

#[test]
fn test_timeout_handler_with_custom_config() {
    let evaluator = CoreJsonPathEvaluator::new("$.test").expect("Failed to create evaluator");
    let json = json!({"test": "value"});

    let config = TimeoutConfig {
        timeout_duration: Duration::from_millis(100),
        log_timeouts: false,
    };

    let result = TimeoutHandler::evaluate_with_timeout(&evaluator, &json, Some(config));
    assert!(result.is_ok());
}

#[test]
fn test_timeout_config_clone() {
    let config = TimeoutConfig::default();
    let cloned = config.clone();
    assert_eq!(config.timeout_duration, cloned.timeout_duration);
    assert_eq!(config.log_timeouts, cloned.log_timeouts);
}

#[test]
fn test_timeout_config_debug() {
    let config = TimeoutConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("TimeoutConfig"));
    assert!(debug_str.contains("1500"));
}