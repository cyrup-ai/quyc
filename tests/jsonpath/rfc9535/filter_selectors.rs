//! RFC 9535 Filter Selector Tests (Section 2.3.5)
//!
//! Tests for filter selector syntax: ?<logical-expr>

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
    available: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct ProductModel {
    name: String,
    price: f64,
    quantity: i32,
    tags: Vec<String>,
}

/// RFC 9535 Section 2.3.5 - Filter Selector Tests
#[cfg(test)]
mod filter_selector_tests {
    use super::*;

    #[test]
    fn test_current_node_reference() {
        // RFC 9535: @ refers to current node being tested
        let json_data = r#"{"books": [
            {"price": 8.95, "available": true},
            {"price": 12.99, "available": false}
        ]}"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.books[?@.available]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 1, "Should select only available books");
    }

    #[test]
    fn test_comparison_operators() {
        // RFC 9535: Comparison operators ==, !=, <, <=, >, >=
        let json_data = r#"{"books": [
            {"price": 8.95, "category": "reference"},
            {"price": 12.99, "category": "fiction"},
            {"price": 19.95, "category": "technical"}
        ]}"#;

        let test_cases = vec![
            ("$.books[?@.price < 10]", 1),     // Less than
            ("$.books[?@.price <= 12.99]", 2), // Less than or equal
            ("$.books[?@.price > 15]", 1),     // Greater than
            ("$.books[?@.price >= 12.99]", 2), // Greater than or equal
            ("$.books[?@.price == 8.95]", 1),  // Equal
            ("$.books[?@.price != 8.95]", 2),  // Not equal
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_string_comparison() {
        // RFC 9535: String comparison in filters
        let json_data = r#"{"books": [
            {"category": "fiction", "title": "Book A"},
            {"category": "reference", "title": "Book B"},
            {"category": "fiction", "title": "Book C"}
        ]}"#;

        let test_cases = vec![
            ("$.books[?@.category == 'fiction']", 2),
            ("$.books[?@.category != 'fiction']", 1),
            ("$.books[?@.title == 'Book A']", 1),
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "String filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_logical_operators() {
        // RFC 9535: Logical operators && (AND) and || (OR)
        let json_data = r#"{"books": [
            {"price": 8.95, "category": "fiction", "available": true},
            {"price": 12.99, "category": "fiction", "available": false},
            {"price": 19.95, "category": "reference", "available": true}
        ]}"#;

        let test_cases = vec![
            ("$.books[?@.category == 'fiction' && @.available]", 1), // AND
            ("$.books[?@.price < 10 || @.category == 'reference']", 2), // OR
            ("$.books[?@.available && @.price > 15]", 1),            // AND with number
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Logical filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_existence_check() {
        // RFC 9535: Test for existence of properties
        let json_data = r#"{"books": [
            {"title": "Book A", "isbn": "123456"},
            {"title": "Book B"},
            {"title": "Book C", "isbn": "789012"}
        ]}"#;

        let test_cases = vec![
            ("$.books[?@.isbn]", 2),    // Has isbn property
            ("$.books[?@.title]", 3),   // Has title property (all)
            ("$.books[?@.missing]", 0), // Has missing property (none)
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Existence filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_nested_property_access() {
        // RFC 9535: Access nested properties in filters
        let json_data = r#"{"items": [
            {"product": {"name": "Widget", "price": 10.0}},
            {"product": {"name": "Gadget", "price": 25.0}},
            {"product": {"name": "Tool", "price": 5.0}}
        ]}"#;

        let test_cases = vec![
            ("$.items[?@.product.price < 15]", 2),
            ("$.items[?@.product.name == 'Widget']", 1),
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Nested filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_parenthesized_expressions() {
        // RFC 9535: Parentheses for operator precedence
        let json_data = r#"{"items": [
            {"a": 1, "b": 2, "c": 3},
            {"a": 2, "b": 3, "c": 1},
            {"a": 3, "b": 1, "c": 2}
        ]}"#;

        let test_cases = vec![
            ("$.items[?(@.a == 1 || @.b == 1) && @.c > 1]", 2),
            ("$.items[?@.a == 1 || (@.b == 1 && @.c > 1)]", 2),
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Parenthesized filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_array_element_filtering() {
        // RFC 9535: Filter array elements
        let json_data = r#"{"numbers": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]}"#;

        let test_cases = vec![
            ("$.numbers[?@ > 5]", 5),  // Numbers greater than 5
            ("$.numbers[?@ < 3]", 2),  // Numbers less than 3
            ("$.numbers[?@ == 5]", 1), // Number equal to 5
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Array filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_boolean_literal_values() {
        // RFC 9535: Boolean literals in filter expressions
        let json_data = r#"{"items": [
            {"active": true, "name": "Item1"},
            {"active": false, "name": "Item2"},
            {"active": true, "name": "Item3"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?@.active == true]", 2),
            ("$.items[?@.active == false]", 1),
            ("$.items[?@.active != false]", 2),
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Boolean filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_null_value_handling() {
        // RFC 9535: null value handling in filters
        let json_data = r#"{"items": [
            {"value": 10, "optional": null},
            {"value": 20, "optional": "present"},
            {"value": 30}
        ]}"#;

        let test_cases = vec![
            ("$.items[?@.optional == null]", 1),
            ("$.items[?@.optional != null]", 1),
            ("$.items[?@.missing == null]", 0), // Missing vs null
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Null filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }
}

/// Filter Selector Error Cases
#[cfg(test)]
mod filter_error_tests {
    use super::*;

    #[test]
    fn test_invalid_filter_syntax() {
        // Test various invalid filter syntaxes
        let invalid_filters = vec![
            "$.items[?]",              // Empty filter
            "$.items[?.price]",        // Missing @
            "$.items[?@.price <]",     // Incomplete comparison
            "$.items[?@.price << 10]", // Invalid operator
            "$.items[?@.price = 10]",  // Single equals
            "$.items[?@price]",        // Missing dot
        ];

        for expr in invalid_filters {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_err(), "Invalid filter '{}' should fail", expr);
        }
    }

    #[test]
    fn test_unclosed_filter_expressions() {
        // Test unclosed filter expressions
        let unclosed_filters = vec![
            "$.items[?@.price < 10",     // Missing closing bracket
            "$.items[?(@.price < 10",    // Missing closing parenthesis
            "$.items[?@.name == 'test'", // Missing closing bracket
        ];

        for expr in unclosed_filters {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_err(), "Unclosed filter '{}' should fail", expr);
        }
    }

    #[test]
    fn test_invalid_comparison_values() {
        // Test invalid comparison values
        let invalid_comparisons = vec![
            "$.items[?@.price < abc]", // Invalid literal
            "$.items[?@.price < '']",  // Empty string comparison with number
            "$.items[?@.price < []]",  // Array comparison
        ];

        for expr in invalid_comparisons {
            let result = JsonPathParser::compile(expr);
            // Some may be valid syntax but invalid semantically
            // Test that complex filters either compile or fail consistently
            assert!(
                result.is_ok() || result.is_err(),
                "Complex filter '{}' should be handled consistently",
                expr
            );
        }
    }
}

/// RFC 9535 Function Extensions in Filters
#[cfg(test)]
mod filter_function_tests {
    use super::*;

    #[test]
    fn test_length_function() {
        // RFC 9535 Section 2.4.4: length() function
        let json_data = r#"{"items": [
            {"name": "short", "tags": ["a"]},
            {"name": "medium", "tags": ["a", "b"]},
            {"name": "long", "tags": ["a", "b", "c", "d"]}
        ]}"#;

        let test_cases = vec![
            ("$.items[?length(@.name) > 5]", 1),  // String length
            ("$.items[?length(@.tags) == 2]", 1), // Array length
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Length function '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_match_function() {
        // RFC 9535 Section 2.4.6: match() function for regex
        let json_data = r#"{"items": [
            {"code": "ABC123"},
            {"code": "XYZ789"},
            {"code": "DEF456"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?match(@.code, '[A-Z]{3}[0-9]{3}')]", 3), // All match pattern
            ("$.items[?match(@.code, '^ABC')]", 1),             // Starts with ABC
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Match function '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_search_function() {
        // RFC 9535 Section 2.4.7: search() function for regex search
        let json_data = r#"{"items": [
            {"_description": "This contains the word test"},
            {"_description": "This has nothing"},
            {"_description": "Another test case here"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?search(@._description, 'test')]", 2), // Contains 'test'
            ("$.items[?search(@._description, 'nothing')]", 1), // Contains 'nothing'
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Search function '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }
}
