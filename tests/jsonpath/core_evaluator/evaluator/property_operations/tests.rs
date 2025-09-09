//! Tests for property operations
//!
//! Comprehensive test coverage for all property operation functionality.

use serde_json::json;
use quyc_client::jsonpath::core_evaluator::evaluator::core_types::CoreJsonPathEvaluator;
use quyc_client::jsonpath::core_evaluator::evaluator::property_operations::core::PropertyOperations;

#[test]
fn test_evaluate_property_path_simple() {
    let json = json!({"a": {"b": {"c": "value"}}});
    let results = PropertyOperations::evaluate_property_path(&json, "a.b.c").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json!("value"));
}

#[test]
fn test_evaluate_property_path_missing() {
    let json = json!({"a": {"b": "value"}});
    let results = PropertyOperations::evaluate_property_path(&json, "a.missing").unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_evaluate_property_path_empty_segment() {
    let json = json!({"a": "value"});
    let results = PropertyOperations::evaluate_property_path(&json, "a..b").unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_find_property_recursive() {
    let json = json!({
        "store": {
            "book": [
                {"title": "Book 1", "author": {"name": "Author 1"}},
                {"title": "Book 2", "author": {"name": "Author 2"}}
            ]
        },
        "title": "Store Title"
    });

    let results = PropertyOperations::find_property_recursive(&json, "title");
    assert_eq!(results.len(), 3);
    assert!(results.contains(&json!("Store Title")));
    assert!(results.contains(&json!("Book 1")));
    assert!(results.contains(&json!("Book 2")));
}

#[test]
fn test_find_property_recursive_nested() {
    let json = json!({
        "level1": {
            "level2": {
                "target": "found"
            }
        }
    });

    let results = PropertyOperations::find_property_recursive(&json, "target");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json!("found"));
}

#[test]
fn test_find_properties_matching_wildcard() {
    let json = json!({
        "name1": "value1",
        "name2": "value2",
        "other": "value3"
    });

    let results = PropertyOperations::find_properties_matching(&json, "name*");
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|(path, _)| path == "name1"));
    assert!(results.iter().any(|(path, _)| path == "name2"));
}

#[test]
fn test_find_properties_matching_exact() {
    let json = json!({
        "exact": "value1",
        "other": "value2"
    });

    let results = PropertyOperations::find_properties_matching(&json, "exact");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "exact");
    assert_eq!(results[0].1, json!("value1"));
}

#[test]
fn test_matches_pattern() {
    assert!(PropertyOperations::matches_pattern("test", "*"));
    assert!(PropertyOperations::matches_pattern("test", "test"));
    assert!(PropertyOperations::matches_pattern("test123", "test*"));
    assert!(PropertyOperations::matches_pattern("123test", "*test"));
    assert!(!PropertyOperations::matches_pattern("test", "other"));
}

#[test]
fn test_get_property_names() {
    let json = json!({"a": 1, "b": 2, "c": 3});
    let names = PropertyOperations::get_property_names(&json);
    assert_eq!(names.len(), 3);
    assert!(names.contains(&"a".to_string()));
    assert!(names.contains(&"b".to_string()));
    assert!(names.contains(&"c".to_string()));
}

#[test]
fn test_get_property_names_non_object() {
    let json = json!([1, 2, 3]);
    let names = PropertyOperations::get_property_names(&json);
    assert!(names.is_empty());
}

#[test]
fn test_has_property_recursive() {
    let json = json!({
        "level1": {
            "level2": {
                "target": "value"
            }
        }
    });

    assert!(PropertyOperations::has_property_recursive(&json, "target"));
    assert!(PropertyOperations::has_property_recursive(&json, "level1"));
    assert!(PropertyOperations::has_property_recursive(&json, "level2"));
    assert!(!PropertyOperations::has_property_recursive(
        &json, "missing"
    ));
}

#[test]
fn test_count_property_occurrences() {
    let json = json!({
        "name": "value1",
        "nested": {
            "name": "value2",
            "deep": {
                "name": "value3"
            }
        }
    });

    let count = PropertyOperations::count_property_occurrences(&json, "name");
    assert_eq!(count, 3);
}

#[test]
fn test_get_property_or_default() {
    let json = json!({"existing": "value"});

    let existing =
        PropertyOperations::get_property_or_default(&json, "existing", json!("default"));
    assert_eq!(existing, json!("value"));

    let missing =
        PropertyOperations::get_property_or_default(&json, "missing", json!("default"));
    assert_eq!(missing, json!("default"));
}

#[test]
fn test_evaluator_property_operations() {
    let evaluator = CoreJsonPathEvaluator::new("$.test").unwrap();
    let json = json!({"a": {"b": "value"}});

    let results = evaluator.evaluate_property_path(&json, "a.b").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json!("value"));

    let recursive_results = evaluator.find_property_recursive(&json, "b");
    assert_eq!(recursive_results.len(), 1);
    assert_eq!(recursive_results[0], json!("value"));
}