//! Comprehensive test suite for descendant operations
//!
//! Contains all test cases for descendant traversal, collection, filtering,
//! and analysis operations with RFC 9535 compliance verification.

use serde_json::json;
use quyc_client::jsonpath::core_evaluator::evaluator::descendant_operations::DescendantOperations;

#[test]
fn test_collect_all_descendants_owned() {
    let json = json!({
        "a": {
            "b": "value1",
            "c": ["value2", "value3"]
        }
    });

    let mut results = Vec::new();
    DescendantOperations::collect_all_descendants_owned(&json, &mut results);

    // Should collect: {"b": "value1", "c": ["value2", "value3"]}, "value1", ["value2", "value3"], "value2", "value3"
    assert_eq!(results.len(), 5);
    assert!(results.contains(&json!({"b": "value1", "c": ["value2", "value3"]})));
    assert!(results.contains(&json!("value1")));
    assert!(results.contains(&json!(["value2", "value3"])));
    assert!(results.contains(&json!("value2")));
    assert!(results.contains(&json!("value3")));
}

#[test]
fn test_collect_descendants_at_depth() {
    let json = json!({
        "level1": {
            "level2": {
                "level3": "target"
            }
        }
    });

    let mut results = Vec::new();
    DescendantOperations::collect_descendants_at_depth(&json, 2, 0, &mut results);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json!({"level3": "target"}));
}#[test]
fn test_count_descendants() {
    let json = json!({
        "a": "value1",
        "b": {
            "c": "value2",
            "d": ["value3", "value4"]
        }
    });

    let count = DescendantOperations::count_descendants(&json);
    // Should count: "value1", {"c": "value2", "d": ["value3", "value4"]}, "value2", ["value3", "value4"], "value3", "value4"
    assert_eq!(count, 6);
}

#[test]
fn test_max_descendant_depth() {
    let json = json!({
        "shallow": "value",
        "deep": {
            "level2": {
                "level3": {
                    "level4": "deepest"
                }
            }
        }
    });

    let depth = DescendantOperations::max_descendant_depth(&json);
    assert_eq!(depth, 4); // shallow=1, deep.level2.level3.level4=4
}

#[test]
fn test_collect_descendants_with_paths() {
    let json = json!({
        "a": {
            "b": "value1"
        },
        "c": ["value2"]
    });

    let mut results = Vec::new();
    DescendantOperations::collect_descendants_with_paths(&json, String::new(), &mut results);

    assert_eq!(results.len(), 4);
    assert!(results.iter().any(|(path, _)| path == "a"));
    assert!(results.iter().any(|(path, _)| path == "a.b"));
    assert!(results.iter().any(|(path, _)| path == "c"));
    assert!(results.iter().any(|(path, _)| path == "c[0]"));
}#[test]
fn test_filter_descendants() {
    let json = json!({
        "numbers": [1, 2, 3],
        "strings": ["a", "b"],
        "nested": {
            "more_numbers": [4, 5]
        }
    });

    let mut results = Vec::new();
    DescendantOperations::filter_descendants(&json, |v| v.is_number(), &mut results);

    assert_eq!(results.len(), 5); // 1, 2, 3, 4, 5
    assert!(results.contains(&json!(1)));
    assert!(results.contains(&json!(2)));
    assert!(results.contains(&json!(3)));
    assert!(results.contains(&json!(4)));
    assert!(results.contains(&json!(5)));
}

#[test]
fn test_collect_leaf_values() {
    let json = json!({
        "a": "leaf1",
        "b": {
            "c": "leaf2",
            "d": []
        },
        "e": {}
    });

    let mut results = Vec::new();
    DescendantOperations::collect_leaf_values(&json, &mut results);

    assert_eq!(results.len(), 3);
    assert!(results.contains(&json!("leaf1")));
    assert!(results.contains(&json!("leaf2")));
    assert!(results.contains(&json!([])));
}#[test]
fn test_has_descendants() {
    assert!(DescendantOperations::has_descendants(
        &json!({"a": "value"})
    ));
    assert!(DescendantOperations::has_descendants(&json!(["value"])));
    assert!(!DescendantOperations::has_descendants(&json!({})));
    assert!(!DescendantOperations::has_descendants(&json!([])));
    assert!(!DescendantOperations::has_descendants(&json!("primitive")));
    assert!(!DescendantOperations::has_descendants(&json!(42)));
    assert!(!DescendantOperations::has_descendants(&json!(null)));
}

#[test]
fn test_empty_structures() {
    let json = json!({});
    let mut results = Vec::new();
    DescendantOperations::collect_all_descendants_owned(&json, &mut results);
    assert!(results.is_empty());

    let json = json!([]);
    let mut results = Vec::new();
    DescendantOperations::collect_all_descendants_owned(&json, &mut results);
    assert!(results.is_empty());
}

#[test]
fn test_primitive_values() {
    let json = json!("primitive");
    let mut results = Vec::new();
    DescendantOperations::collect_all_descendants_owned(&json, &mut results);
    assert!(results.is_empty());

    assert_eq!(DescendantOperations::count_descendants(&json), 0);
    assert_eq!(DescendantOperations::max_descendant_depth(&json), 0);
}