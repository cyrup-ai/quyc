//! Tests for JSONPath compiler functionality
//!
//! Tests for JSONPath expression compilation, validation, and RFC 9535 compliance

use quyc::jsonpath::compiler::JsonPathParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recursive_descent_wildcard_compilation() {
        // Test RFC 9535 compliant recursive descent patterns
        let valid_patterns = vec![
            "$..[*]",     // Valid: recursive descent followed by bracket selector
            "$..level1",  // Valid: recursive descent followed by property
            "$..['key']", // Valid: recursive descent followed by bracket selector
        ];

        for pattern in valid_patterns {
            let result = JsonPathParser::compile(pattern);
            assert!(
                result.is_ok(),
                "Pattern {} should compile successfully",
                pattern
            );
        }

        // Test RFC 9535 valid patterns including recursive descent with wildcard
        let additional_valid_patterns = vec![
            "$..*", // Valid: recursive descent followed by wildcard (RFC 9535 compliant)
        ];

        for pattern in additional_valid_patterns {
            let result = JsonPathParser::compile(pattern);
            assert!(
                result.is_ok(),
                "Pattern {} should compile successfully per RFC 9535",
                pattern
            );
        }

        // Test that invalid patterns are properly rejected
        let invalid_patterns = vec![
            "$..", // Invalid: bare recursive descent without following segment
        ];

        for pattern in invalid_patterns {
            let result = JsonPathParser::compile(pattern);
            assert!(
                result.is_err(),
                "Pattern {} should be rejected as invalid",
                pattern
            );
        }
    }
}
