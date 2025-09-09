//! RFC 9535 Unicode & String Handling Compliance Tests
//!
//! Tests for complete escape sequence validation (Table 4), Unicode scalar value handling,
//! single/double quote string literals, surrogate pair processing, case sensitivity validation

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};

#[cfg(test)]
mod unicode_escape_sequence_tests {
    use super::*;

    #[test]
    fn test_basic_escape_sequences() {
        // RFC 9535 Table 4: Basic escape sequences
        let test_data = r#"{"strings": [
            "backspace: \b",
            "tab: \t", 
            "newline: \n",
            "formfeed: \f",
            "carriage_return: \r",
            "quote: \"",
            "apostrophe: \'",
            "solidus: \/",
            "backslash: \\"
        ]}"#;

        let test_cases = vec![
            ("$.strings[0]", 1), // Backspace
            ("$.strings[1]", 1), // Tab
            ("$.strings[2]", 1), // Newline
            ("$.strings[3]", 1), // Form feed
            ("$.strings[4]", 1), // Carriage return
            ("$.strings[5]", 1), // Quote
            ("$.strings[6]", 1), // Apostrophe
            ("$.strings[7]", 1), // Solidus
            ("$.strings[8]", 1), // Backslash
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Escape sequence test '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_unicode_escape_sequences() {
        // RFC 9535: Unicode escape sequences \uXXXX
        let test_data = r#"{"unicode": [
            "euro: \u20AC",
            "copyright: \u00A9", 
            "trademark: \u2122",
            "smile: \u263A",
            "snowman: \u2603"
        ]}"#;

        let test_cases = vec![
            ("$.unicode[0]", 1), // Euro symbol - first item
            ("$.unicode[1]", 1), // Copyright - second item
            ("$.unicode[2]", 1), // Trademark - third item
            ("$.unicode[3]", 1), // Smile - fourth item
            ("$.unicode[4]", 1), // Snowman - fifth item
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Unicode escape test '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_invalid_escape_sequences() {
        // RFC 9535: Invalid escape sequences should cause errors
        let invalid_expressions = vec![
            r#"$.data[?@.text == "invalid \x escape"]"#, // Invalid \x escape
            r#"$.data[?@.text == "incomplete \u123"]"#,  // Incomplete unicode
            r#"$.data[?@.text == "invalid \z escape"]"#, // Unknown escape
        ];

        for expr in invalid_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Invalid escape sequence '{}' should fail parsing",
                expr
            );
        }
    }
}

#[cfg(test)]
mod unicode_character_handling_tests {
    use super::*;

    #[test]
    fn test_unicode_character_length() {
        // RFC 9535: length() function must count Unicode characters, not bytes
        let test_data = r#"{"text": [
            "ascii",
            "café", 
            "🌟⭐✨",
            "日本語",
            "مرحبا"
        ]}"#;

        let test_cases = vec![
            ("$.text[?length(@) == 5]", 2), // "ascii" (5 chars) and "مرحبا" (5 chars)
            ("$.text[?length(@) == 4]", 1), // "café" (4 chars)
            ("$.text[?length(@) == 3]", 2), // "🌟⭐✨" and "日本語" (3 chars each)
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Unicode character length test '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_emoji_and_grapheme_clusters() {
        // RFC 9535: Complex Unicode including emoji and grapheme clusters
        let test_data = r#"{"emojis": [
            "👨‍👩‍👧‍👦",
            "🧑🏽‍💻",
            "🇺🇸",
            "👍🏿",
            "🏳️‍🌈"
        ]}"#;

        // Test that complex emoji are handled correctly in string operations
        let test_cases = vec![
            ("$.emojis[?@ =~ '.*👨.*']", 1), // Family emoji
            ("$.emojis[?@ =~ '.*💻.*']", 1), // Technologist emoji
            ("$.emojis[?@ =~ '.*🇺🇸.*']", 1), // Flag emoji
            ("$.emojis[?@ =~ '.*👍.*']", 1), // Thumbs up with skin tone
            ("$.emojis[?@ =~ '.*🏳.*']", 1),  // Rainbow flag
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Emoji handling test '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_surrogate_pairs() {
        // RFC 9535: Proper handling of Unicode surrogate pairs
        let test_data = r#"{"surrogates": [
            "High surrogate: \uD800\uDC00",
            "Musical note: \uD834\uDD1E", 
            "Emoji: \uD83D\uDE00"
        ]}"#;

        let test_cases = vec![
            ("$.surrogates[?@ =~ '.*\\\\uD800\\\\uDC00.*']", 1), // Basic surrogate pair
            ("$.surrogates[?@ =~ '.*\\\\uD834\\\\uDD1E.*']", 1), // Musical symbol
            ("$.surrogates[?@ =~ '.*\\\\uD83D\\\\uDE00.*']", 1), // Emoji via surrogates
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Surrogate pair test '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_invalid_surrogate_pairs() {
        // RFC 9535: Invalid surrogate pairs should be handled gracefully
        let invalid_surrogates = vec![
            r#"{"text": "orphaned high: \uD800"}"#, // Orphaned high surrogate
            r#"{"text": "orphaned low: \uDC00"}"#,  // Orphaned low surrogate
            r#"{"text": "reversed: \uDC00\uD800"}"#, // Reversed surrogate pair
        ];

        for _json_data in invalid_surrogates {
            // These should either parse correctly (replacement chars) or fail gracefully
            let mut stream = JsonArrayStream::<serde_json::Value>::new("$.text");
            let chunk = Bytes::from(_json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // Should not panic - either successful parsing with replacement chars or controlled error
            println!("Invalid surrogate handling result count: {}", results.len());
        }
    }
}

#[cfg(test)]
mod string_literal_quoting_tests {
    use super::*;

    #[test]
    fn test_single_vs_double_quotes() {
        // RFC 9535: Both single and double quotes should be supported
        let test_data = r#"{"data": "test_value"}"#;

        let test_cases = vec![
            (r#"$.data[?@ == "test_value"]"#, true),  // Double quotes
            (r#"$.data[?@ == 'test_value']"#, true),  // Single quotes
            (r#"$.data[?@ == "test_value']"#, false), // Mismatched quotes
            (r#"$.data[?@ == 'test_value"]"#, false), // Mismatched quotes
        ];

        for (expr, should_parse) in test_cases {
            let result = JsonPathParser::compile(expr);
            if should_parse {
                assert!(
                    result.is_ok(),
                    "Expression '{}' should parse successfully",
                    expr
                );
            } else {
                assert!(
                    result.is_err(),
                    "Expression '{}' should fail to parse",
                    expr
                );
            }
        }
    }

    #[test]
    fn test_nested_quote_escaping() {
        // RFC 9535: Proper escaping of quotes within strings
        let test_data = r#"{"quotes": [
            "He said \"Hello\"",
            "She said 'Goodbye'",
            "Mixed: \"It's fine\"",
            "Escaped: 'He said \"Hi\"'"
        ]}"#;

        let test_cases = vec![
            (r#"$.quotes[?@ =~ '.*\"Hello\".*']"#, 1), // Escaped double quotes
            (r#"$.quotes[?@ =~ ".*'Goodbye'.*"]"#, 1), // Single quotes in double-quoted string
            (r#"$.quotes[?@ =~ '.*\"It\'s fine\".*']"#, 1), // Mixed quote escaping
            (r#"$.quotes[?@ =~ ".*'He said \\\"Hi\\\"'.*"]"#, 1), // Complex nested escaping
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Quote escaping test '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }
}

#[cfg(test)]
mod case_sensitivity_tests {
    use super::*;

    #[test]
    fn test_property_name_case_sensitivity() {
        // RFC 9535: Property names are case-sensitive
        let test_data = r#"{"Data": "uppercase", "data": "lowercase", "DATA": "allcaps"}"#;

        let test_cases = vec![
            ("$.Data", "uppercase"),
            ("$.data", "lowercase"),
            ("$.DATA", "allcaps"),
        ];

        for (expr, expected_value) in test_cases {
            let mut stream = JsonArrayStream::<String>::new(expr);
            let chunk = Bytes::from(test_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                1,
                "Expression '{}' should find exactly one result",
                expr
            );
            let result = &results[0];
            assert_eq!(
                result, expected_value,
                "Property '{}' should return '{}'",
                expr, expected_value
            );
        }
    }

    #[test]
    fn test_string_comparison_case_sensitivity() {
        // RFC 9535: String comparisons are case-sensitive by default
        let test_data = r#"{"items": [
            {"name": "Apple"},
            {"name": "apple"}, 
            {"name": "APPLE"}
        ]}"#;

        let test_cases = vec![
            (r#"$.items[?@.name == "Apple"]"#, 1), // Exact case match
            (r#"$.items[?@.name == "apple"]"#, 1), // Lowercase match
            (r#"$.items[?@.name == "APPLE"]"#, 1), // Uppercase match
            (r#"$.items[?@.name == "ApPlE"]"#, 0), // Mixed case no match
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Case sensitivity test '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_unicode_case_sensitivity() {
        // RFC 9535: Unicode case sensitivity
        let test_data = r#"{"unicode": [
            {"text": "Café"},
            {"text": "café"},
            {"text": "CAFÉ"}
        ]}"#;

        let test_cases = vec![
            (r#"$.unicode[?@.text == "Café"]"#, 1), // Mixed case
            (r#"$.unicode[?@.text == "café"]"#, 1), // Lowercase
            (r#"$.unicode[?@.text == "CAFÉ"]"#, 1), // Uppercase
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Unicode case sensitivity test '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }
}

#[cfg(test)]
mod string_normalization_tests {
    use super::*;

    #[test]
    fn test_unicode_normalization() {
        // RFC 9535: Unicode normalization considerations
        let test_data = r#"{"normalized": [
            {"text": "café"},
            {"text": "cafe\u0301"},
            {"text": "caf\u00E9"}
        ]}"#;

        // Test that different Unicode normalizations are handled consistently
        let test_cases = vec![
            (r#"$.normalized[?@.text =~ '.*café.*']"#, 3), // Should match all forms
            (r#"$.normalized[?@.text =~ '.*\\u0301.*']"#, 1), // Combining diacritic
            (r#"$.normalized[?@.text =~ '.*\\u00E9.*']"#, 1), // Precomposed character
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Unicode normalization test '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_zero_width_characters() {
        // RFC 9535: Zero-width characters should be handled correctly
        let test_data = r#"{"zwc": [
            "normal",
            "zero\u200Bwidth",
            "zero\u200Cwidth\u200Djoiner"
        ]}"#;

        let test_cases = vec![
            ("$.zwc[?length(@) == 6]", 1),                 // "normal"
            (r#"$.zwc[?@ =~ '.*\\u200B.*']"#, 1),          // Zero-width space
            (r#"$.zwc[?@ =~ '.*\\u200C.*\\u200D.*']"#, 1), // Zero-width non-joiner + joiner
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Zero-width character test '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }
}
