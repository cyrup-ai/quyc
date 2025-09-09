//! JSONPath Parser Tests
//!
//! Tests for the core JSONPath parsing functionality, extracted from inline tests

use quyc::jsonpath::{JsonPathParser, JsonSelector};

#[cfg(test)]
mod parser_basic_tests {
    use super::*;

    #[test]
    fn test_simple_root_expression() {
        let expr = JsonPathParser::compile("$").expect("Bare root is valid per RFC 9535");
        assert_eq!(expr.selectors().len(), 1);
        assert!(matches!(expr.selectors()[0], JsonSelector::Root));
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

        // Test simple recursive descent - RFC 9535 compliant pattern
        let simple_recursive = JsonPathParser::compile("$..[*]").expect("Valid recursive descent");

        // Use RFC 9535 compliant pattern for complex expression
        let complex = JsonPathParser::compile("$..[?@.active]").expect("Valid expression");

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
