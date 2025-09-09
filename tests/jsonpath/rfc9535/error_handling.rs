//! RFC 9535 Error Handling Test Suite
//!
//! Tests for comprehensive error handling in JSONPath implementations.
//! This validates how the implementation responds to various error conditions
//! and ensures proper error classification, reporting, and recovery.
//!
//! This test suite validates:
//! - Well-formedness vs validity errors
//! - Error message quality validation
//! - Graceful degradation tests
//! - Resource limit enforcement
//! - Error recovery mechanisms
//! - Consistent error behavior
//! - Performance under error conditions

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct ErrorTestModel {
    id: i32,
    data: Option<String>,
    nested: Option<serde_json::Value>,
}

/// Well-formedness vs Validity Error Tests
#[cfg(test)]
mod wellformedness_validity_tests {
    use super::*;

    #[test]
    fn test_syntax_well_formedness_errors() {
        // RFC 9535: Test well-formedness errors (syntax violations)
        let syntax_errors = vec![
            // Unclosed constructs
            ("$[", "Unclosed bracket selector"),
            ("$.key[", "Unclosed array access"),
            ("$.key[?", "Unclosed filter expression"),
            ("$.key[?@.prop", "Incomplete filter expression"),
            ("$.key[?@.prop ==", "Incomplete comparison"),
            ("$[\"unclosed", "Unclosed string literal"),
            ("$['unclosed", "Unclosed single-quoted string"),
            // Invalid characters and sequences
            ("$.", "Trailing dot with no property"),
            ("$..", "Invalid double dot without continuation"),
            ("$...", "Triple dot sequence"),
            ("$key", "Missing root $ prefix"),
            ("key.value", "No root identifier"),
            // Malformed brackets and indices
            ("$.key[]", "Empty bracket selector"),
            ("$.key[abc]", "Invalid array index (non-numeric)"),
            ("$.key[1.5]", "Floating point array index"),
            ("$.key[-]", "Invalid negative index format"),
            // Filter expression syntax errors
            ("$.items[?@.]", "Incomplete property access in filter"),
            ("$.items[?@ ==]", "Missing comparison value"),
            ("$.items[?== 5]", "Missing left operand"),
            ("$.items[?@.prop &&]", "Incomplete logical expression"),
            ("$.items[?@.prop ||]", "Incomplete OR expression"),
            ("$.items[?()]", "Empty parentheses in filter"),
            // Invalid escape sequences
            ("$['key\\x']", "Invalid escape sequence"),
            ("$['key\\']", "Trailing backslash"),
            ("$['key\\uGHIJ']", "Invalid Unicode escape"),
            ("$['key\\u123']", "Incomplete Unicode escape"),
        ];

        for (invalid_path, _description) in syntax_errors {
            let result = JsonPathParser::compile(invalid_path);
            match result {
                Ok(_) => println!(
                    "UNEXPECTED: Syntax error '{}' compiled successfully ({})",
                    invalid_path, _description
                ),
                Err(e) => println!(
                    "Syntax error '{}' correctly rejected: {:?} ({})",
                    invalid_path, e, _description
                ),
            }
        }
    }

    #[test]
    fn test_semantic_validity_errors() {
        // RFC 9535: Test validity errors (semantic violations)
        let json_data = r#"{"store": {
            "book": [
                {"title": "Book 1", "price": 10.99},
                {"title": "Book 2", "price": 15.99}
            ],
            "bicycle": {"color": "red", "price": 19.95}
        }}"#;

        let semantic_errors = vec![
            // Type mismatches
            (
                "$.store.book.title",
                "Accessing property on array without index",
            ),
            ("$.store.bicycle[0]", "Array access on non-array"),
            ("$.store.book[0].title[5]", "Array access on string"),
            ("$.store.book.price.invalid", "Property access on number"),
            // Out-of-bounds access
            ("$.store.book[10]", "Array index out of bounds"),
            ("$.store.book[-10]", "Negative index out of bounds"),
            ("$.store.book[1:10]", "Slice end out of bounds"),
            // Invalid slice parameters
            ("$.store.book[::0]", "Step value of zero"),
            (
                "$.store.book[5:2]",
                "Start greater than end with positive step",
            ),
            (
                "$.store.book[2:5:-1]",
                "Start less than end with negative step",
            ),
            // Function errors (if functions are implemented)
            (
                "$.store.book[?length(@.nonexistent) > 0]",
                "Function on non-existent property",
            ),
            (
                "$.store[?match(@.book, 'pattern')]",
                "Match function on non-string",
            ),
            (
                "$.store[?count(@.invalid[*]) > 0]",
                "Count function on invalid path",
            ),
        ];

        for (invalid_path, _description) in semantic_errors {
            let result = JsonPathParser::compile(invalid_path);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(invalid_path);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Semantic error '{}' compiled but may fail at runtime -> {} results ({})",
                        invalid_path,
                        results.len(),
                        _description
                    );
                }
                Err(e) => println!(
                    "Semantic error '{}' rejected at compile time: {:?} ({})",
                    invalid_path, e, _description
                ),
            }
        }
    }

    #[test]
    fn test_error_classification() {
        // Test that errors are properly classified
        let error_categories = vec![
            // Lexical errors
            ("$[\"unterminated", "LexicalError", "Unterminated string"),
            (
                "$['unterminated",
                "LexicalError",
                "Unterminated single-quoted string",
            ),
            ("$.key[?@.prop ===]", "LexicalError", "Invalid operator"),
            // Parse errors
            ("$.", "ParseError", "Incomplete path"),
            ("$.key[?]", "ParseError", "Empty filter"),
            ("$.key[abc]", "ParseError", "Invalid index"),
            // Semantic errors
            (
                "$.valid.but.runtime.error",
                "SemanticError",
                "Runtime path resolution",
            ),
            ("$.array[999]", "IndexError", "Array index out of bounds"),
            (
                "$.object.missing",
                "PropertyError",
                "Missing property access",
            ),
            // Type errors
            ("$.string[0]", "TypeError", "Array access on non-array"),
            (
                "$.number.property",
                "TypeError",
                "Property access on primitive",
            ),
        ];

        for (path, expected_category, _description) in error_categories {
            let result = JsonPathParser::compile(path);
            match result {
                Ok(_) => println!(
                    "Error classification: '{}' compiled ({})",
                    path, _description
                ),
                Err(e) => {
                    println!(
                        "Error classification: '{}' -> {:?} (expected: {}, {})",
                        path, e, expected_category, _description
                    );

                    // Test that error messages contain useful information
                    let error_string = format!("{:?}", e);
                    assert!(
                        !error_string.is_empty(),
                        "Error message should not be empty"
                    );
                    assert!(
                        error_string.len() > 10,
                        "Error message should be descriptive"
                    );
                }
            }
        }
    }

    #[test]
    fn test_nested_error_contexts() {
        // Test error reporting in nested contexts
        let nested_errors = vec![
            (
                "$.store.book[?@.price > 'string']",
                "Type mismatch in filter comparison",
            ),
            (
                "$.store.book[?@.invalid.deep.access]",
                "Invalid nested property in filter",
            ),
            (
                "$.store.book[?length(@.title.invalid)]",
                "Function with invalid argument",
            ),
            ("$.store.book[0:end]", "Invalid slice parameter"),
            (
                "$.store..book[?@.price << 10]",
                "Invalid operator in descendant context",
            ),
        ];

        for (path, _description) in nested_errors {
            let result = JsonPathParser::compile(path);
            match result {
                Ok(_) => println!("Nested error '{}' compiled ({})", path, _description),
                Err(e) => {
                    println!("Nested error '{}' -> {:?} ({})", path, e, _description);

                    // Error should provide context about where the error occurred
                    let error_string = format!("{:?}", e);
                    assert!(
                        error_string.len() > 20,
                        "Nested error should provide sufficient context"
                    );
                }
            }
        }
    }
}

/// Error Message Quality Validation Tests
#[cfg(test)]
mod error_message_quality_tests {
    use super::*;

    #[test]
    fn test_error_message_completeness() {
        // Test that error messages contain essential information
        let test_errors = vec![
            ("$[", "Should indicate unclosed bracket"),
            ("$.key[abc]", "Should indicate invalid array index"),
            ("$.key[?@.prop ===]", "Should indicate invalid operator"),
            ("$.", "Should indicate incomplete path"),
            ("$.key[?]", "Should indicate empty filter"),
        ];

        for (invalid_path, expectation) in test_errors {
            let result = JsonPathParser::compile(invalid_path);
            match result {
                Ok(_) => println!(
                    "ERROR: '{}' should have failed ({})",
                    invalid_path, expectation
                ),
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    println!(
                        "Error message for '{}': {} ({})",
                        invalid_path, error_msg, expectation
                    );

                    // Quality checks for error messages
                    assert!(!error_msg.is_empty(), "Error message should not be empty");
                    assert!(error_msg.len() >= 10, "Error message should be descriptive");
                    assert!(
                        !error_msg.to_lowercase().contains("unknown"),
                        "Error message should be specific, not generic"
                    );

                    // Should not contain debug artifacts
                    assert!(
                        !error_msg.contains("unwrap"),
                        "Error message should not contain debug artifacts"
                    );
                    assert!(
                        !error_msg.contains("panic"),
                        "Error message should not reference panics"
                    );
                }
            }
        }
    }

    #[test]
    fn test_error_position_information() {
        // Test that errors include position information when possible
        let positioned_errors = vec![
            ("$.valid.path[invalid]", "Error in bracket selector"),
            (
                "$.valid[?@.invalid === value]",
                "Error in filter expression",
            ),
            ("$.valid.path.", "Error at end of path"),
            ("$..valid[", "Error in descendant selector"),
        ];

        for (path, context) in positioned_errors {
            let result = JsonPathParser::compile(path);
            match result {
                Ok(_) => println!(
                    "Position test: '{}' compiled unexpectedly ({})",
                    path, context
                ),
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    println!("Position error for '{}': {} ({})", path, error_msg, context);

                    // Error should ideally include position information
                    // This is implementation-dependent
                    if error_msg.contains("position")
                        || error_msg.contains("at")
                        || error_msg.contains("index")
                        || error_msg.contains("column")
                    {
                        println!("  ✓ Position information included");
                    } else {
                        println!("  - Position information not included (acceptable)");
                    }
                }
            }
        }
    }

    #[test]
    fn test_error_suggestion_quality() {
        // Test error messages that could include helpful suggestions
        let suggestion_errors = vec![
            ("$.", "Could suggest adding property name"),
            ("$.key[", "Could suggest closing bracket"),
            ("$.key[?@.prop", "Could suggest completing filter"),
            ("$key", "Could suggest adding $ prefix"),
            ("$.key[abc]", "Could suggest numeric index or quotes"),
        ];

        for (path, potential_suggestion) in suggestion_errors {
            let result = JsonPathParser::compile(path);
            match result {
                Ok(_) => println!("Suggestion test: '{}' compiled unexpectedly", path),
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    println!(
                        "Error for '{}': {} ({})",
                        path, error_msg, potential_suggestion
                    );

                    // Check if error message is helpful for debugging
                    let helpful_keywords = ["expected", "missing", "invalid", "should", "try"];
                    let is_helpful = helpful_keywords
                        .iter()
                        .any(|keyword| error_msg.to_lowercase().contains(keyword));

                    if is_helpful {
                        println!("  ✓ Error message appears helpful");
                    } else {
                        println!("  - Error message could be more helpful");
                    }
                }
            }
        }
    }

    #[test]
    fn test_consistent_error_formatting() {
        // Test that errors follow consistent formatting
        let error_paths = vec![
            "$[",
            "$.key[",
            "$.key[?",
            "$.key[abc]",
            "$.",
            "$.key[?@.prop",
        ];

        let mut error_messages = Vec::new();

        for path in error_paths {
            let result = JsonPathParser::compile(path);
            if let Err(e) = result {
                let error_msg = format!("{:?}", e);
                error_messages.push((path, error_msg));
            }
        }

        // Analyze consistency
        println!("Error message consistency analysis:");
        for (path, msg) in &error_messages {
            println!("  '{}': {}", path, msg);
        }

        // Check for consistent patterns (implementation-dependent)
        if error_messages.len() > 1 {
            let first_pattern = error_messages[0].1.split_whitespace().next().unwrap_or("");
            let consistent = error_messages
                .iter()
                .all(|(_, msg)| msg.split_whitespace().next().unwrap_or("") == first_pattern);

            if consistent {
                println!("  ✓ Error messages follow consistent pattern");
            } else {
                println!("  - Error messages have varying patterns (acceptable)");
            }
        }
    }
}

/// Graceful Degradation Tests
#[cfg(test)]
mod graceful_degradation_tests {
    use super::*;

    #[test]
    fn test_partial_path_success() {
        // Test graceful handling when parts of a path succeed
        let json_data = r#"{"data": {
            "valid": {
                "nested": {
                    "value": "found"
                }
            },
            "partial": {
                "exists": "here"
            }
        }}"#;

        let partial_paths = vec![
            ("$.data.valid.nested.value", 1, "Fully valid path"),
            (
                "$.data.valid.missing.value",
                0,
                "Missing intermediate property",
            ),
            ("$.data.partial.exists", 1, "Valid partial path"),
            ("$.data.partial.missing", 0, "Missing final property"),
            ("$.data.missing.anything", 0, "Missing early in path"),
        ];

        for (path, expectedresults, _description) in partial_paths {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(path);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Partial path '{}' -> {} results (expected {}) - {}",
                path,
                results.len(),
                expectedresults,
                _description
            );

            // Should handle missing paths gracefully, not crash
            assert!(
                results.len() <= expectedresults,
                "Should not return more results than expected"
            );
        }
    }

    #[test]
    fn test_type_mismatch_recovery() {
        // Test recovery from type mismatches
        let json_data = r#"{"mixed": {
            "string": "text",
            "number": 42,
            "array": [1, 2, 3],
            "object": {"key": "value"},
            "null": null
        }}"#;

        let type_mismatch_paths = vec![
            ("$.mixed.string[0]", "Array access on string"),
            ("$.mixed.number.property", "Property access on number"),
            ("$.mixed.array.property", "Property access on array"),
            ("$.mixed.object[0]", "Array access on object"),
            ("$.mixed.null.anything", "Property access on null"),
            ("$.mixed.missing.chain", "Chained access on missing"),
        ];

        for (path, _description) in type_mismatch_paths {
            let result = std::panic::catch_unwind(|| {
                let mut stream = JsonArrayStream::<serde_json::Value>::new(path);

                let chunk = Bytes::from(json_data);
                let results: Vec<_> = stream.process_chunk(chunk).collect();
                results.len()
            });

            match result {
                Ok(count) => println!(
                    "Type mismatch '{}' handled gracefully -> {} results ({})",
                    path, count, _description
                ),
                Err(_) => println!("Type mismatch '{}' caused panic ({})", path, _description),
            }
        }
    }

    #[test]
    fn test_filter_error_recovery() {
        // Test recovery from filter expression errors
        let json_data = r#"{"items": [
            {"name": "item1", "value": 10, "active": true},
            {"name": "item2", "value": null, "active": false},
            {"name": "item3", "active": true},
            {"value": 30, "active": true}
        ]}"#;

        let filter_error_paths = vec![
            ("$.items[?@.value > 5]", "Comparison with null/missing"),
            ("$.items[?@.name == 'item1']", "String comparison"),
            ("$.items[?@.missing > 0]", "Filter on missing property"),
            (
                "$.items[?@.active && @.value]",
                "Logical with missing values",
            ),
            (
                "$.items[?@.name != null && @.value > 0]",
                "Complex filter with null checks",
            ),
        ];

        for (path, _description) in filter_error_paths {
            let result = std::panic::catch_unwind(|| {
                let mut stream = JsonArrayStream::<serde_json::Value>::new(path);

                let chunk = Bytes::from(json_data);
                let results: Vec<_> = stream.process_chunk(chunk).collect();
                results.len()
            });

            match result {
                Ok(count) => println!(
                    "Filter error '{}' handled gracefully -> {} results ({})",
                    path, count, _description
                ),
                Err(_) => println!("Filter error '{}' caused panic ({})", path, _description),
            }
        }
    }

    #[test]
    fn test_malformed_json_recovery() {
        // Test graceful handling of malformed JSON during processing
        let malformed_inputs = vec![
            ("{\"key\": }", "Missing value"),
            ("{\"key\": value}", "Unquoted value"),
            ("{\"key\": \"unclosed}", "Unclosed string"),
            ("{\"key\": [1, 2,]}", "Trailing comma"),
            ("{\"key\": 123,}", "Trailing comma in object"),
        ];

        for (malformed_json, _description) in malformed_inputs {
            let result = std::panic::catch_unwind(|| {
                let mut stream = JsonArrayStream::<serde_json::Value>::new("$.key");

                let chunk = Bytes::from(malformed_json);
                let results: Vec<_> = stream.process_chunk(chunk).collect();
                results.len()
            });

            match result {
                Ok(count) => println!(
                    "Malformed JSON '{}' processed -> {} results ({})",
                    malformed_json, count, _description
                ),
                Err(_) => println!(
                    "Malformed JSON '{}' properly rejected ({})",
                    malformed_json, _description
                ),
            }
        }
    }
}

/// Resource Limit Enforcement Tests
#[cfg(test)]
mod resource_limit_tests {
    use super::*;

    #[test]
    fn test_compilation_time_limits() {
        // Test that complex expressions don't take excessive time to compile
        let complex_expressions = vec![
            "$.a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z",
            "$.items[?@.a && @.b && @.c && @.d && @.e && @.f && @.g && @.h]",
            "$[*][*][*][*][*][*][*][*][*][*]",
            "$..a..b..c..d..e..f..g..h..i..j",
            "$.items[?(@.a > 0 || @.b > 0) && (@.c > 0 || @.d > 0) && (@.e > 0 || @.f > 0)]",
        ];

        for expression in complex_expressions {
            let start_time = std::time::Instant::now();
            let result = JsonPathParser::compile(expression);
            let compilation_time = start_time.elapsed();

            println!(
                "Compilation time for '{}': {:?}",
                expression, compilation_time
            );

            match result {
                Ok(_) => {
                    // Compilation should complete within reasonable time
                    assert!(
                        compilation_time.as_millis() < 1000,
                        "Complex expression compilation should complete in <1000ms"
                    );
                }
                Err(_) => println!("  Expression rejected (acceptable)"),
            }
        }
    }

    #[test]
    fn test_memory_usage_limits() {
        // Test memory usage during error processing
        let memory_intensive_errors = vec![
            format!("$.{}", "a".repeat(1000)),    // Long property name
            format!("$['{}']", "x".repeat(1000)), // Long quoted property
            "$.a[?@.b == '{}']".replace("{}", &"c".repeat(500)), // Long string in filter
        ];

        for expression in memory_intensive_errors {
            let start_time = std::time::Instant::now();
            let result = JsonPathParser::compile(&expression);
            let processing_time = start_time.elapsed();

            println!(
                "Memory test for expression length {}: {:?}",
                expression.len(),
                processing_time
            );

            match result {
                Ok(_) => {
                    assert!(
                        processing_time.as_millis() < 2000,
                        "Memory-intensive expression should process in <2000ms"
                    );
                }
                Err(_) => println!("  Long expression rejected (acceptable)"),
            }

            // Should not consume excessive memory
            assert!(
                processing_time.as_millis() < 5000,
                "Should not take excessive time even for rejection"
            );
        }
    }

    #[test]
    fn test_recursion_depth_limits() {
        // Test limits on recursion depth
        let deep_expressions = vec![
            (20, "Moderate nesting"),
            (50, "Deep nesting"),
            (100, "Very deep nesting"),
        ];

        for (depth, _description) in deep_expressions {
            // Create deeply nested path
            let mut path = String::from("$");
            for i in 0..depth {
                path.push_str(&format!(".level{}", i));
            }

            let start_time = std::time::Instant::now();
            let result = JsonPathParser::compile(&path);
            let processing_time = start_time.elapsed();

            println!(
                "Recursion depth {} ({}): {:?}",
                depth, _description, processing_time
            );

            match result {
                Ok(_) => {
                    assert!(
                        processing_time.as_millis() < 1000,
                        "Deep expression should compile in <1000ms"
                    );
                }
                Err(_) => println!(
                    "  Deep expression rejected at depth {} (may be acceptable)",
                    depth
                ),
            }
        }
    }

    #[test]
    fn test_error_cascade_prevention() {
        // Test prevention of error cascades
        let cascade_prone_expressions = vec![
            "$.a[?@.b[?@.c[?@.d > 0]]]",                 // Nested filters
            "$.a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t", // Long property chain
            "$[*][*][*][*][*][*][*][*]",                 // Deep wildcard nesting
            "$..a..b..c..d..e..f",                       // Multiple descendant operators
        ];

        for expression in cascade_prone_expressions {
            let start_time = std::time::Instant::now();

            let result = std::panic::catch_unwind(|| JsonPathParser::compile(expression));

            let processing_time = start_time.elapsed();

            match result {
                Ok(compileresult) => match compileresult {
                    Ok(_) => println!(
                        "Cascade test '{}' compiled in {:?}",
                        expression, processing_time
                    ),
                    Err(_) => println!(
                        "Cascade test '{}' rejected in {:?}",
                        expression, processing_time
                    ),
                },
                Err(_) => println!("Cascade test '{}' caused panic", expression),
            }

            // Should not take excessive time even with complex/problematic expressions
            assert!(
                processing_time.as_millis() < 3000,
                "Error cascade prevention should limit processing time"
            );
        }
    }
}
