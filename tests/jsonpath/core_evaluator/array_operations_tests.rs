//! Tests for array operations
//!
//! Comprehensive test coverage for JSONPath array indexing, slicing, and operations.

use serde_json::json;
use quyc_client::jsonpath::core_evaluator::array_operations::ArrayOperations;

#[test]
fn test_positive_index() {
    let arr = vec![json!(1), json!(2), json!(3)];
    let results =
        ArrayOperations::apply_index(&arr, 1, false).expect("Failed to apply positive index");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json!(2));
}

#[test]
fn test_negative_index() {
    let arr = vec![json!(1), json!(2), json!(3)];
    let results =
        ArrayOperations::apply_index(&arr, -1, true).expect("Failed to apply negative index");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json!(3));
}

#[test]
fn test_slice_operation() {
    let arr = vec![json!(1), json!(2), json!(3), json!(4), json!(5)];
    let results = ArrayOperations::apply_slice(&arr, Some(1), Some(4), 1)
        .expect("Failed to apply slice operation");
    assert_eq!(results.len(), 3);
    assert_eq!(results, vec![json!(2), json!(3), json!(4)]);
}

#[test]
fn test_slice_with_step() {
    let arr = vec![json!(1), json!(2), json!(3), json!(4), json!(5)];
    let results = ArrayOperations::apply_slice(&arr, Some(0), Some(5), 2)
        .expect("Failed to apply slice with step");
    assert_eq!(results.len(), 3);
    assert_eq!(results, vec![json!(1), json!(3), json!(5)]);
}

#[test]
fn test_reverse_slice() {
    let arr = vec![json!(1), json!(2), json!(3), json!(4), json!(5)];
    let results = ArrayOperations::apply_slice(&arr, Some(4), Some(0), -1)
        .expect("Failed to apply reverse slice");
    assert_eq!(results.len(), 4);
    assert_eq!(results, vec![json!(5), json!(4), json!(3), json!(2)]);
}

#[test]
fn test_index_validation() {
    let arr = vec![json!(1), json!(2), json!(3)];
    assert!(ArrayOperations::is_valid_index(&arr, 0, false));
    assert!(ArrayOperations::is_valid_index(&arr, 2, false));
    assert!(!ArrayOperations::is_valid_index(&arr, 3, false));
    assert!(ArrayOperations::is_valid_index(&arr, -1, true));
    assert!(!ArrayOperations::is_valid_index(&arr, -4, true));
}