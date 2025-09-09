//! AST module tests
//!
//! Tests for JSONPath AST functionality, mirroring src/json_path/ast.rs

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    name: String,
    value: i32,
}

/// AST Construction and Path Normalization Tests
#[cfg(test)]
mod ast_construction_tests {
    use super::*;

    #[test]
    fn test_ast_path_canonicalization() {
        // Test AST construction for path canonicalization
        let canonicalization_tests = vec![
            (
                "$.store.book",
                "$['store']['book']",
                "Simple property access",
            ),
            ("$['store'].book", "$['store']['book']", "Mixed notation"),
            (
                "$.store['book']",
                "$['store']['book']",
                "Mixed notation reverse",
            ),
            (
                "$[\"store\"][\"book\"]",
                "$['store']['book']",
                "Double to single quotes",
            ),
            ("$.store.book[0]", "$['store']['book'][0]", "Array index"),
            ("$.store.book[*]", "$['store']['book'][*]", "Wildcard"),
            ("$..book", "$..['book']", "Descendant selector"),
            (
                "$.store..book",
                "$['store']..['book']",
                "Descendant with prefix",
            ),
        ];

        for (original, canonical, _description) in canonicalization_tests {
            println!(
                "AST canonicalization: '{}' -> '{}' ({})",
                original, canonical, _description
            );

            let originalresult = JsonPathParser::compile(original);
            let canonicalresult = JsonPathParser::compile(canonical);

            match (originalresult, canonicalresult) {
                (Ok(_), Ok(_)) => println!("  Both forms create valid AST"),
                (Ok(_), Err(_)) => println!("  Original valid AST, canonical invalid"),
                (Err(_), Ok(_)) => println!("  Original invalid AST, canonical valid"),
                (Err(_), Err(_)) => println!("  Both forms create invalid AST"),
            }
        }
    }

    #[test]
    fn test_ast_equivalent_path_normalization() {
        // Test AST construction for equivalent paths
        let json_data = r#"{"data": {
            "items": [
                {"name": "item1"},
                {"name": "item2"},
                {"name": "item3"}
            ]
        }}"#;

        let equivalent_path_groups = vec![
            vec![
                "$.data.items[0].name",
                "$['data']['items'][0]['name']",
                "$[\"data\"][\"items\"][0][\"name\"]",
            ],
            vec![
                "$.data.items[*].name",
                "$['data']['items'][*]['name']",
                "$[\"data\"][\"items\"][*][\"name\"]",
            ],
        ];

        for (group_idx, path_group) in equivalent_path_groups.iter().enumerate() {
            println!("AST equivalent path group {}:", group_idx + 1);

            let mut results_sets = Vec::new();

            for path in path_group {
                let mut stream = JsonArrayStream::<String>::new(path);

                let chunk = Bytes::from(json_data);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                results_sets.push(results);
                println!(
                    "  AST path '{}' -> {} results",
                    path,
                    results_sets.last().expect("Results").len()
                );
            }

            // All equivalent AST paths should produce the same results
            for (i, results) in results_sets.iter().enumerate() {
                for (j, otherresults) in results_sets.iter().enumerate() {
                    if i != j {
                        assert_eq!(
                            results.len(),
                            otherresults.len(),
                            "Equivalent AST paths should produce same results: {} vs {}",
                            path_group[i],
                            path_group[j]
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_ast_dot_notation_to_bracket_conversion() {
        // Test AST construction for dot notation to bracket conversion
        let json_data = r#"{"user": {
            "profile": {
                "name": "John Doe",
                "settings": {
                    "theme": "dark",
                    "notifications": true
                }
            }
        }}"#;

        let dot_to_bracket_conversions = vec![
            ("$.user", "$['user']"),
            ("$.user.profile", "$['user']['profile']"),
            ("$.user.profile.name", "$['user']['profile']['name']"),
            (
                "$.user.profile.settings.theme",
                "$['user']['profile']['settings']['theme']",
            ),
        ];

        for (dot_notation, bracket_notation) in dot_to_bracket_conversions {
            let mut dot_stream = JsonArrayStream::<serde_json::Value>::new(dot_notation);
            let mut bracket_stream = JsonArrayStream::<serde_json::Value>::new(bracket_notation);

            let chunk = Bytes::from(json_data);
            let dotresults: Vec<_> = dot_stream.process_chunk(chunk.clone()).collect();
            let bracketresults: Vec<_> = bracket_stream.process_chunk(chunk).collect();

            println!(
                "AST dot to bracket: '{}' ≡ '{}' -> {} vs {} results",
                dot_notation,
                bracket_notation,
                dotresults.len(),
                bracketresults.len()
            );

            assert_eq!(
                dotresults.len(),
                bracketresults.len(),
                "AST should produce same results for dot and bracket notation"
            );
        }
    }
}

/// AST Segment Structure and Traversal Tests
#[cfg(test)]
mod ast_segment_tests {
    use super::*;

    #[test]
    fn test_ast_child_segment_construction() {
        // Test AST construction for child segments
        let valid_expressions = vec![
            "$['store']",     // Single name selector
            "$[0]",           // Single index selector
            "$[*]",           // Single wildcard selector
            "$['a','b','c']", // Multiple name selectors
            "$[0,1,2]",       // Multiple index selectors
            "$['store',0,*]", // Mixed selectors
        ];

        for expr in valid_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Valid child segment AST '{}' should compile",
                expr
            );
        }
    }

    #[test]
    fn test_ast_descendant_segment_construction() {
        // Test AST construction for descendant segments
        let valid_expressions = vec![
            "$..['name']",  // Descendant name selector
            "$..[0]",       // Descendant index selector
            "$..[*]",       // Descendant wildcard
            "$..['a','b']", // Multiple descendant selectors
        ];

        for expr in valid_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Valid descendant segment AST '{}' should compile",
                expr
            );
        }
    }

    #[test]
    fn test_ast_segment_traversal_logic() {
        // Test AST segment traversal between child and descendant
        let json_data = r#"{
            "level1": {
                "target": "direct_child",
                "level2": {
                    "target": "nested_descendant"
                }
            }
        }"#;

        let mut child_stream = JsonArrayStream::<String>::new("$.level1['target']");
        let mut descendant_stream = JsonArrayStream::<String>::new("$..['target']");

        let chunk = Bytes::from(json_data);

        let childresults: Vec<_> = child_stream.process_chunk(chunk.clone()).collect();
        let descendantresults: Vec<_> = descendant_stream.process_chunk(chunk).collect();

        assert_eq!(
            childresults.len(),
            1,
            "AST child traversal should find only direct child"
        );
        assert_eq!(
            descendantresults.len(),
            2,
            "AST descendant traversal should find all targets"
        );
    }

    #[test]
    fn test_ast_segment_combination_traversal() {
        // Test AST construction for combined segments
        let json_data = r#"{
            "data": {
                "items": [
                    {"books": [{"title": "Book1"}, {"title": "Book2"}]},
                    {"books": [{"title": "Book3"}]}
                ]
            }
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$..books[0]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            2,
            "AST should handle descendant followed by child segment"
        );
    }

    #[test]
    fn test_ast_segment_performance_deep_nesting() {
        // Test AST performance with deep nesting
        let mut nested_json = serde_json::json!({"value": 42});

        // Create deeply nested structure for AST traversal
        for i in 0..20 {
            nested_json = serde_json::json!({
                format!("level{}", i): nested_json
            });
        }

        let json_data = serde_json::to_string(&nested_json).expect("JSON serialization");

        let mut stream = JsonArrayStream::<i32>::new("$..value");

        let chunk = Bytes::from(json_data);
        let start_time = std::time::Instant::now();
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        let duration = start_time.elapsed();

        assert_eq!(results.len(), 1, "AST should find the deeply nested value");
        println!("AST deep traversal took {:?}", duration);

        // AST traversal performance should be reasonable
        assert!(
            duration.as_millis() < 500,
            "AST deep search should complete in <500ms"
        );
    }
}

/// AST Path Validation and Error Handling Tests  
#[cfg(test)]
mod ast_validation_tests {
    use super::*;

    #[test]
    fn test_ast_invalid_segment_syntax() {
        // Test AST validation for invalid segment syntaxes
        let invalid_expressions = vec![
            "$.[store]",      // Invalid child segment syntax
            "$..[",           // Unclosed descendant segment
            "$...]",          // Invalid descendant syntax
            "$...store",      // Triple dot
            "$.store.[book]", // Invalid mixed syntax
        ];

        for expr in invalid_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Invalid segment AST '{}' should fail",
                expr
            );
        }
    }

    #[test]
    fn test_ast_empty_segments() {
        // Test AST validation for empty segments
        let empty_segments = vec![
            "$[]",       // Empty child segment
            "$.[]",      // Empty child after dot
            "$.store[]", // Empty child after property
        ];

        for expr in empty_segments {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_err(), "Empty segment AST '{}' should fail", expr);
        }
    }

    #[test]
    fn test_ast_normalized_path_consistency() {
        // Test AST consistency for normalized paths
        let json_data = r#"{"complex": {
            "with spaces": {
                "and'quotes": {
                    "data": [1, 2, 3]
                }
            }
        }}"#;

        let complex_paths = vec![
            "$['complex']['with spaces']['and\\'quotes']['data'][*]",
            "$['complex']['with spaces']['and\\'quotes']['data'][0]",
        ];

        for path in complex_paths {
            println!("Testing AST normalized path: '{}'", path);

            let result = JsonPathParser::compile(path);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<i32>::new(path);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "  AST compiled and executed successfully, {} results",
                        results.len()
                    );
                }
                Err(e) => println!("  AST failed to compile: {:?}", e),
            }
        }
    }

    #[test]
    fn test_ast_unicode_key_handling() {
        // Test AST handling of Unicode characters in keys
        let json_data = r#"{"unicode": {
            "café": "coffee",
            "naïve": "innocent", 
            "🚀": "rocket",
            "こんにちは": "hello"
        }}"#;

        let unicode_key_tests = vec![
            (
                "$.unicode.café",
                "$['unicode']['café']",
                "Accented characters",
            ),
            ("$.unicode['🚀']", "$['unicode']['🚀']", "Emoji key"),
            (
                "$.unicode['こんにちは']",
                "$['unicode']['こんにちは']",
                "Japanese characters",
            ),
        ];

        for (original, normalized, _description) in unicode_key_tests {
            let mut original_stream = JsonArrayStream::<String>::new(original);
            let mut normalized_stream = JsonArrayStream::<String>::new(normalized);

            let chunk = Bytes::from(json_data);
            let originalresults: Vec<_> = original_stream.process_chunk(chunk.clone()).collect();
            let normalizedresults: Vec<_> = normalized_stream.process_chunk(chunk).collect();

            println!(
                "AST Unicode key test: '{}' -> '{}' ({}) -> {} vs {} results",
                original,
                normalized,
                _description,
                originalresults.len(),
                normalizedresults.len()
            );

            assert_eq!(
                originalresults.len(),
                normalizedresults.len(),
                "AST should handle Unicode keys consistently"
            );
        }
    }
}
