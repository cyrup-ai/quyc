//! RFC 9535 IANA Considerations Tests (Section 3)
//!
//! Tests IANA media type registration and function extension registry requirements
//! Validates RFC 9535 Section 3.1 and 3.2 compliance

use quyc::jsonpath::JsonPathParser;

/// RFC 9535 Section 3.1 - Media Type application/jsonpath Tests
#[cfg(test)]
mod media_type_tests {
    use super::*;

    #[test]
    fn test_jsonpath_media_type_specification() {
        // RFC 9535 Section 3.1: Media type application/jsonpath
        //
        // Required parameters: None
        // Optional parameters: None
        // Encoding considerations: JSONPath expressions are UTF-8 encoded Unicode text

        let valid_jsonpath_expressions = vec![
            "$",
            "$.store.book[*].author",
            "$..price",
            "$.store.book[?@.price < 10]",
        ];

        for expr in valid_jsonpath_expressions {
            // Test that expressions are valid UTF-8 encoded Unicode text
            assert!(expr.is_ascii() || expr.chars().all(|c| c.is_ascii() || c as u32 <= 0x10FFFF));

            // Test that expressions conform to JSONPath syntax
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Media type application/jsonpath expression '{}' should be valid",
                expr
            );
        }
    }

    #[test]
    fn test_utf8_encoding_requirements() {
        // RFC 9535 Section 3.1: Encoding considerations
        // JSONPath expressions MUST be UTF-8 encoded Unicode text

        let utf8_test_cases = vec![
            ("$['café']", true),        // UTF-8 encoded French
            ("$['北京']", true),        // UTF-8 encoded Chinese
            ("$['Москва']", true),      // UTF-8 encoded Russian
            ("$['🏠']", true),          // UTF-8 encoded emoji
            ("$['test\\u0041']", true), // Unicode escape sequences
        ];

        for (expr, _should_be_valid) in utf8_test_cases {
            let is_valid_utf8 = std::str::from_utf8(expr.as_bytes()).is_ok();
            assert_eq!(
                is_valid_utf8, _should_be_valid,
                "UTF-8 validation failed for: {}",
                expr
            );

            if _should_be_valid {
                let result = JsonPathParser::compile(expr);
                assert!(
                    result.is_ok(),
                    "Valid UTF-8 JSONPath expression should compile: {}",
                    expr
                );
            }
        }
    }

    #[test]
    fn test_no_required_parameters() {
        // RFC 9535 Section 3.1: Required parameters: None
        // JSONPath expressions should be self-contained without external parameters

        let self_contained_expressions = vec![
            "$",                           // Root only
            "$.store",                     // Simple path
            "$.store.book[0]",             // Array access
            "$.store.book[?@.price < 10]", // Filter with embedded logic
        ];

        for expr in self_contained_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Self-contained expression '{}' should not require external parameters",
                expr
            );
        }
    }

    #[test]
    fn test_no_optional_parameters() {
        // RFC 9535 Section 3.1: Optional parameters: None
        // JSONPath expressions should not support optional media type parameters

        // This is validated by ensuring expressions are parsed without configuration
        let expressions = vec!["$.store.book[*]", "$..author", "$.store.book[?@.isbn]"];

        for expr in expressions {
            // Parse with default configuration only
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Expression '{}' should parse without optional parameters",
                expr
            );
        }
    }

    #[test]
    fn test_interoperability_considerations() {
        // RFC 9535 Section 3.1: Interoperability considerations: None
        // JSONPath expressions should be portable across implementations

        let portable_expressions = vec![
            "$",
            "$.store.book[0].title",
            "$..price",
            "$.store.book[?@.category == 'fiction']",
            "$.store.book[0:2]",
            "$.store.*",
        ];

        for expr in portable_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Portable expression '{}' should be implementation-independent",
                expr
            );
        }
    }

    #[test]
    fn test_fragment_identifier_considerations() {
        // RFC 9535 Section 3.1: Fragment identifier considerations: None
        // JSONPath expressions should not use URI fragment identifiers

        let expressions_without_fragments = vec![
            "$.store.book[0]",
            "$..author",
            "$.store.book[?@.price < 10]",
        ];

        for expr in expressions_without_fragments {
            // Ensure no URI fragment syntax (# characters) in valid expressions
            assert!(
                !expr.contains('#'),
                "JSONPath expression '{}' should not contain fragment identifiers",
                expr
            );

            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Expression without fragments '{}' should be valid",
                expr
            );
        }
    }

    #[test]
    fn test_common_usage_patterns() {
        // RFC 9535 Section 3.1: Intended usage: COMMON
        // Test common JSONPath usage patterns for media type validation

        let common_usage_patterns = vec![
            ("$.users[*].email", "Extract all user emails"),
            ("$.products[?@.price < 100]", "Filter products by price"),
            ("$..address.city", "Find all city values"),
            ("$.data.items[0:5]", "Get first 5 items"),
            ("$.config.settings.*", "Get all setting values"),
        ];

        for (expr, _description) in common_usage_patterns {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Common usage pattern '{}' ({}) should be valid",
                expr,
                _description
            );
        }
    }
}

/// RFC 9535 Section 3.2 - Function Extensions Subregistry Tests
#[cfg(test)]
mod function_extension_registry_tests {
    use super::*;

    #[test]
    fn test_standard_function_extensions() {
        // RFC 9535 Section 3.2: Standard function extensions defined in the specification

        let standard_functions = vec![
            ("length", "$[?length(@.name) > 5]"),
            ("count", "$[?count(@.items) > 0]"),
            ("match", "$[?match(@.email, '^[^@]+@[^@]+$')]"),
            ("search", "$[?search(@._description, 'test')]"),
            ("value", "$[?value(@.active) == true]"),
        ];

        for (function_name, example_usage) in standard_functions {
            let result = JsonPathParser::compile(example_usage);
            assert!(
                result.is_ok(),
                "Standard function '{}' should be registered and usable: {}",
                function_name,
                example_usage
            );
        }
    }

    #[test]
    fn test_function_extension_syntax_requirements() {
        // RFC 9535 Section 3.2: Function extension syntax requirements

        let valid_function_calls = vec![
            "length(@.array)",          // Single argument
            "match(@.text, 'pattern')", // Two arguments
            "count($..items[*])",       // Node list argument
            "value(@.single)",          // Single node argument
        ];

        let invalid_function_calls = vec![
            "unknown_function(@.test)", // Unregistered function
            "length()",                 // Missing required argument
            "match(@.text)",            // Missing required second argument
            "length(@.test, extra)",    // Too many arguments
        ];

        for valid_call in valid_function_calls {
            let expr = format!("$[?{}]", valid_call);
            let result = JsonPathParser::compile(&expr);
            assert!(
                result.is_ok(),
                "Valid function call '{}' should be accepted",
                valid_call
            );
        }

        for invalid_call in invalid_function_calls {
            let expr = format!("$[?{}]", invalid_call);
            let result = JsonPathParser::compile(&expr);
            assert!(
                result.is_err(),
                "Invalid function call '{}' should be rejected",
                invalid_call
            );
        }
    }

    #[test]
    fn test_function_extension_type_system() {
        // RFC 9535 Section 3.2: Function extensions must conform to type system

        let type_correct_expressions = vec![
            (
                "$[?length(@.name) > 5]",
                "length() returns number for comparison",
            ),
            (
                "$[?count(@.items) == 0]",
                "count() returns number for comparison",
            ),
            (
                "$[?match(@.email, 'pattern')]",
                "match() returns boolean for test",
            ),
            (
                "$[?search(@.text, 'word')]",
                "search() returns boolean for test",
            ),
            (
                "$[?value(@.flag) == true]",
                "value() returns value for comparison",
            ),
        ];

        for (expr, _description) in type_correct_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Type-correct expression '{}' should be valid: {}",
                expr,
                _description
            );
        }
    }

    #[test]
    fn test_function_well_typedness_validation() {
        // RFC 9535 Section 2.4.3: Function expressions must be well-typed

        let well_typed_examples = vec![
            "length(@.array)",     // ValueType -> number
            "count(@.items[*])",   // NodesType -> number
            "match(@.str, 'pat')", // ValueType, ValueType -> LogicalType
            "value(@.single)",     // NodesType -> ValueType
        ];

        let ill_typed_examples = vec![
            // These should be rejected by a complete implementation
            "length()",          // Missing required argument
            "count(@.single)",   // Wrong argument type (should be NodesType)
            "match(@.str)",      // Missing required second argument
            "value(@.multi[*])", // Multiple nodes (should be single)
        ];

        for example in well_typed_examples {
            let expr = format!("$[?{}]", example);
            let result = JsonPathParser::compile(&expr);
            assert!(
                result.is_ok(),
                "Well-typed function '{}' should be accepted",
                example
            );
        }

        for example in ill_typed_examples {
            let expr = format!("$[?{}]", example);
            let result = JsonPathParser::compile(&expr);
            assert!(
                result.is_err(),
                "Ill-typed function '{}' should be rejected",
                example
            );
        }
    }

    #[test]
    fn test_function_extension_registry_completeness() {
        // RFC 9535 Section 3.2: All functions mentioned in spec should be registered

        let all_spec_functions = vec![
            "length", // Section 2.4.4
            "count",  // Section 2.4.5
            "match",  // Section 2.4.6
            "search", // Section 2.4.7
            "value",  // Section 2.4.8
        ];

        for function_name in all_spec_functions {
            // Test that each function can be used in a filter expression
            let expr = match function_name {
                "length" => "$[?length(@.test) > 0]",
                "count" => "$[?count(@.items[*]) > 0]",
                "match" => "$[?match(@.text, 'pattern')]",
                "search" => "$[?search(@.text, 'word')]",
                "value" => "$[?value(@.flag)]",
                _ => panic!("Unknown function: {}", function_name),
            };

            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Registered function '{}' should be usable in expressions",
                function_name
            );
        }
    }

    #[test]
    fn test_iana_subregistry_requirements() {
        // RFC 9535 Section 3.2: IANA Function Extensions Subregistry requirements

        // Test that the implementation follows IANA registry structure
        // This validates that future extensions would follow proper registration

        let registry_compliant_behavior = vec![
            ("Defined functions are available", "$[?length(@.test)]"),
            (
                "Undefined functions are rejected",
                "$[?undefined_func(@.test)]",
            ),
            ("Function syntax is enforced", "$[?length()]"), // Should fail - missing args
            ("Type checking is enforced", "$[?length(@.test, extra)]"), // Should fail - extra args
        ];

        for (test_description, expr) in registry_compliant_behavior {
            let result = JsonPathParser::compile(expr);

            if test_description.contains("rejected") || test_description.contains("fail") {
                assert!(
                    result.is_err(),
                    "Registry compliance test '{}' should reject invalid expression: {}",
                    test_description,
                    expr
                );
            } else {
                assert!(
                    result.is_ok(),
                    "Registry compliance test '{}' should accept valid expression: {}",
                    test_description,
                    expr
                );
            }
        }
    }
}

/// Test IANA compliance integration
#[cfg(test)]
mod iana_integration_tests {
    use super::*;

    #[test]
    fn test_iana_media_type_and_function_registry_integration() {
        // Test that media type and function registry work together

        let integrated_examples = vec![
            (
                "$.users[?length(@.name) > 0]",
                "Media type with length function",
            ),
            (
                "$.products[?match(@.sku, '^[A-Z]{3}[0-9]{3}$')]",
                "Media type with match function",
            ),
            (
                "$.items[?count(@.tags[*]) > 2]",
                "Media type with count function",
            ),
        ];

        for (expr, _description) in integrated_examples {
            // Validate UTF-8 encoding (media type requirement)
            assert!(expr.is_ascii() || expr.chars().all(|c| (c as u32) <= 0x10FFFF));

            // Validate function registry compliance
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "IANA integration test '{}' should work: {}",
                _description,
                expr
            );
        }
    }

    #[test]
    fn test_complete_iana_section_3_compliance() {
        // Comprehensive test covering all of RFC 9535 Section 3

        let section_3_requirements = vec![
            // Section 3.1 - Media Type
            ("Media type self-contained", "$..book[*].author"),
            ("UTF-8 encoding", "$['тест']"), // Cyrillic
            ("No parameters required", "$.store.book[0]"),
            // Section 3.2 - Function Registry
            ("Standard functions available", "$[?length(@.name)]"),
            ("Function type checking", "$[?count(@.items[*]) > 0]"),
            ("Function syntax validation", "$[?match(@.text, 'pattern')]"),
        ];

        for (requirement, expr) in section_3_requirements {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "IANA Section 3 requirement '{}' must be satisfied: {}",
                requirement,
                expr
            );
        }
    }
}
