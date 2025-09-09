//! RFC 9535 Core Requirements Tests (Section 2)
//!
//! Tests for RFC 9535 Section 2 core requirements:
//! - Well-formedness vs validity distinction tests (Section 2.1)
//! - Null vs missing value semantic tests (Section 2.6)
//! - Normalized paths canonical form enforcement tests (Section 2.7)
//!
//! This test suite validates:
//! - Well-formed but invalid JSONPath expressions
//! - Null value vs missing property semantics
//! - Canonical normalized path forms
//! - Semantic equivalence requirements

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathError, JsonPathParser};

/// Test data for core requirements validation
const CORE_TEST_JSON: &str = r#"{
  "store": {
    "book": [
      {
        "category": "reference",
        "author": "Nigel Rees",
        "title": "Sayings of the Century", 
        "price": 8.95,
        "isbn": "0-553-21311-3",
        "metadata": null,
        "tags": ["classic", "quotes"]
      },
      {
        "category": "fiction",
        "author": "Evelyn Waugh",
        "title": "Sword of Honour",
        "price": 12.99,
        "availability": null,
        "tags": ["fiction", "war"]
      },
      {
        "category": "fiction",
        "author": "Herman Melville",
        "title": "Moby Dick",
        "price": 8.99,
        "tags": null
      }
    ],
    "bicycle": {
      "color": "red",
      "price": 19.95,
      "availability": null
    }
  },
  "expensive": 10,
  "missing_test": {
    "present": "value",
    "null_value": null,
    "empty_string": "",
    "zero": 0,
    "false_value": false,
    "empty_array": [],
    "empty_object": {}
  }
}"#;

/// RFC 9535 Section 2.1 - Well-formedness vs Validity Tests
#[cfg(test)]
mod well_formedness_validity_tests {
    use super::*;

    #[test]
    fn test_well_formed_but_invalid_expressions() {
        // RFC 9535 Section 2.1: Well-formed expressions that are invalid
        let well_formed_invalid = vec![
            // Syntactically correct but semantically invalid
            ("$.store.book[999]", true, "Out of bounds array index"),
            ("$.store.nonexistent", true, "Non-existent property"),
            ("$.store.book[-999]", true, "Out of bounds negative index"),
            (
                "$.store.book[0].missing_property",
                true,
                "Missing nested property",
            ),
            (
                "$.store.book[?@.nonexistent == 'value']",
                true,
                "Filter on missing property",
            ),
            // Well-formed expressions with type mismatches
            ("$.store.book.author", true, "Property access on array"),
            ("$.store.bicycle[0]", true, "Array access on object"),
            ("$.expensive.property", true, "Property access on primitive"),
            // Well-formed but logically invalid filters
            ("$.store.book[?@.price > @.price]", true, "Self-comparison"),
            (
                "$.store.book[?@.price == @.title]",
                true,
                "Type mismatch comparison",
            ),
            (
                "$.store.book[?@.author.length]",
                true,
                "Property on primitive",
            ),
        ];

        for (expr, _should_compile, _description) in well_formed_invalid {
            let result = JsonPathParser::compile(expr);

            if _should_compile {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Well-formed expression should compile: {} ({})",
                    expr,
                    _description
                );

                // Test execution - should not crash but may return empty results
                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(CORE_TEST_JSON);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                // Should execute without error, even if no results
                println!(
                    "Well-formed invalid '{}' -> {} results ({})",
                    expr,
                    results.len(),
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Malformed expression should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_malformed_expressions() {
        // RFC 9535 Section 2.1: Malformed expressions (not well-formed)
        let malformed_expressions = vec![
            // Syntax errors
            ("$.", false, "Trailing dot"),
            ("$.store[", false, "Unclosed bracket"),
            ("$.store]", false, "Unmatched bracket"),
            ("$.store.book[0", false, "Unclosed array access"),
            ("$.store.book[?@.price >]", false, "Incomplete comparison"),
            (
                "$.store.book[?@.price > 10 &&]",
                false,
                "Incomplete logical expression",
            ),
            // Invalid tokens
            ("store", false, "Missing root $"),
            (
                "$.store..book",
                false,
                "Double dots without recursive descent",
            ),
            ("$.store...book", false, "Triple dots"),
            ("$.store.book[*,]", false, "Trailing comma"),
            ("$.store.book[,*]", false, "Leading comma"),
            // Invalid filter syntax
            ("$.store.book[?]", false, "Empty filter"),
            ("$.store.book[@.price > 10]", false, "Missing ? in filter"),
            ("$.store.book[?price > 10]", false, "Missing @ in filter"),
            ("$.store.book[?@.price ++ 10]", false, "Invalid operator"),
            // Invalid escape sequences
            ("$['unclosed string]", false, "Unclosed string"),
            ("$['invalid\\escape']", false, "Invalid escape sequence"),
            ("$['\\unicode']", false, "Invalid unicode escape"),
        ];

        for (expr, _should_compile, _description) in malformed_expressions {
            let result = JsonPathParser::compile(expr);

            assert!(
                result.is_err(),
                "RFC 9535: Malformed expression should be rejected: {} ({})",
                expr,
                _description
            );

            // Verify error message is informative
            if let Err(JsonPathError::InvalidExpression { reason, .. }) = result {
                assert!(
                    !reason.is_empty(),
                    "Error message should not be empty for: {}",
                    expr
                );
            }
        }
    }

    #[test]
    fn test_validity_semantic_checks() {
        // RFC 9535 Section 2.1: Semantic validity beyond syntax
        let semantic_validity_tests = vec![
            // Valid semantics
            ("$.store.book[0].author", true, "Valid property chain"),
            ("$.store.book[?@.price > 5]", true, "Valid filter"),
            ("$.store.book[*].title", true, "Valid wildcard usage"),
            ("$..price", true, "Valid recursive descent"),
            // Questionable but syntactically valid semantics
            (
                "$.store.book[0].author.length",
                true,
                "Property on string (valid in some contexts)",
            ),
            ("$.store.book[999999]", true, "Very large array index"),
            ("$[0][1][2][3][4]", true, "Deep array access chain"),
            ("$.a.b.c.d.e.f.g.h.i.j", true, "Very deep property chain"),
        ];

        for (expr, _should_be_valid, _description) in semantic_validity_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Semantically valid expression should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Semantically invalid expression should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }
}

/// RFC 9535 Section 2.6 - Null vs Missing Value Semantics Tests
#[cfg(test)]
mod null_vs_missing_semantics_tests {
    use super::*;

    #[test]
    fn test_null_value_vs_missing_property() {
        // RFC 9535 Section 2.6: Distinguish null values from missing properties
        let null_vs_missing_tests = vec![
            // Properties that exist but have null values
            ("$.missing_test.null_value", 1, "Null value property exists"),
            ("$.store.book[0].metadata", 1, "Null metadata exists"),
            (
                "$.store.book[1].availability",
                1,
                "Null availability exists",
            ),
            ("$.store.book[2].tags", 1, "Null tags exists"),
            (
                "$.store.bicycle.availability",
                1,
                "Null bicycle availability",
            ),
            // Properties that don't exist (missing)
            (
                "$.missing_test.nonexistent",
                0,
                "Missing property returns nothing",
            ),
            ("$.store.book[0].missing_field", 0, "Missing book field"),
            (
                "$.store.book[0].author.missing",
                0,
                "Missing nested property",
            ),
            ("$.store.missing_section", 0, "Missing store section"),
            // Edge cases with falsy values that are NOT null
            ("$.missing_test.empty_string", 1, "Empty string exists"),
            ("$.missing_test.zero", 1, "Zero value exists"),
            ("$.missing_test.false_value", 1, "False value exists"),
            ("$.missing_test.empty_array", 1, "Empty array exists"),
            ("$.missing_test.empty_object", 1, "Empty object exists"),
        ];

        for (expr, expected_count, _description) in null_vs_missing_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(CORE_TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: Null vs missing test: {} ({}) should return {} results",
                expr,
                _description,
                expected_count
            );
        }
    }

    #[test]
    fn test_null_value_filter_semantics() {
        // RFC 9535 Section 2.6: Null values in filter expressions
        let null_filter_tests = vec![
            // Explicit null comparisons
            (
                "$.store.book[?@.metadata == null]",
                1,
                "Filter for null metadata",
            ),
            (
                "$.store.book[?@.availability == null]",
                1,
                "Filter for null availability",
            ),
            ("$.store.book[?@.tags == null]", 1, "Filter for null tags"),
            // Non-null comparisons
            (
                "$.store.book[?@.metadata != null]",
                2,
                "Filter for non-null metadata",
            ),
            (
                "$.store.book[?@.author != null]",
                3,
                "Filter for non-null author",
            ),
            (
                "$.store.book[?@.price != null]",
                3,
                "Filter for non-null price",
            ),
            // Missing property comparisons (should not match null)
            (
                "$.store.book[?@.nonexistent == null]",
                0,
                "Missing property not equal to null",
            ),
            (
                "$.store.book[?@.nonexistent != null]",
                0,
                "Missing property not not-equal to null",
            ),
            // Truthiness tests with null values
            (
                "$.store.book[?@.metadata]",
                0,
                "Null value is falsy in test",
            ),
            ("$.store.book[?@.author]", 3, "Non-null strings are truthy"),
            ("$.store.book[?@.tags]", 2, "Non-null arrays are truthy"),
        ];

        for (expr, expected_count, _description) in null_filter_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(CORE_TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: Null filter test: {} ({}) should return {} results",
                expr,
                _description,
                expected_count
            );
        }
    }

    #[test]
    fn test_missing_vs_null_edge_cases() {
        // RFC 9535 Section 2.6: Edge cases for null vs missing semantics
        let edge_case_json = r#"{
            "explicit_null": null,
            "nested": {
                "also_null": null
            },
            "array_with_nulls": [null, "value", null],
            "object_with_nulls": {
                "null_prop": null,
                "real_prop": "value"
            }
        }"#;

        let edge_case_tests = vec![
            // Direct null access
            ("$.explicit_null", 1, "Direct null property"),
            ("$.nested.also_null", 1, "Nested null property"),
            ("$.array_with_nulls[0]", 1, "Null in array position 0"),
            ("$.array_with_nulls[2]", 1, "Null in array position 2"),
            ("$.object_with_nulls.null_prop", 1, "Null in nested object"),
            // Missing property access
            ("$.missing_property", 0, "Missing top-level property"),
            ("$.nested.missing_property", 0, "Missing nested property"),
            ("$.array_with_nulls[5]", 0, "Missing array index"),
            (
                "$.object_with_nulls.missing_prop",
                0,
                "Missing object property",
            ),
            // Recursive descent with nulls and missing
            ("$..null_prop", 1, "Recursive descent finds null"),
            (
                "$..missing_prop",
                0,
                "Recursive descent doesn't find missing",
            ),
            (
                "$..*",
                9,
                "Recursive descent finds all values including nulls",
            ),
        ];

        for (expr, expected_count, _description) in edge_case_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(edge_case_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: Null vs missing edge case: {} ({}) should return {} results",
                expr,
                _description,
                expected_count
            );
        }
    }
}

/// RFC 9535 Section 2.7 - Normalized Paths Canonical Form Tests
#[cfg(test)]
mod normalized_paths_tests {
    use super::*;

    #[test]
    fn test_normalized_path_equivalences() {
        // RFC 9535 Section 2.7: Equivalent expressions should produce same results
        let equivalence_tests = vec![
            // Dot vs bracket notation equivalences
            (
                "$.store.book",
                "$['store']['book']",
                "Dot vs bracket notation",
            ),
            (
                "$.store.book[0].author",
                "$['store']['book'][0]['author']",
                "Mixed dot and bracket",
            ),
            (
                "$.store.bicycle.color",
                "$['store']['bicycle']['color']",
                "Property chain equivalence",
            ),
            // Array access equivalences
            (
                "$.store.book[0]",
                "$.store['book'][0]",
                "Array access with bracket property",
            ),
            (
                "$.store.book[1].title",
                "$.store.book[1]['title']",
                "Array element property access",
            ),
            // Wildcard equivalences (when applicable)
            (
                "$.store.book[*].author",
                "$.store['book'][*]['author']",
                "Wildcard with bracket notation",
            ),
        ];

        for (expr1, expr2, _description) in equivalence_tests {
            let mut stream1 = JsonArrayStream::<serde_json::Value>::new(expr1);
            let mut stream2 = JsonArrayStream::<serde_json::Value>::new(expr2);

            let chunk = Bytes::from(CORE_TEST_JSON);
            let results1: Vec<_> = stream1.process_chunk(chunk.clone()).collect();
            let results2: Vec<_> = stream2.process_chunk(chunk).collect();

            assert_eq!(
                results1.len(),
                results2.len(),
                "RFC 9535: Equivalent expressions should produce same results: '{}' vs '{}' ({})",
                expr1,
                expr2,
                _description
            );

            println!(
                "✓ Normalized equivalence: '{}' ≡ '{}' ({} results) ({})",
                expr1,
                expr2,
                results1.len(),
                _description
            );
        }
    }

    #[test]
    fn test_canonical_normalized_forms() {
        // RFC 9535 Section 2.7: Canonical normalized path forms
        let canonical_tests = vec![
            // Preferred canonical forms
            ("$.store.book", true, "Canonical dot notation"),
            ("$.store.book[0]", true, "Canonical array access"),
            ("$.store.book[0].author", true, "Canonical mixed access"),
            // Non-canonical but equivalent forms
            ("$['store']['book']", true, "Non-canonical bracket form"),
            ("$.store['book']", true, "Mixed notation"),
            ("$['store'].book", true, "Reverse mixed notation"),
            // Check that all compile to equivalent internal representations
            ("$.store.book[*].title", true, "Canonical wildcard"),
            (
                "$['store']['book'][*]['title']",
                true,
                "Non-canonical wildcard",
            ),
        ];

        for (expr, _should_be_valid, _description) in canonical_tests {
            let result = JsonPathParser::compile(expr);

            assert!(
                result.is_ok() == _should_be_valid,
                "RFC 9535: Canonical form test: {} ({}) validity: {}",
                expr,
                _description,
                _should_be_valid
            );
        }
    }

    #[test]
    fn test_path_normalization_edge_cases() {
        // RFC 9535 Section 2.7: Edge cases in path normalization
        let normalization_edge_cases = vec![
            // Empty property names
            ("$['']", true, "Empty string property"),
            ("$.''", false, "Empty string in dot notation (invalid)"),
            // Special character property names
            ("$['property-with-hyphens']", true, "Hyphens in property"),
            (
                "$.property_with_underscores",
                true,
                "Underscores in dot notation",
            ),
            ("$['property.with.dots']", true, "Dots in bracket notation"),
            (
                "$.property.with.dots",
                false,
                "Dots in dot notation (interpreted as chain)",
            ),
            // Numeric property names
            ("$['123']", true, "Numeric string property"),
            ("$.123", false, "Numeric property in dot notation"),
            ("$[123]", true, "Array index vs string property"),
            // Unicode normalization
            ("$['café']", true, "Unicode property name"),
            ("$['cafe\\u0301']", true, "Unicode with combining character"),
            // Case sensitivity
            ("$.store.Book", true, "Case sensitive property"),
            ("$.store.book", true, "Original case property"),
        ];

        for (expr, _should_be_valid, _description) in normalization_edge_cases {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid normalization case should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid normalization case should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_semantic_equivalence_validation() {
        // RFC 9535 Section 2.7: Semantic equivalence validation
        let semantic_equivalence_tests = vec![
            // These should behave identically despite different notation
            (
                "$.store.book[0].author",
                "$['store']['book'][0]['author']",
                "Full bracket vs dot equivalence",
            ),
            (
                "$.store.book[?@.price > 10]",
                "$['store']['book'][?@.price > 10]",
                "Filter with different property access",
            ),
            (
                "$.store.book[*].category",
                "$['store']['book'][*]['category']",
                "Wildcard with bracket notation",
            ),
            (
                "$..author",
                "$..**['author']",
                "Recursive descent property access",
            ),
        ];

        for (expr1, expr2, _description) in semantic_equivalence_tests {
            // Both should compile successfully
            let result1 = JsonPathParser::compile(expr1);
            let result2 = JsonPathParser::compile(expr2);

            assert!(
                result1.is_ok() && result2.is_ok(),
                "RFC 9535: Both equivalent expressions should compile: '{}' and '{}' ({})",
                expr1,
                expr2,
                _description
            );

            // Both should produce same results when executed
            let mut stream1 = JsonArrayStream::<serde_json::Value>::new(expr1);
            let mut stream2 = JsonArrayStream::<serde_json::Value>::new(expr2);

            let chunk = Bytes::from(CORE_TEST_JSON);
            let results1: Vec<_> = stream1.process_chunk(chunk.clone()).collect();
            let results2: Vec<_> = stream2.process_chunk(chunk).collect();

            assert_eq!(
                results1.len(),
                results2.len(),
                "RFC 9535: Semantically equivalent expressions should produce same results: '{}' vs '{}' ({})",
                expr1,
                expr2,
                _description
            );
        }
    }
}

/// RFC 9535 Section 2.1.2 - Nodelist Ordering Preservation Tests
#[cfg(test)]
mod nodelist_ordering_tests {
    use super::*;

    #[test]
    fn test_multi_selector_overlappingresults_ordering() {
        // RFC 9535 Section 2.1.2: "The nodelist resulting from the last segment is presented as the result of the query"
        // "A segment operates on each of the nodes in its input nodelist in turn, and the resultant nodelists
        // are concatenated in the order of the input nodelist they were derived from to produce the result of the segment"

        let test_json = r#"{
            "items": [
                {"id": 1, "type": "book", "category": "fiction"},
                {"id": 2, "type": "book", "category": "science"},
                {"id": 3, "type": "magazine", "category": "fiction"},
                {"id": 4, "type": "book", "category": "history"}
            ]
        }"#;

        // Test case 1: Index selector ordering tests
        let index_selector_tests = vec![
            (
                "$.items[0,2,1]", // Selectors in specific order
                vec![1, 3, 2],    // Expected IDs in selector order
                "Multiple index selectors should preserve selector order",
            ),
            (
                "$.items[2,0,3,1]", // Different selector order
                vec![3, 1, 4, 2],   // Expected IDs in selector order
                "Index selectors should maintain specified order regardless of array position",
            ),
        ];

        // Test case 2: Property selector ordering tests
        let property_selector_tests = vec![(
            "$['items'][0,1]['id','type']",   // Nested multi-selectors
            vec!["id", "type", "id", "type"], // Should interleave based on selector order
            "Nested multi-selectors should preserve concatenation order",
        )];

        // Test index selector ordering
        for (expr, expected_pattern, _description) in index_selector_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_pattern.len(),
                "RFC 9535 nodelist ordering: {} should produce {} results",
                expr,
                expected_pattern.len()
            );

            for (i, result) in results.iter().enumerate() {
                let id_value = result["id"].as_i64().unwrap_or(-1);
                assert_eq!(
                    id_value, expected_pattern[i] as i64,
                    "RFC 9535: {} - Position {} should have ID {}, got {}",
                    _description, i, expected_pattern[i], id_value
                );
            }
        }

        // Test property selector ordering
        for (expr, expected_fields, _description) in property_selector_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_fields.len(),
                "RFC 9535 nodelist ordering: {} should produce {} results",
                expr,
                expected_fields.len()
            );

            // Check alternating id/type pattern
            for (i, result) in results.iter().enumerate() {
                let expected_field = expected_fields[i];
                if expected_field == "id" {
                    assert!(
                        result.is_number(),
                        "RFC 9535: Position {} should be numeric id, got {:?}",
                        i,
                        result
                    );
                } else {
                    assert!(
                        result.is_string(),
                        "RFC 9535: Position {} should be string type, got {:?}",
                        i,
                        result
                    );
                }
            }
        }
    }

    #[test]
    fn test_duplicate_node_preservation() {
        // RFC 9535 Section 2.1.2: "A node may be selected more than once and appears that number of times in the nodelist. Duplicate nodes are not removed."

        let test_json = r#"{
            "data": {
                "x": 42,
                "y": 24  
            }
        }"#;

        let test_cases = vec![
            (
                "$.data['x','x','y','x']", // Duplicate selectors
                vec![42, 42, 24, 42],      // Should preserve all duplicates in order
                "Duplicate selectors should preserve all occurrences in selector order",
            ),
            (
                "$['data','data'].x", // Duplicate path segments
                vec![42, 42],         // Should get x twice
                "Duplicate path segments should produce duplicate results",
            ),
        ];

        for (expr, expected_values, _description) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_values.len(),
                "RFC 9535 duplicate preservation: {} should produce {} results",
                expr,
                expected_values.len()
            );

            for (i, result) in results.iter().enumerate() {
                let value = result.as_i64().unwrap_or(-1);
                assert_eq!(
                    value, expected_values[i] as i64,
                    "RFC 9535: {} - Position {} should be {}, got {}",
                    _description, i, expected_values[i], value
                );
            }
        }
    }

    #[test]
    fn test_empty_nodelist_propagation() {
        // RFC 9535 Section 2.1.2: "As a consequence of this approach, if any of the segments produces an empty nodelist, then the whole query produces an empty nodelist."

        let test_json = r#"{
            "store": {
                "book": [{"title": "Book 1"}],
                "magazine": [{"title": "Magazine 1"}]
            }
        }"#;

        let empty_producing_expressions = vec![
            "$.store.nonexistent.anything", // nonexistent produces empty, should propagate
            "$.store.book[99].title",       // out-of-bounds index produces empty
            "$.store[''].title",            // empty string selector on object without empty key
            "$.store.book[?@.missing].title", // filter that matches nothing
        ];

        for expr in empty_producing_expressions {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                0,
                "RFC 9535 empty propagation: Expression '{}' should produce empty nodelist due to empty segment propagation",
                expr
            );
        }
    }
}
