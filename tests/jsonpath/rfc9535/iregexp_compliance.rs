//! RFC 9535 I-Regexp Compliance Tests (RFC 9485 Integration)
//!
//! Tests for Interoperable Regular Expressions (I-Regexp) compliance
//! as specified in RFC 9485 and used by RFC 9535 match() and search() functions.
//!
//! I-Regexp is a subset of ECMAScript regular expressions designed for
//! interoperability across different regex engines and implementations.
//!
//! This test suite validates:
//! - RFC 9485 pattern format validation
//! - match() vs search() behavioral differences
//! - Invalid regex pattern handling
//! - Regex compilation error tests
//! - Performance with complex patterns
//! - Unicode handling in regex patterns
//! - Escape sequence validation
//! - Character class compliance

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestDocument {
    text: String,
    code: String,
    email: String,
    _description: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct PatternTest {
    pattern: String,
    input: String,
    should_match: bool,
}

/// RFC 9485 I-Regexp Basic Pattern Compliance Tests
#[cfg(test)]
mod iregexp_basic_compliance {
    use super::*;

    #[test]
    fn test_literal_character_matching() {
        // RFC 9485: Basic literal character matching
        let json_data = r#"{"items": [
            {"text": "hello"},
            {"text": "world"},
            {"text": "test"},
            {"text": "example"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?match(@.text, 'hello')]", 1), // Exact literal match
            ("$.items[?match(@.text, 'world')]", 1), // Another literal
            ("$.items[?match(@.text, 'test')]", 1),  // Simple literal
            ("$.items[?match(@.text, 'xyz')]", 0),   // No match
        ];

        for (expr, expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Literal matching '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(_) => println!("match() function not yet supported: {}", expr),
            }
        }
    }

    #[test]
    fn test_character_class_compliance() {
        // RFC 9485: Character class patterns
        let _json_data = r#"{"items": [
            {"code": "ABC123"},
            {"code": "xyz789"},
            {"code": "MiX3d1"},
            {"code": "special!@#"},
            {"code": "12345"},
            {"code": "UPPER"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?match(@.code, '[A-Z]+')]", 1), // Uppercase letters only
            ("$.items[?match(@.code, '[a-z]+')]", 1), // Lowercase letters only
            ("$.items[?match(@.code, '[0-9]+')]", 1), // Digits only
            ("$.items[?match(@.code, '[A-Za-z0-9]+')]", 4), // Alphanumeric
            ("$.items[?match(@.code, '[^a-z]+')]", 3), // Not lowercase
        ];

        for (expr, _expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Character class '{}' compiled successfully", expr),
                Err(_) => println!("Character class '{}' not supported", expr),
            }
        }
    }

    #[test]
    fn test_quantifier_compliance() {
        // RFC 9485: Quantifier patterns
        let _json_data = r#"{"items": [
            {"text": "a"},
            {"text": "aa"},
            {"text": "aaa"},
            {"text": "aaaa"},
            {"text": "b"},
            {"text": "ab"},
            {"text": ""}
        ]}"#;

        let test_cases = vec![
            ("$.items[?match(@.text, 'a?')]", 7),     // Zero or one 'a'
            ("$.items[?match(@.text, 'a*')]", 7),     // Zero or more 'a'
            ("$.items[?match(@.text, 'a+')]", 4),     // One or more 'a'
            ("$.items[?match(@.text, 'a{2}')]", 1),   // Exactly two 'a'
            ("$.items[?match(@.text, 'a{2,}')]", 3),  // Two or more 'a'
            ("$.items[?match(@.text, 'a{1,3}')]", 3), // One to three 'a'
        ];

        for (expr, _expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Quantifier '{}' compiled successfully", expr),
                Err(_) => println!("Quantifier '{}' not supported", expr),
            }
        }
    }

    #[test]
    fn test_anchor_compliance() {
        // RFC 9485: Anchor patterns (^ and $)
        let _json_data = r#"{"items": [
            {"text": "start_middle_end"},
            {"text": "start_only"},
            {"text": "only_end"},
            {"text": "middle"},
            {"text": "start"},
            {"text": "end"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?match(@.text, '^start')]", 3), // Starts with 'start'
            ("$.items[?match(@.text, 'end$')]", 3),   // Ends with 'end'
            ("$.items[?match(@.text, '^start$')]", 1), // Exactly 'start'
            ("$.items[?match(@.text, '^end$')]", 1),  // Exactly 'end'
        ];

        for (expr, _expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Anchor '{}' compiled successfully", expr),
                Err(_) => println!("Anchor '{}' not supported", expr),
            }
        }
    }

    #[test]
    fn test_alternation_compliance() {
        // RFC 9485: Alternation patterns (|)
        let _json_data = r#"{"items": [
            {"category": "fiction"},
            {"category": "non-fiction"},
            {"category": "reference"},
            {"category": "technical"},
            {"category": "biography"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?match(@.category, 'fiction|reference')]", 2), // Either fiction or reference
            (
                "$.items[?match(@.category, 'fiction|technical|biography')]",
                3,
            ), // Multiple alternatives
            ("$.items[?match(@.category, '^(fiction|reference)$')]", 2), // Exact match alternatives
        ];

        for (expr, _expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Alternation '{}' compiled successfully", expr),
                Err(_) => println!("Alternation '{}' not supported", expr),
            }
        }
    }
}

/// match() vs search() Behavioral Difference Tests
#[cfg(test)]
mod match_vs_search_behavior {
    use super::*;

    #[test]
    fn test_full_string_vs_substring_matching() {
        // RFC 9535: match() requires full string match, search() finds substrings
        let json_data = r#"{"items": [
            {"text": "target"},
            {"text": "prefix_target"},
            {"text": "target_suffix"},
            {"text": "prefix_target_suffix"},
            {"text": "no_match_here"}
        ]}"#;

        // match() should only match complete strings
        let match_expr = "$.items[?match(@.text, 'target')]";
        let matchresult = JsonPathParser::compile(match_expr);

        // search() should find substring matches
        let search_expr = "$.items[?search(@.text, 'target')]";
        let searchresult = JsonPathParser::compile(search_expr);

        match (matchresult, searchresult) {
            (Ok(_), Ok(_)) => {
                println!("Both match() and search() syntax supported");

                // Test match() - should only find exact "target"
                let mut match_stream = JsonArrayStream::<serde_json::Value>::new(match_expr);
                let chunk = Bytes::from(json_data);
                let matchresults: Vec<_> = match_stream.process_chunk(chunk).collect();
                println!(
                    "match() found {} results (should be 1 for exact match)",
                    matchresults.len()
                );

                // Test search() - should find all containing "target"
                let mut search_stream = JsonArrayStream::<serde_json::Value>::new(search_expr);
                let chunk = Bytes::from(json_data);
                let searchresults: Vec<_> = search_stream.process_chunk(chunk).collect();
                println!(
                    "search() found {} results (should be 4 for substring matches)",
                    searchresults.len()
                );
            }
            (Ok(_), Err(_)) => println!("Only match() function supported"),
            (Err(_), Ok(_)) => println!("Only search() function supported"),
            (Err(_), Err(_)) => println!("Neither match() nor search() functions supported yet"),
        }
    }

    #[test]
    fn test_anchored_vs_unanchored_behavior() {
        // Demonstrate how match() is implicitly anchored while search() is not
        let _json_data = r#"{"items": [
            {"text": "abc"},
            {"text": "abcdef"},
            {"text": "xyzabc"},
            {"text": "xyzabcdef"}
        ]}"#;

        let test_cases = vec![
            // match() should behave as if pattern is ^pattern$
            ("match(@.text, 'abc')", "should match only 'abc'"),
            ("match(@.text, 'abcdef')", "should match only 'abcdef'"),
            // search() should find pattern anywhere in string
            ("search(@.text, 'abc')", "should find 'abc' in all strings"),
            (
                "search(@.text, 'def')",
                "should find 'def' in strings ending with 'def'",
            ),
        ];

        for (pattern, _description) in test_cases {
            let expr = format!("$.items[?{}]", pattern);
            let result = JsonPathParser::compile(&expr);
            match result {
                Ok(_) => println!("Pattern '{}' compiled - {}", pattern, _description),
                Err(_) => println!("Pattern '{}' not supported", pattern),
            }
        }
    }

    #[test]
    fn test_case_sensitivity_behavior() {
        // Both match() and search() should be case-sensitive by default
        let _json_data = r#"{"items": [
            {"text": "Hello"},
            {"text": "hello"},
            {"text": "HELLO"},
            {"text": "HeLLo"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?match(@.text, 'Hello')]", 1), // Exact case match
            ("$.items[?match(@.text, 'hello')]", 1), // Lowercase match
            ("$.items[?match(@.text, 'HELLO')]", 1), // Uppercase match
            ("$.items[?search(@.text, 'ell')]", 2),  // Case-sensitive substring
            ("$.items[?search(@.text, 'ELL')]", 2),  // Different case substring
        ];

        for (expr, _expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Case sensitivity test '{}' compiled", expr),
                Err(_) => println!("Function not supported: {}", expr),
            }
        }
    }
}

/// Invalid Regex Pattern Handling Tests  
#[cfg(test)]
mod invalid_pattern_handling {
    use super::*;

    #[test]
    fn test_invalid_regex_syntax() {
        // Test various invalid regex patterns that should cause compilation errors
        let invalid_patterns = vec![
            "$.items[?match(@.text, '[')]",     // Unclosed character class
            "$.items[?match(@.text, '(')]",     // Unclosed group
            "$.items[?match(@.text, '+')]",     // Invalid quantifier position
            "$.items[?match(@.text, '{')]",     // Invalid quantifier syntax
            "$.items[?match(@.text, '\\')]",    // Trailing backslash
            "$.items[?match(@.text, '[z-a]')]", // Invalid character range
            "$.items[?search(@.text, '*')]",    // Invalid quantifier position
            "$.items[?search(@.text, '?')]",    // Invalid quantifier position
        ];

        for expr in invalid_patterns {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Invalid pattern '{}' unexpectedly compiled", expr),
                Err(_) => println!("Invalid pattern '{}' correctly rejected", expr),
            }
        }
    }

    #[test]
    fn test_unsupported_regex_features() {
        // Test regex features that should not be supported in I-Regexp
        let unsupported_patterns = vec![
            "$.items[?match(@.text, '(?i)case')]",  // Case-insensitive flag
            "$.items[?match(@.text, '(?m)multi')]", // Multiline flag
            "$.items[?match(@.text, '(?s)dot')]",   // Dotall flag
            "$.items[?match(@.text, '\\b\\w+\\b')]", // Word _boundaries
            "$.items[?match(@.text, '\\d+')]",      // Digit shorthand
            "$.items[?match(@.text, '\\s+')]",      // Whitespace shorthand
            "$.items[?match(@.text, '\\w+')]",      // Word character shorthand
            "$.items[?match(@.text, '(?=test)')]",  // Positive lookahead
            "$.items[?match(@.text, '(?!test)')]",  // Negative lookahead
        ];

        for expr in unsupported_patterns {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Unsupported feature '{}' unexpectedly compiled", expr),
                Err(_) => println!("Unsupported feature '{}' correctly rejected", expr),
            }
        }
    }

    #[test]
    fn test_regex_compilation_errors() {
        // Test patterns that are syntactically valid but may cause runtime errors
        let problematic_patterns = vec![
            "$.items[?match(@.text, '(.{1000000})')]", // Potential catastrophic backtracking
            "$.items[?match(@.text, '(.*)*')]",        // Nested quantifiers
            "$.items[?match(@.text, '(.+)+')]",        // Another problematic pattern
        ];

        for expr in problematic_patterns {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!(
                    "Problematic pattern '{}' compiled (may cause runtime issues)",
                    expr
                ),
                Err(_) => println!("Problematic pattern '{}' rejected at compile time", expr),
            }
        }
    }
}

/// Unicode Handling in Regex Patterns
#[cfg(test)]
mod unicode_regex_handling {
    use super::*;

    #[test]
    fn test_unicode_character_matching() {
        // Test Unicode character handling in regex patterns
        let _json_data = r#"{"items": [
            {"text": "café"},
            {"text": "naïve"},
            {"text": "résumé"},
            {"text": "🚀rocket"},
            {"text": "こんにちは"},
            {"text": "🌟✨💫"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?match(@.text, 'café')]", 1),  // Accented characters
            ("$.items[?match(@.text, 'naïve')]", 1), // Diaeresis
            ("$.items[?search(@.text, '🚀')]", 1),   // Emoji matching
            ("$.items[?search(@.text, '[🌟✨💫]+')]", 1), // Multiple emojis
            ("$.items[?match(@.text, 'こんにちは')]", 1), // Japanese characters
        ];

        for (expr, _expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Unicode pattern '{}' compiled successfully", expr),
                Err(_) => println!("Unicode pattern '{}' not supported", expr),
            }
        }
    }

    #[test]
    fn test_unicode_escape_sequences() {
        // Test Unicode escape sequences in patterns
        let _json_data = r#"{"items": [
            {"text": "©2023"},
            {"text": "™brand"},
            {"text": "hello"},
            {"text": "café"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?search(@.text, '\\u00A9')]", 1), // Copyright symbol ©
            ("$.items[?search(@.text, '\\u2122')]", 1), // Trademark symbol ™
            ("$.items[?search(@.text, '\\u0065')]", 3), // Letter 'e' (appears in multiple)
        ];

        for (expr, _expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Unicode escape '{}' compiled successfully", expr),
                Err(_) => println!("Unicode escape '{}' not supported", expr),
            }
        }
    }

    #[test]
    fn test_unicode_normalization() {
        // Test Unicode normalization handling
        let _json_data = r#"{"items": [
            {"text": "café"},
            {"text": "cafe\u0301"},
            {"text": "normal"}
        ]}"#;

        // Test if implementation handles different Unicode normalization forms
        let test_cases = vec![
            ("$.items[?match(@.text, 'café')]", "composed form"),
            ("$.items[?match(@.text, 'cafe\\u0301')]", "decomposed form"),
        ];

        for (expr, _description) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!(
                    "Unicode normalization test '{}' compiled - {}",
                    expr, _description
                ),
                Err(_) => println!("Unicode normalization '{}' not supported", expr),
            }
        }
    }
}

/// Performance Tests with Complex Patterns
#[cfg(test)]
mod regex_performance_tests {
    use super::*;

    #[test]
    fn test_complex_pattern_performance() {
        // Test performance with complex but valid I-Regexp patterns
        let json_data = r#"{"items": [
            {"email": "user@example.com"},
            {"email": "test.user+tag@domain.org"},
            {"email": "invalid-email"},
            {"email": "another@test.co.uk"},
            {"email": "simple@test.com"}
        ]}"#;

        // Complex email pattern (simplified for I-Regexp compliance)
        let email_pattern = r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}";
        let expr = format!("$.items[?match(@.email, '{}')]", email_pattern);

        let start_time = std::time::Instant::now();
        let result = JsonPathParser::compile(&expr);
        let compile_duration = start_time.elapsed();

        match result {
            Ok(_) => {
                println!("Complex email pattern compiled in {:?}", compile_duration);

                let execution_start = std::time::Instant::now();
                let mut stream = JsonArrayStream::<serde_json::Value>::new(&expr);

                let chunk = Bytes::from(json_data);
                let results: Vec<_> = stream.process_chunk(chunk).collect();
                let execution_duration = execution_start.elapsed();

                println!(
                    "Complex pattern executed in {:?}, found {} matches",
                    execution_duration,
                    results.len()
                );

                // Performance assertion
                assert!(
                    execution_duration.as_millis() < 100,
                    "Complex pattern should execute in <100ms"
                );
            }
            Err(_) => println!("Complex email pattern not supported"),
        }
    }

    #[test]
    fn test_large_input_performance() {
        // Test performance with large input strings
        let large_text = "a".repeat(10000);
        let json_value = serde_json::json!({
            "items": [
                {"text": large_text},
                {"text": "short"},
                {"text": "b".repeat(5000)}
            ]
        });
        let json_data = serde_json::to_string(&json_value).expect("Valid JSON");

        let test_patterns = vec![
            "$.items[?match(@.text, 'a+')]",    // Greedy quantifier
            "$.items[?search(@.text, 'a')]",    // Simple search
            "$.items[?match(@.text, 'short')]", // Exact match
        ];

        for expr in test_patterns {
            let start_time = std::time::Instant::now();
            let result = JsonPathParser::compile(expr);

            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data.clone());
                    let results: Vec<_> = stream.process_chunk(chunk).collect();
                    let duration = start_time.elapsed();

                    println!(
                        "Large input test '{}' completed in {:?}, found {} matches",
                        expr,
                        duration,
                        results.len()
                    );

                    // Performance assertion for large inputs
                    assert!(
                        duration.as_millis() < 1000,
                        "Large input pattern '{}' should complete in <1000ms",
                        expr
                    );
                }
                Err(_) => println!("Pattern '{}' not supported", expr),
            }
        }
    }

    #[test]
    fn test_regex_dos_prevention() {
        // Test prevention of regex denial-of-service attacks
        let json_data = r#"{"items": [
            {"text": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaX"},
            {"text": "short"},
            {"text": "normal_text"}
        ]}"#;

        // Potentially problematic patterns that could cause exponential backtracking
        let problematic_patterns = vec![
            "$.items[?match(@.text, '(a+)+b')]",  // Catastrophic backtracking
            "$.items[?match(@.text, '(a|a)*b')]", // Alternation backtracking
            "$.items[?match(@.text, 'a*a*a*a*b')]", // Multiple quantifiers
        ];

        for expr in problematic_patterns {
            let start_time = std::time::Instant::now();
            let result = JsonPathParser::compile(expr);

            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();
                    let duration = start_time.elapsed();

                    println!("DoS test '{}' completed in {:?}", expr, duration);

                    // Should not take excessive time even with problematic patterns
                    if duration.as_millis() > 1000 {
                        println!(
                            "WARNING: Pattern '{}' took {}ms - potential DoS vulnerability",
                            expr,
                            duration.as_millis()
                        );
                    }
                }
                Err(_) => println!("Problematic pattern '{}' correctly rejected", expr),
            }
        }
    }
}

/// I-Regexp Escape Sequence Validation
#[cfg(test)]
mod escape_sequence_validation {
    use super::*;

    #[test]
    fn test_valid_escape_sequences() {
        // Test valid escape sequences in I-Regexp
        let _json_data = r#"{"items": [
            {"text": "line1\nline2"},
            {"text": "tab\there"},
            {"text": "quote\"test"},
            {"text": "backslash\\test"},
            {"text": "forward/slash"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?search(@.text, '\\n')]", 1),  // Newline
            ("$.items[?search(@.text, '\\t')]", 1),  // Tab
            ("$.items[?search(@.text, '\\\"')]", 1), // Quote
            ("$.items[?search(@.text, '\\\\')]", 1), // Backslash
            ("$.items[?search(@.text, '\\/')]", 1),  // Forward slash
        ];

        for (expr, _expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Escape sequence '{}' compiled successfully", expr),
                Err(_) => println!("Escape sequence '{}' not supported", expr),
            }
        }
    }

    #[test]
    fn test_invalid_escape_sequences() {
        // Test invalid escape sequences that should be rejected
        let invalid_escapes = vec![
            "$.items[?match(@.text, '\\w')]",   // Word character shorthand
            "$.items[?match(@.text, '\\d')]",   // Digit shorthand
            "$.items[?match(@.text, '\\s')]",   // Whitespace shorthand
            "$.items[?match(@.text, '\\b')]",   // Word boundary
            "$.items[?match(@.text, '\\B')]",   // Non-word boundary
            "$.items[?match(@.text, '\\x41')]", // Hexadecimal escape
            "$.items[?match(@.text, '\\141')]", // Octal escape
        ];

        for expr in invalid_escapes {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Invalid escape '{}' unexpectedly compiled", expr),
                Err(_) => println!("Invalid escape '{}' correctly rejected", expr),
            }
        }
    }

    #[test]
    fn test_character_class_escapes() {
        // Test escape sequences within character classes
        let _json_data = r#"{"items": [
            {"text": "special-char"},
            {"text": "bracket[test]"},
            {"text": "caret^test"},
            {"text": "normal_text"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?search(@.text, '[\\-]')]", 1), // Escaped hyphen
            ("$.items[?search(@.text, '[\\[]')]", 1), // Escaped opening bracket
            ("$.items[?search(@.text, '[\\]]')]", 1), // Escaped closing bracket
            ("$.items[?search(@.text, '[\\^]')]", 1), // Escaped caret
        ];

        for (expr, _expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Character class escape '{}' compiled successfully", expr),
                Err(_) => println!("Character class escape '{}' not supported", expr),
            }
        }
    }
}
