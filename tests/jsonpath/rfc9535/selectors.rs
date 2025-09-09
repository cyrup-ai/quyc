//! RFC 9535 Selector Compliance Tests
//!
//! Tests for all five selector types defined in RFC 9535 Section 2.3:
//! - Name Selector (2.3.1)
//! - Wildcard Selector (2.3.2)
//! - Index Selector (2.3.3)
//! - Array Slice Selector (2.3.4)
//! - Filter Selector (2.3.5)

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct BookModel {
    category: String,
    author: String,
    title: String,
    price: f64,
    isbn: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct StoreModel {
    color: String,
    price: f64,
}

/// RFC 9535 Section 2.3.1 - Name Selector Tests
#[cfg(test)]
mod name_selector_tests {
    use super::*;

    #[test]
    fn test_single_quoted_name_selector() {
        // RFC 9535: name-selector with single quotes
        let result = JsonPathParser::compile("$['store']");
        assert!(
            result.is_ok(),
            "Single quoted name selector should be valid"
        );

        let json_data = r#"{"store": {"book": [{"category": "fiction"}]}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$['store']");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(results.len(), 1, "Should select store object");
    }

    #[test]
    fn test_double_quoted_name_selector() {
        // RFC 9535: name-selector with double quotes
        let result = JsonPathParser::compile("$[\"store\"]");
        assert!(
            result.is_ok(),
            "Double quoted name selector should be valid"
        );

        let json_data = r#"{"store": {"book": [{"category": "fiction"}]}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$[\"store\"]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(results.len(), 1, "Should select store object");
    }

    #[test]
    fn test_dot_notation_name_selector() {
        // RFC 9535: Dot notation for name selectors
        let result = JsonPathParser::compile("$.store");
        assert!(result.is_ok(), "Dot notation should be valid");

        let json_data = r#"{"store": {"book": [{"category": "fiction"}]}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(results.len(), 1, "Should select store object");
    }

    #[test]
    fn test_escaped_characters_in_names() {
        // RFC 9535: Escaped characters in string literals
        let expressions = vec![
            r#"$['store\'s']"#,    // Escaped single quote
            r#"$["store\"s"]"#,    // Escaped double quote
            r#"$['store\\book']"#, // Escaped backslash
            r#"$['store\nbook']"#, // Escaped newline
            r#"$['store\tbook']"#, // Escaped tab
        ];

        for expr in expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Escaped characters '{}' should be valid",
                expr
            );
        }
    }

    #[test]
    fn test_unicode_property_names() {
        // RFC 9535: Unicode support in name selectors
        let expressions = vec![
            "$.Ñoño",        // Spanish characters
            "$['中文']",     // Chinese characters
            "$['🚀rocket']", // Emoji
            "$['ßeta']",     // German character
        ];

        for expr in expressions {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_ok(), "Unicode name '{}' should be valid", expr);
        }
    }

    #[test]
    fn test_nonexistent_property_selection() {
        // RFC 9535: Name selector should select at most one member
        let json_data = r#"{"store": {"book": []}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.nonexistent");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            0,
            "Nonexistent property should return empty nodelist"
        );
    }
}

/// RFC 9535 Section 2.3.2 - Wildcard Selector Tests  
#[cfg(test)]
mod wildcard_selector_tests {
    use super::*;

    #[test]
    fn test_object_wildcard_selection() {
        // RFC 9535: Wildcard selects all children of object
        let json_data = r#"{"store": {"book": [], "bicycle": {"color": "red"}}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store[*]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            2,
            "Wildcard should select all object members"
        );
    }

    #[test]
    fn test_array_wildcard_selection() {
        // RFC 9535: Wildcard selects all elements of array
        let json_data = r#"{"books": [{"id": 1}, {"id": 2}, {"id": 3}]}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.books[*]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            3,
            "Wildcard should select all array elements"
        );
    }

    #[test]
    fn test_root_wildcard_selection() {
        // RFC 9535: Root wildcard should select all top-level children
        let json_data = r#"{"a": 1, "b": 2, "c": 3}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$[*]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            3,
            "Root wildcard should select all top-level members"
        );
    }

    #[test]
    fn test_wildcard_on_primitive_values() {
        // RFC 9535: Wildcard on primitive values should return empty nodelist
        let json_data = r#"{"value": 42}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.value[*]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            0,
            "Wildcard on primitive should return empty nodelist"
        );
    }

    #[test]
    fn test_dot_star_notation() {
        // RFC 9535: .* notation for wildcard
        let result = JsonPathParser::compile("$.store.*");
        assert!(result.is_ok(), "Dot-star notation should be valid");

        let json_data = r#"{"store": {"book": [], "bicycle": {}}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store.*");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            2,
            "Dot-star should select all store children"
        );
    }
}

/// RFC 9535 Section 2.3.3 - Index Selector Tests
#[cfg(test)]
mod index_selector_tests {
    use super::*;

    #[test]
    fn test_positive_index_selection() {
        // RFC 9535: Non-negative indices select from start (0-based)
        let json_data = r#"{"books": ["book0", "book1", "book2"]}"#;

        let test_cases = vec![
            ("$.books[0]", 0, "book0"),
            ("$.books[1]", 1, "book1"),
            ("$.books[2]", 2, "book2"),
        ];

        for (expr, expected_index, expected_value) in test_cases {
            let mut stream = JsonArrayStream::<String>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                1,
                "Index [{}] should select one element",
                expected_index
            );
            {
                let value = &results[0];
                assert_eq!(value, expected_value, "Should select correct value");
            }
        }
    }

    #[test]
    fn test_negative_index_selection() {
        // RFC 9535: Negative indices select from end (-1 is last element)
        let json_data = r#"{"books": ["book0", "book1", "book2"]}"#;

        let test_cases = vec![
            ("$.books[-1]", "book2"), // Last element
            ("$.books[-2]", "book1"), // Second to last
            ("$.books[-3]", "book0"), // Third to last (first)
        ];

        for (expr, expected_value) in test_cases {
            let mut stream = JsonArrayStream::<String>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                1,
                "Negative index '{}' should select one element",
                expr
            );
            {
                let value = &results[0];
                assert_eq!(
                    value, expected_value,
                    "Should select correct value from end"
                );
            }
        }
    }

    #[test]
    fn test_out_of_bounds_index() {
        // RFC 9535: Out of bounds index should return empty nodelist
        let json_data = r#"{"books": ["book0", "book1"]}"#;

        let out_of_bounds_cases = vec![
            "$.books[5]",  // Positive out of bounds
            "$.books[-5]", // Negative out of bounds
        ];

        for expr in out_of_bounds_cases {
            let mut stream = JsonArrayStream::<String>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                0,
                "Out of bounds '{}' should return empty nodelist",
                expr
            );
        }
    }

    #[test]
    fn test_index_on_non_array() {
        // RFC 9535: Index selector on non-array should return empty nodelist
        let json_data = r#"{"store": {"name": "bookstore"}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store[0]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            0,
            "Index on object should return empty nodelist"
        );
    }

    #[test]
    fn test_zero_index_validation() {
        // RFC 9535: Index 0 should be valid first element
        let result = JsonPathParser::compile("$.books[0]");
        assert!(result.is_ok(), "Index 0 should be valid");

        let json_data = r#"{"books": ["first"]}"#;
        let mut stream = JsonArrayStream::<String>::new("$.books[0]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 1, "Index 0 should select first element");
        assert_eq!(results[0], "first", "Should select first element");
    }
}
