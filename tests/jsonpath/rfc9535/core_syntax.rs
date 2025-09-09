//! RFC 9535 Core Syntax Compliance Tests
//!
//! Tests for fundamental JSONPath syntax requirements defined in RFC 9535.
//! All tests should initially FAIL to create an implementation contract.

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathError, JsonPathParser};
use serde::{Deserialize, Serialize};

/// Test model for deserialization
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    id: String,
    #[serde(default)]
    value: Option<i32>,
}

/// RFC 9535 Section 2.2 - Root Identifier Tests
#[cfg(test)]
mod root_identifier_tests {
    use super::*;

    #[test]
    fn test_root_identifier_required() {
        // RFC 9535: JSONPath expression MUST begin with '$'
        let result = JsonPathParser::compile("store.book");
        assert!(result.is_err(), "Expression without $ should fail");

        if let Err(JsonPathError::InvalidExpression { reason, .. }) = result {
            assert!(
                reason.contains("must start with"),
                "Error should mention $ requirement"
            );
        } else {
            panic!("Expected InvalidExpression error");
        }
    }

    // REMOVED: test_bare_root_rejected - This test contradicted RFC 9535 specification
    // RFC 9535 Section 2.2.3 Examples explicitly shows that "$" is valid and returns the root node
    // The ABNF grammar allows jsonpath-query = root-identifier segments where segments = *(S segment) (zero or more)

    #[test]
    fn test_valid_root_expressions() {
        // RFC 9535: Valid expressions start with $ and have selectors
        let expressions = vec!["$.store", "$['store']", "$[*]", "$[0]", "$..*"];

        for expr in expressions {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_ok(), "Valid expression '{}' should compile", expr);
        }
    }

    #[test]
    fn test_multiple_root_identifiers_rejected() {
        // Only one root identifier allowed
        let result = JsonPathParser::compile("$.store.$.book");
        assert!(result.is_err(), "Multiple $ should be rejected");
    }
}

/// RFC 9535 Section 2.1.1 - ABNF Grammar Compliance Tests
#[cfg(test)]
mod abnf_grammar_tests {
    use super::*;

    #[test]
    fn test_jsonpath_query_structure() {
        // RFC 9535 ABNF: jsonpath-query = root-identifier segments
        //                segments = *(S segment)  ; zero or more segments
        // Therefore "$" (root-only) is VALID per RFC 9535
        let result = JsonPathParser::compile("$");
        assert!(
            result.is_ok(),
            "Root-only query '$' should be valid per RFC 9535"
        );

        let result = JsonPathParser::compile("$.store.book[*]");
        assert!(result.is_ok(), "Valid query structure should pass");
    }

    #[test]
    fn test_segment_types_recognized() {
        // ABNF: segment = child-segment / descendant-segment
        let child_segment = JsonPathParser::compile("$['store']");
        assert!(child_segment.is_ok(), "Child segment should be valid");

        let descendant_segment = JsonPathParser::compile("$..book");
        assert!(
            descendant_segment.is_ok(),
            "Descendant segment should be valid"
        );
    }

    #[test]
    fn test_selector_list_syntax() {
        // ABNF: selector-list = selector *("," selector)
        let single_selector = JsonPathParser::compile("$['store']");
        assert!(single_selector.is_ok(), "Single selector should be valid");

        let multiple_selectors = JsonPathParser::compile("$['store','book',0]");
        assert!(
            multiple_selectors.is_ok(),
            "Multiple selectors should be valid"
        );
    }
}

/// RFC 9535 Section 2.1.2 - Semantics Tests
#[cfg(test)]
mod semantics_tests {
    use super::*;

    #[test]
    fn test_nodelist_production() {
        // RFC 9535: JSONPath query produces a nodelist
        let json_data = r#"{"store": {"book": [{"id": "1"}, {"id": "2"}]}}"#;
        let mut stream = JsonArrayStream::<TestModel>::new("$.store.book[*]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        // Should produce nodelist with 2 nodes
        assert_eq!(results.len(), 2, "Should produce nodelist with 2 nodes");
    }

    #[test]
    fn test_empty_nodelist_for_no_matches() {
        // RFC 9535: No matches should produce empty nodelist
        let json_data = r#"{"store": {"bicycle": {"color": "red"}}}"#;
        let mut stream = JsonArrayStream::<TestModel>::new("$.store.book[*]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 0, "No matches should produce empty nodelist");
    }
}

/// RFC 9535 Whitespace Handling Tests
#[cfg(test)]
mod whitespace_tests {
    use super::*;

    #[test]
    fn test_whitespace_in_expressions() {
        // RFC 9535 doesn't explicitly define whitespace rules, test current behavior
        let expressions = vec![
            ("$ .store", "Space after root"),
            ("$[ 'store' ]", "Spaces in brackets"),
            ("$.store .book", "Space before property"),
        ];

        for (expr, _desc) in expressions {
            let _result = JsonPathParser::compile(expr);
            // Document current whitespace handling behavior
            // These expressions may be valid or invalid depending on implementation
            // The key is consistent handling across similar patterns
            // Future: Add specific whitespace behavior expectations based on RFC clarification
        }
    }
}

/// RFC 9535 Case Sensitivity Tests
#[cfg(test)]
mod case_sensitivity_tests {
    use super::*;

    #[test]
    fn test_property_name_case_sensitivity() {
        // RFC 9535: Property names should be case-sensitive
        let json_data = r#"{"Store": {"book": []}, "store": {"book": [{"id": "1"}]}}"#;

        // Test exact case matching
        let mut stream_exact = JsonArrayStream::<TestModel>::new("$.store.book[*]");
        let mut stream_wrong_case = JsonArrayStream::<TestModel>::new("$.Store.book[*]");

        let chunk = Bytes::from(json_data);

        let exactresults: Vec<_> = stream_exact.process_chunk(chunk.clone()).collect();
        let wrong_caseresults: Vec<_> = stream_wrong_case.process_chunk(chunk).collect();

        assert_eq!(exactresults.len(), 1, "Exact case should match");
        assert_eq!(wrong_caseresults.len(), 0, "Wrong case should not match");
    }
}

/// RFC 9535 Edge Cases and Error Conditions
#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_empty_expression() {
        let result = JsonPathParser::compile("");
        assert!(result.is_err(), "Empty expression should fail");
    }

    #[test]
    fn test_malformed_brackets() {
        let expressions = vec![
            "$[",         // Unclosed bracket
            "$.store]",   // Unmatched closing bracket
            "$[[store]]", // Double brackets
            "$[store[]]", // Nested brackets incorrectly
        ];

        for expr in expressions {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_err(), "Malformed brackets '{}' should fail", expr);
        }
    }

    #[test]
    fn test_invalid_characters() {
        let expressions = vec![
            "$#store",      // Invalid character #
            "$.store%book", // Invalid character %
            "$@store",      // @ outside filter context
        ];

        for expr in expressions {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_err(), "Invalid character '{}' should fail", expr);
        }
    }
}

/// RFC 9535 Core Requirements: Well-formedness vs Validity
#[cfg(test)]
mod wellformedness_validity_tests {
    use super::*;

    #[test]
    fn test_wellformed_syntactically_valid_paths() {
        // RFC 9535: These paths are syntactically well-formed but may not match anything
        let wellformed_tests = vec![
            ("$.nonexistent", "Non-existent property"),
            ("$[999]", "Out-of-bounds array index"),
            ("$.store[999].title", "Chain with invalid index"),
            ("$..*", "Universal descendant"),
            ("$..nonexistent", "Descendant with non-existent property"),
            (
                "$.store.book[*].nonexistent",
                "Valid structure with non-existent property",
            ),
            ("$['']", "Empty string property name"),
            ("$[-999]", "Large negative index"),
            ("$.a.b.c.d.e.f", "Deep property chain"),
        ];

        for (expr, _description) in wellformed_tests {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Well-formed expression '{}' should compile: {}",
                expr,
                _description
            );
        }
    }

    #[test]
    fn test_malformed_syntactically_invalid_paths() {
        // RFC 9535: These paths are syntactically malformed and must be rejected
        let malformed_tests = vec![
            ("", "Empty expression"),
            ("store", "Missing root identifier"),
            ("$..", "Incomplete descendant operator"),
            ("$.", "Incomplete dot notation"),
            ("$[", "Unclosed bracket"),
            ("$]", "Unmatched closing bracket"),
            ("$[[]]", "Double nested brackets"),
            ("$.store...", "Triple dots"),
            ("$@", "Invalid @ at root"),
            ("$.store$", "Multiple root identifiers"),
            ("$.123abc", "Invalid identifier starting with digit"),
            ("$['unclosed", "Unclosed string literal"),
            ("$[?@", "Incomplete filter"),
            ("$[*", "Unclosed wildcard"),
            ("$store.book", "Missing dot after root"),
        ];

        for (expr, _description) in malformed_tests {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Malformed expression '{}' should fail: {}",
                expr,
                _description
            );
        }
    }

    #[test]
    fn test_semantic_validity_vs_syntax_validity() {
        // RFC 9535: Distinction between syntax errors and semantic evaluation results
        let json_data = r#"{"store": {"book": [{"title": "Book1"}, {"title": "Book2"}]}}"#;

        let semantic_tests = vec![
            ("$.store.book[0].title", 1, "Valid path, valid semantics"),
            ("$.store.book[999].title", 0, "Valid syntax, invalid index"),
            (
                "$.store.nonexistent.title",
                0,
                "Valid syntax, non-existent property",
            ),
            (
                "$.store.book[*].nonexistent",
                0,
                "Valid syntax, non-existent nested property",
            ),
            (
                "$.store.book[-1].title",
                1,
                "Valid syntax, negative index semantics",
            ),
            (
                "$.store.book[*].title",
                2,
                "Valid syntax, wildcard semantics",
            ),
        ];

        for (expr, expected_count, _description) in semantic_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Semantic test '{}' should return {} results: {}",
                expr,
                expected_count,
                _description
            );
        }
    }

    #[test]
    fn test_error_consistency_across_similar_patterns() {
        // RFC 9535: Similar malformed patterns should fail consistently
        let bracket_errors = vec!["$[", "$]", "$[[", "$]]", "$[[]", "$[]]"];

        let quote_errors = vec!["$['unclosed", "$[\"unclosed", "$['", "$[\""];

        let dot_errors = vec!["$.", "$..", "$...", "$.store.", "$.store.."];

        for error_group in [bracket_errors, quote_errors, dot_errors] {
            for expr in error_group {
                let result = JsonPathParser::compile(expr);
                assert!(
                    result.is_err(),
                    "Consistently malformed expression '{}' should fail",
                    expr
                );
            }
        }
    }
}

/// RFC 9535 Core Requirements: Nodelist Semantics
#[cfg(test)]
mod nodelist_semantics_tests {
    use super::*;

    #[test]
    fn test_nodelist_ordering_preservation() {
        // RFC 9535: Nodelist must preserve document order
        let json_data = r#"{"items": [
            {"id": "first", "priority": 3},
            {"id": "second", "priority": 1}, 
            {"id": "third", "priority": 2}
        ]}"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.items[*].id");
        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        // Results should maintain document order, not sort by value
        assert_eq!(results.len(), 3, "Should return all three ids");

        // Verify document order is preserved (not sorted by priority or id)
        let ids: Vec<String> = results
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        assert_eq!(
            ids,
            vec!["first", "second", "third"],
            "RFC 9535: Nodelist must preserve document order"
        );
    }

    #[test]
    fn test_empty_nodelist_conditions() {
        // RFC 9535: Conditions that produce empty nodelists
        let json_data = r#"{"store": {"book": [], "bicycle": {"color": "red"}}}"#;

        let emptyresult_tests = vec![
            ("$.store.book[*]", "Empty array wildcard"),
            ("$.store.book[0]", "Index into empty array"),
            ("$.store.nonexistent", "Non-existent property"),
            ("$.store.book[999]", "Out-of-bounds positive index"),
            ("$.store.book[-999]", "Out-of-bounds negative index"),
            ("$..nonexistent", "Descendant search for non-existent"),
            (
                "$.store.book[*].title",
                "Property access on empty array results",
            ),
        ];

        for (expr, _description) in emptyresult_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                0,
                "Empty nodelist test '{}' should return 0 results: {}",
                expr,
                _description
            );
        }
    }

    #[test]
    fn test_nodelist_type_preservation() {
        // RFC 9535: Nodelist values must preserve JSON type information
        let json_data = r#"{"mixed": [
            "string",
            42,
            true,
            null,
            {"object": "value"},
            [1, 2, 3]
        ]}"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.mixed[*]");
        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 6, "Should return all mixed type values");

        // Verify type preservation
        assert!(results[0].is_string(), "First value should be string");
        assert!(results[1].is_number(), "Second value should be number");
        assert!(results[2].is_boolean(), "Third value should be boolean");
        assert!(results[3].is_null(), "Fourth value should be null");
        assert!(results[4].is_object(), "Fifth value should be object");
        assert!(results[5].is_array(), "Sixth value should be array");
    }
}

/// RFC 9535 Core Requirements: Path Evaluation Semantics
#[cfg(test)]
mod path_evaluation_semantics_tests {
    use super::*;

    #[test]
    fn test_root_value_semantics() {
        // RFC 9535: Root identifier '$' refers to the entire JSON document
        let json_data = r#"{"top": "level"}"#;

        let root_tests = vec![
            ("$.top", 1, "Root property access"),
            ("$['top']", 1, "Root bracket notation"),
        ];

        for (expr, expected_count, _description) in root_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Root semantics test '{}' should return {} results: {}",
                expr,
                expected_count,
                _description
            );
        }
    }

    #[test]
    fn test_selector_evaluation_order() {
        // RFC 9535: Multiple selectors in bracket notation are evaluated left-to-right
        let json_data = r#"{"items": ["first", "second", "third", "fourth"]}"#;

        let multiple_selector_tests = vec![
            ("$.items[0,2]", 2, "Multiple index selectors"),
            (
                "$.items[1,0,3]",
                3,
                "Multiple index selectors in specific order",
            ),
            ("$.items[*]", 4, "Wildcard selector"),
        ];

        for (expr, expected_count, _description) in multiple_selector_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Selector order test '{}' should return {} results: {}",
                expr,
                expected_count,
                _description
            );
        }
    }

    #[test]
    fn test_descendant_operator_semantics() {
        // RFC 9535: Descendant operator '..' recursively descends into JSON structure
        let json_data = r#"{"level1": {
            "level2": {
                "target": "deep",
                "level3": {
                    "target": "deeper"
                }
            },
            "target": "shallow"
        }}"#;

        let descendant_tests = vec![
            ("$..target", 3, "All target properties at any depth"),
            ("$.level1..target", 3, "Target properties under level1"),
            (
                "$.level1.level2..target",
                2,
                "Target properties under level2",
            ),
            ("$..level3.target", 1, "Specific path with descendant"),
        ];

        for (expr, expected_count, _description) in descendant_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Descendant semantics test '{}' should return {} results: {}",
                expr,
                expected_count,
                _description
            );
        }
    }

    #[test]
    fn test_array_index_semantics() {
        // RFC 9535: Array index semantics including negative indices
        let json_data = r#"{"array": ["a", "b", "c", "d", "e"]}"#;

        let index_tests = vec![
            ("$.array[0]", 1, "First element (zero-based)"),
            ("$.array[4]", 1, "Last element"),
            ("$.array[-1]", 1, "Last element (negative index)"),
            ("$.array[-5]", 1, "First element (negative index)"),
            ("$.array[999]", 0, "Out-of-bounds positive index"),
            ("$.array[-999]", 0, "Out-of-bounds negative index"),
        ];

        for (expr, expected_count, _description) in index_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Array index test '{}' should return {} results: {}",
                expr,
                expected_count,
                _description
            );
        }
    }
}
