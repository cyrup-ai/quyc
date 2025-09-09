//! JSONPath Parser Tests
//!
//! Tests for the core JSONPath parsing functionality and RFC 9535 core syntax compliance

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathError, JsonPathParser, JsonSelector};
use serde::{Deserialize, Serialize};

/// Test model for deserialization
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    id: String,
    value: i32,
}

#[cfg(test)]
mod parser_basic_tests {
    use super::*;

    #[test]
    fn test_simple_root_expression() {
        let expr = JsonPathParser::compile("$").expect_err("Should reject bare root");
        assert!(matches!(expr, JsonPathError::InvalidExpression { .. }));
    }

    #[test]
    fn test_child_property_access() {
        let expr = JsonPathParser::compile("$.data").expect("Valid expression");
        assert_eq!(expr.selectors().len(), 2);
        assert!(matches!(expr.selectors()[0], JsonSelector::Root));
        assert!(
            matches!(expr.selectors()[1], JsonSelector::Child { ref name, .. } if name == "data")
        );
    }

    #[test]
    fn test_array_wildcard() {
        let expr = JsonPathParser::compile("$.data[*]").expect("Valid expression");
        assert!(expr.is_array_stream());
        assert_eq!(expr.selectors().len(), 3);
        assert!(matches!(expr.selectors()[2], JsonSelector::Wildcard));
    }

    #[test]
    fn test_array_index() {
        let expr = JsonPathParser::compile("$.items[0]").expect("Valid expression");
        assert!(!expr.is_array_stream());
        assert!(matches!(
            expr.selectors()[2],
            JsonSelector::Index {
                index: 0,
                from_end: false
            }
        ));
    }

    #[test]
    fn test_negative_array_index() {
        let expr = JsonPathParser::compile("$.items[-1]").expect("Valid expression");
        assert!(matches!(
            expr.selectors()[2],
            JsonSelector::Index {
                index: -1,
                from_end: true
            }
        ));
    }

    #[test]
    fn test_array_slice() {
        let expr = JsonPathParser::compile("$.items[1:3]").expect("Valid expression");
        assert!(expr.is_array_stream());
        assert!(matches!(
            expr.selectors()[2],
            JsonSelector::Slice {
                start: Some(1),
                end: Some(3),
                step: None
            }
        ));
    }

    #[test]
    fn test_filter_expression() {
        let expr = JsonPathParser::compile("$.items[?(@.active)]").expect("Valid expression");
        assert!(expr.is_array_stream());
        assert!(matches!(expr.selectors()[2], JsonSelector::Filter { .. }));
    }

    #[test]
    fn test_complexity_scoring() {
        let simple = JsonPathParser::compile("$.data").expect("Valid expression");

        // Test simple recursive descent step by step
        let result = JsonPathParser::compile("..");
        if let Err(e) = &result {
            println!("Failed to parse '$..'': {:?}", e);
        }
        let simple_recursive = result.expect("Valid recursive descent");

        let complex = JsonPathParser::compile("$..items[?(@.active)]").expect("Valid expression");

        assert!(complex.complexity_score() > simple.complexity_score());
        assert!(complex.complexity_score() > simple_recursive.complexity_score());
    }

    #[test]
    fn test_invalid_expressions() {
        assert!(JsonPathParser::compile("").is_err());
        assert!(JsonPathParser::compile("data").is_err()); // Must start with $
        assert!(JsonPathParser::compile("$.").is_err()); // Incomplete property access
        assert!(JsonPathParser::compile("$['unclosed").is_err()); // Unterminated string
    }
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

    #[test]
    fn test_bare_root_rejected() {
        // RFC 9535: Bare '$' is not a valid expression
        let result = JsonPathParser::compile("$");
        assert!(result.is_err(), "Bare root $ should be rejected");
    }

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

        for (expr, desc) in expressions {
            let result = JsonPathParser::compile(expr);
            // Document current whitespace handling behavior
            match result {
                Ok(_) => println!("Whitespace test '{}' passed: {}", expr, desc),
                Err(_) => println!("Whitespace test '{}' failed: {}", expr, desc),
            }
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
