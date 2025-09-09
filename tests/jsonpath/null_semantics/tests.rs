//! Tests for RFC 9535 Null vs Missing Value Semantics
//!
//! Comprehensive test suite to verify correct implementation of null vs missing
//! distinction throughout JSONPath evaluation.

use quyc_client::jsonpath::null_semantics::{NullSemantics, PropertyAccessResult};

#[test]
fn test_null_vs_missing_distinction() {
    // Test null value present
    let json_with_null = serde_json::json!({"a": null});
    let null_result = NullSemantics::access_property(&json_with_null, "a");
    assert!(matches!(null_result, PropertyAccessResult::NullValue));
    assert!(null_result.is_present());
    assert!(!null_result.is_missing());
    assert!(null_result.is_null());

    // Test property missing
    let json_empty = serde_json::json!({});
    let missing_result = NullSemantics::access_property(&json_empty, "a");
    assert!(matches!(missing_result, PropertyAccessResult::Missing));
    assert!(!missing_result.is_present());
    assert!(missing_result.is_missing());
    assert!(!missing_result.is_null());

    // Test non-null value
    let json_with_value = serde_json::json!({"a": "hello"});
    let value_result = NullSemantics::access_property(&json_with_value, "a");
    assert!(matches!(value_result, PropertyAccessResult::Value(_)));
    assert!(value_result.is_present());
    assert!(!value_result.is_missing());
    assert!(!value_result.is_null());
}

#[test]
fn test_property_path_access() {
    let json = serde_json::json!({
        "store": {
            "book": null,
            "bicycle": {
                "color": "red"
            }
        }
    });

    // Access null value through path
    let null_path_result =
        NullSemantics::access_property_path(&json, &["store".to_string(), "book".to_string()]);
    assert!(matches!(null_path_result, PropertyAccessResult::NullValue));

    // Access missing property through path
    let missing_path_result = NullSemantics::access_property_path(
        &json,
        &["store".to_string(), "missing".to_string()],
    );
    assert!(matches!(missing_path_result, PropertyAccessResult::Missing));

    // Access existing value through path
    let value_path_result = NullSemantics::access_property_path(
        &json,
        &[
            "store".to_string(),
            "bicycle".to_string(),
            "color".to_string(),
        ],
    );
    assert!(matches!(value_path_result, PropertyAccessResult::Value(_)));
}

#[test]
fn test_array_access() {
    let json = serde_json::json!([null, "value", 42]);

    // Access null element
    let null_element = NullSemantics::access_array_index(&json, 0);
    assert!(matches!(null_element, PropertyAccessResult::NullValue));

    // Access regular element
    let value_element = NullSemantics::access_array_index(&json, 1);
    assert!(matches!(value_element, PropertyAccessResult::Value(_)));

    // Access out of bounds (missing)
    let missing_element = NullSemantics::access_array_index(&json, 10);
    assert!(matches!(missing_element, PropertyAccessResult::Missing));

    // Negative index access
    let last_element = NullSemantics::access_array_index(&json, -1);
    assert!(matches!(last_element, PropertyAccessResult::Value(_)));
}

#[test]
fn test_comparison_semantics() {
    let null_result = PropertyAccessResult::NullValue;
    let missing_result = PropertyAccessResult::Missing;
    let value_result = PropertyAccessResult::Value(serde_json::json!("test"));

    // Null vs null
    assert!(
        NullSemantics::compare_with_null_semantics(&null_result, &null_result)
            .expect("Failed to compare null vs null")
    );

    // Missing vs missing
    assert!(
        NullSemantics::compare_with_null_semantics(&missing_result, &missing_result)
            .expect("Failed to compare missing vs missing")
    );

    // Null vs missing (different)
    assert!(
        !NullSemantics::compare_with_null_semantics(&null_result, &missing_result)
            .expect("Failed to compare null vs missing")
    );

    // Value vs missing (different)
    assert!(
        !NullSemantics::compare_with_null_semantics(&value_result, &missing_result)
            .expect("Failed to compare value vs missing")
    );
}

#[test]
fn test_implementation_validation() {
    // This should pass without errors
    NullSemantics::validate_implementation()
        .expect("Failed to validate null semantics implementation");
}