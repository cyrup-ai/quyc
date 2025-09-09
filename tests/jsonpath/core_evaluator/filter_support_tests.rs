//! Tests for filter support implementation
//! 
//! Extracted from src/jsonpath/core_evaluator/filter_support.rs
//! Tests filter support utilities for JSONPath evaluation

use serde_json::json;
use quyc_client::jsonpath::core_evaluator::filter_support::FilterSupport;

#[test]
fn test_collect_existing_properties() {
    let arr = vec![
        json!({"name": "Alice", "age": 30}),
        json!({"name": "Bob", "city": "NYC"}),
        json!({"age": 25, "country": "USA"}),
    ];

    let properties = FilterSupport::collect_existing_properties(&arr);
    assert!(properties.contains("name"));
    assert!(properties.contains("age"));
    assert!(properties.contains("city"));
    assert!(properties.contains("country"));
    assert_eq!(properties.len(), 4);
}

#[test]
fn test_has_property() {
    let obj = json!({"name": "Alice", "age": 30});
    assert!(FilterSupport::has_property(&obj, "name"));
    assert!(FilterSupport::has_property(&obj, "age"));
    assert!(!FilterSupport::has_property(&obj, "city"));
}

#[test]
fn test_get_property() {
    let obj = json!({"name": "Alice", "age": 30});
    assert_eq!(FilterSupport::get_property(&obj, "name"), Some(&json!("Alice")));
    assert_eq!(FilterSupport::get_property(&obj, "age"), Some(&json!(30)));
    assert_eq!(FilterSupport::get_property(&obj, "city"), None);
}

#[test]
fn test_matches_type() {
    assert!(FilterSupport::matches_type(&json!(null), "null"));
    assert!(FilterSupport::matches_type(&json!(true), "boolean"));
    assert!(FilterSupport::matches_type(&json!(42), "number"));
    assert!(FilterSupport::matches_type(&json!("hello"), "string"));
    assert!(FilterSupport::matches_type(&json!([1, 2, 3]), "array"));
    assert!(FilterSupport::matches_type(&json!({"a": 1}), "object"));
}

#[test]
fn test_is_truthy() {
    assert!(!FilterSupport::is_truthy(&json!(null)));
    assert!(!FilterSupport::is_truthy(&json!(false)));
    assert!(!FilterSupport::is_truthy(&json!(0)));
    assert!(!FilterSupport::is_truthy(&json!("")));
    assert!(!FilterSupport::is_truthy(&json!([])));
    assert!(!FilterSupport::is_truthy(&json!({})));

    assert!(FilterSupport::is_truthy(&json!(true)));
    assert!(FilterSupport::is_truthy(&json!(1)));
    assert!(FilterSupport::is_truthy(&json!("hello")));
    assert!(FilterSupport::is_truthy(&json!([1])));
    assert!(FilterSupport::is_truthy(&json!({"a": 1})));
}

#[test]
fn test_compare_values() {
    use std::cmp::Ordering;

    assert_eq!(FilterSupport::compare_values(&json!(1), &json!(2)), Some(Ordering::Less));
    assert_eq!(FilterSupport::compare_values(&json!(2), &json!(1)), Some(Ordering::Greater));
    assert_eq!(FilterSupport::compare_values(&json!(1), &json!(1)), Some(Ordering::Equal));

    assert_eq!(FilterSupport::compare_values(&json!("a"), &json!("b")), Some(Ordering::Less));
    assert_eq!(FilterSupport::compare_values(&json!(true), &json!(false)), Some(Ordering::Greater));

    // Different types cannot be compared
    assert_eq!(FilterSupport::compare_values(&json!(1), &json!("1")), None);
}

#[test]
fn test_contains_value() {
    assert!(FilterSupport::contains_value(&json!([1, 2, 3]), &json!(2)));
    assert!(!FilterSupport::contains_value(&json!([1, 2, 3]), &json!(4)));

    assert!(FilterSupport::contains_value(&json!({"a": 1, "b": 2}), &json!(1)));
    assert!(!FilterSupport::contains_value(&json!({"a": 1, "b": 2}), &json!(3)));

    assert!(FilterSupport::contains_value(&json!("hello world"), &json!("world")));
    assert!(!FilterSupport::contains_value(&json!("hello world"), &json!("foo")));
}