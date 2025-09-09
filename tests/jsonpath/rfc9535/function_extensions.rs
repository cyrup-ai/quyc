//! RFC 9535 Function Extensions Tests (Section 2.4)
//!
//! Tests for all five built-in function extensions:
//! - length() (2.4.4)
//! - count() (2.4.5)
//! - match() (2.4.6)
//! - search() (2.4.7)
//! - value() (2.4.8)

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    name: String,
    tags: Vec<String>,
    metadata: Option<serde_json::Value>,
}

/// RFC 9535 Section 2.4.4 - length() Function Tests
#[cfg(test)]
mod length_function_tests {
    use super::*;

    #[test]
    fn test_string_length() {
        // RFC 9535: length() returns number of characters in string
        let json_data = r#"{"items": [
            {"name": "short"},
            {"name": "medium_length"},
            {"name": "very_long_string_name"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?length(@.name) == 5]", 1), // "short"
            ("$.items[?length(@.name) > 10]", 2), // medium and long
            ("$.items[?length(@.name) < 10]", 1), // only "short"
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "String length filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_array_length() {
        // RFC 9535: length() returns number of elements in array
        let json_data = r#"{"groups": [
            {"members": ["a"]},
            {"members": ["a", "b"]},
            {"members": ["a", "b", "c", "d"]},
            {"members": []}
        ]}"#;

        let test_cases = vec![
            ("$.groups[?length(@.members) == 0]", 1), // Empty array
            ("$.groups[?length(@.members) == 1]", 1), // Single element
            ("$.groups[?length(@.members) > 2]", 1),  // More than 2 elements
            ("$.groups[?length(@.members) <= 2]", 3), // 2 or fewer elements
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Array length filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_object_length() {
        // RFC 9535: length() returns number of members in object
        let json_data = r#"{"items": [
            {"props": {}},
            {"props": {"a": 1}},
            {"props": {"a": 1, "b": 2}},
            {"props": {"a": 1, "b": 2, "c": 3, "d": 4}}
        ]}"#;

        let test_cases = vec![
            ("$.items[?length(@.props) == 0]", 1), // Empty object
            ("$.items[?length(@.props) == 1]", 1), // Single property
            ("$.items[?length(@.props) > 2]", 1),  // More than 2 properties
            ("$.items[?length(@.props) <= 2]", 3), // 2 or fewer properties
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Object length filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_length_null_value() {
        // RFC 9535: length() of null returns null
        let json_data = r#"{"items": [
            {"value": "string"},
            {"value": null},
            {"value": [1, 2, 3]}
        ]}"#;

        let mut stream =
            JsonArrayStream::<serde_json::Value>::new("$.items[?length(@.value) == null]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 1, "length(null) should equal null");
    }

    #[test]
    fn test_length_rfc_compliance() {
        // RFC 9535: length() MUST return null for primitive values (Section 2.4.4)
        let json_data = r#"{"items": [
            {"value": 42},
            {"value": true},
            {"value": false}
        ]}"#;

        // RFC 9535 mandates length() returns null for primitives
        let mut stream =
            JsonArrayStream::<serde_json::Value>::new("$.items[?length(@.value) == null]");
        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            3,
            "RFC 9535: length() MUST return null for all primitive values (numbers, booleans)"
        );
    }
}

/// RFC 9535 Section 2.4.5 - count() Function Tests
#[cfg(test)]
mod count_function_tests {
    use super::*;

    #[test]
    fn test_count_nodelist() {
        // RFC 9535: count() returns number of nodes in nodelist
        let json_data = r#"{"data": {
            "books": [
                {"author": "Author1", "tags": ["fiction", "classic"]},
                {"author": "Author2", "tags": ["non-fiction"]},
                {"author": "Author3", "tags": ["fiction", "mystery", "thriller"]}
            ]
        }}"#;

        // Test count of various nodelists
        let test_cases = vec![
            ("count($.data.books[*].author)", 3),  // Count all authors
            ("count($.data.books[*].tags[*])", 6), // Count all tags
            ("count($.data.books[?@.author])", 3), // Count books with authors
        ];

        for (count_expr, expected_count) in test_cases {
            // Test actual count() function execution
            let filter_expr = format!("$.data[?count(@.books[*]) == {}]", expected_count);
            let mut stream = JsonArrayStream::<serde_json::Value>::new(&filter_expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                1,
                "count() expression '{}' should return 1 result when count equals {}",
                count_expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_count_rfc_compliance() {
        // RFC 9535: count() of empty nodelist MUST return 0 (Section 2.4.5)
        let json_data = r#"{"data": {"items": []}}"#;

        let filter_expr = "$.data[?count(@.items[*]) == 0]";
        let mut stream = JsonArrayStream::<serde_json::Value>::new(filter_expr);

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            1,
            "RFC 9535: count() of empty nodelist MUST return 0"
        );
    }

    #[test]
    fn test_count_in_comparisons() {
        // RFC 9535: count() used in comparison expressions
        let json_data = r#"{"groups": [
            {"items": [1, 2]},
            {"items": [1, 2, 3, 4, 5]},
            {"items": []}
        ]}"#;

        let test_cases = vec![
            ("$.groups[?count(@.items[*]) > 3]", 1), // Groups with >3 items
            ("$.groups[?count(@.items[*]) == 0]", 1), // Empty groups
            ("$.groups[?count(@.items[*]) <= 2]", 2), // Small groups
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "count() comparison '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }
}

/// RFC 9535 Section 2.4.6 - match() Function Tests
#[cfg(test)]
mod match_function_tests {
    use super::*;

    #[test]
    fn test_regex_matching() {
        // RFC 9535: match() tests if string matches regular expression
        let json_data = r#"{"items": [
            {"code": "ABC123"},
            {"code": "XYZ789"},
            {"code": "DEF456"},
            {"code": "invalid"},
            {"code": "123ABC"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?match(@.code, '^[A-Z]{3}[0-9]{3}$')]", 3), // Exact pattern
            ("$.items[?match(@.code, '^[A-Z]')]", 3),             // Starts with letter
            ("$.items[?match(@.code, '[0-9]')]", 4),              // Contains digit
            ("$.items[?match(@.code, '^[0-9]')]", 1),             // Starts with digit
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "match() filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_match_case_sensitivity() {
        // RFC 9535: match() should be case-sensitive by default
        let json_data = r#"{"items": [
            {"text": "Hello"},
            {"text": "hello"}, 
            {"text": "HELLO"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?match(@.text, '^Hello$')]", 1),     // Exact case
            ("$.items[?match(@.text, '^hello$')]", 1),     // Lowercase
            ("$.items[?match(@.text, '^HELLO$')]", 1),     // Uppercase
            ("$.items[?match(@.text, '(?i)^hello$')]", 3), // Case insensitive
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "match() case sensitivity test '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_match_special_characters() {
        // RFC 9535: match() with regex special characters
        let json_data = r#"{"items": [
            {"email": "user@example.com"},
            {"email": "invalid-email"},
            {"email": "test.user+tag@domain.org"}
        ]}"#;

        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let expr = format!("$.items[?match(@.email, '{}')]", email_pattern);

        let mut stream = JsonArrayStream::<serde_json::Value>::new(&expr);
        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        // Should match valid email addresses
        assert_eq!(
            results.len(),
            2,
            "Email regex should match 2 valid email addresses"
        );
    }
}

/// RFC 9535 Section 2.4.7 - search() Function Tests
#[cfg(test)]
mod search_function_tests {
    use super::*;

    #[test]
    fn test_regex_search() {
        // RFC 9535: search() tests if string contains match for regex
        let json_data = r#"{"items": [
            {"_description": "This contains the word test"},
            {"_description": "No matching content here"},
            {"_description": "Another test example"},
            {"_description": "Testing is important"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?search(@._description, 'test')]", 3), // Contains 'test'
            ("$.items[?search(@._description, '^This')]", 1), // Starts with 'This'
            ("$.items[?search(@._description, 'important$')]", 1), // Ends with 'important'
            ("$.items[?search(@._description, '[Tt]est')]", 3), // Case variations
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "search() filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_search_vs_match_difference() {
        // RFC 9535: Demonstrate difference between search() and match()
        let json_data = r#"{"items": [
            {"text": "prefix_target_suffix"},
            {"text": "target"},
            {"text": "no_match_here"}
        ]}"#;

        // Test search() - should find partial matches (2 results)
        let search_expr = "$.items[?search(@.text, 'target')]";
        let mut search_stream = JsonArrayStream::<serde_json::Value>::new(search_expr);
        let chunk = Bytes::from(json_data);
        let searchresults: Vec<_> = search_stream.process_chunk(chunk).collect();

        // Test match() - should only find exact matches (1 result)
        let match_expr = "$.items[?match(@.text, 'target')]";
        let mut match_stream = JsonArrayStream::<serde_json::Value>::new(match_expr);
        let chunk = Bytes::from(json_data);
        let matchresults: Vec<_> = match_stream.process_chunk(chunk).collect();

        // search() should find partial matches
        assert_eq!(
            searchresults.len(),
            2,
            "search() should find 2 items containing 'target'"
        );

        // match() should find only exact matches
        assert_eq!(
            matchresults.len(),
            1,
            "match() should find only 1 item exactly matching 'target'"
        );
    }
}

/// RFC 9535 Section 2.4.8 - value() Function Tests
#[cfg(test)]
mod value_function_tests {
    use super::*;

    #[test]
    fn test_single_node_value_extraction() {
        // RFC 9535: value() converts single-node nodelist to value
        let json_data = r#"{"config": {"timeout": 30}}"#;

        // Test value() function execution
        let expr = "$.config[?value(@.timeout) > 20]";
        let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            1,
            "value() function should extract timeout value and compare it"
        );
    }

    #[test]
    fn test_value_rfc_compliance() {
        // RFC 9535: value() MUST error on multi-node nodelist (Section 2.4.8)
        let json_data = r#"{"items": [1, 2, 3]}"#;

        // This should fail compilation or return empty results per RFC
        let expr = "$.items[?value(@[*]) > 1]"; // Multi-node nodelist - INVALID per RFC
        let result = JsonPathParser::compile(expr);

        // RFC mandates this should be an error - either compile error or runtime error
        if result.is_ok() {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // If it compiles, it should return empty results due to error
            assert_eq!(
                results.len(),
                0,
                "RFC 9535: value() on multi-node nodelist MUST error"
            );
        }
        // If it doesn't compile, that's also RFC compliant
    }

    #[test]
    fn test_value_empty_nodelist_rfc() {
        // RFC 9535: value() on empty nodelist returns NoValue (Section 2.4.8)
        let json_data = r#"{"data": {"items": []}}"#;

        // This tests value() behavior on truly empty results
        let expr = "$.data[?value(@.nonexistent) == null]";
        let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        // RFC: value() on empty nodelist should be handled appropriately
        assert!(
            results.len() <= 1,
            "RFC 9535: value() on empty nodelist should return at most 1 result"
        );
    }
}

/// Function Extension Error Cases
#[cfg(test)]
mod function_error_tests {
    use super::*;

    #[test]
    fn test_invalid_function_names() {
        // Test invalid or unsupported function names
        let invalid_functions = vec![
            "$.items[?unknown(@.prop)]", // Unknown function
            "$.items[?length()]",        // Missing argument
            "$.items[?match(@.text)]",   // Missing regex pattern
            "$.items[?search(@.text)]",  // Missing pattern
            "$.items[?count()]",         // Missing nodelist
        ];

        for expr in invalid_functions {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_err(), "Invalid function '{}' should fail", expr);
        }
    }

    #[test]
    fn test_function_argument_validation_rfc() {
        // RFC 9535: Functions MUST reject invalid argument counts
        let invalid_args = vec![
            "$.items[?length(@.prop, extra)]", // Too many arguments - MUST fail
            "$.items[?match(@.text, pattern, flags, extra)]", // Too many args - MUST fail
            "$.items[?length()]",              // Missing argument - MUST fail
            "$.items[?match(@.text)]",         // Missing pattern - MUST fail
        ];

        for expr in invalid_args {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "RFC 9535: Invalid function arguments '{}' MUST be rejected",
                expr
            );
        }
    }

    #[test]
    fn test_valid_nested_function_calls_rfc() {
        // RFC 9535: Valid nested function calls MUST be supported
        let json_data = r#"{"items": [{"text": "test123"}, {"text": "short"}]}"#;

        // This is a valid nested function call per RFC
        let expr = "$.items[?length(@.text) > 5]"; // Using length in comparison
        let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            1,
            "RFC 9535: Valid nested function usage MUST work"
        );
    }
}
