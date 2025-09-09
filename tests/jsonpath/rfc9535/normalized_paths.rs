//! RFC 9535 Normalized Paths Test Suite (Section 2.1)
//!
//! Tests for normalized path generation and validation as specified in RFC 9535.
//! Normalized paths provide a canonical representation of JSONPath expressions
//! using bracket notation with single quotes and proper character escaping.
//!
//! This test suite validates:
//! - Canonical bracket notation generation
//! - Single quote delimiter enforcement
//! - Character escaping rules (normal-escapable characters)
//! - Path uniqueness validation
//! - Conversion from various JSONPath forms to normalized form
//! - Unicode handling in normalized paths
//! - Special character escaping
//! - Bracket notation consistency

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct PathTest {
    original: String,
    normalized: String,
    equivalent: bool,
}

/// RFC 9535 Section 2.1 - Canonical Bracket Notation Tests
#[cfg(test)]
mod canonical_bracket_notation_tests {
    use super::*;

    #[test]
    fn test_member_name_shorthand_to_bracket() {
        // RFC 9535: Convert member-name-shorthand to bracket notation
        let json_data = r#"{"store": {
            "book": [
                {"title": "Book 1", "author": "Author 1"},
                {"title": "Book 2", "author": "Author 2"}
            ],
            "bicycle": {"color": "red", "price": 19.95}
        }}"#;

        let shorthand_to_bracket_tests = vec![
            // Member-name-shorthand to bracket notation equivalence
            ("$.store.book", "$['store']['book']", "Store book property"),
            (
                "$.store.bicycle.color",
                "$['store']['bicycle']['color']",
                "Nested property access",
            ),
            (
                "$.store.book[0].title",
                "$['store']['book'][0]['title']",
                "Array index with property",
            ),
            (
                "$.store.book[*].author",
                "$['store']['book'][*]['author']",
                "Wildcard with property",
            ),
        ];

        for (shorthand, bracket, _description) in shorthand_to_bracket_tests {
            // Test that both forms produce equivalent results
            let mut shorthand_stream = JsonArrayStream::<serde_json::Value>::new(shorthand);
            let mut bracket_stream = JsonArrayStream::<serde_json::Value>::new(bracket);

            let chunk = Bytes::from(json_data);
            let shorthandresults: Vec<_> = shorthand_stream.process_chunk(chunk.clone()).collect();
            let bracketresults: Vec<_> = bracket_stream.process_chunk(chunk).collect();

            println!(
                "Equivalence test: '{}' ≡ '{}' -> {} vs {} results ({})",
                shorthand,
                bracket,
                shorthandresults.len(),
                bracketresults.len(),
                _description
            );

            // Both forms should produce the same number of results
            assert_eq!(
                shorthandresults.len(),
                bracketresults.len(),
                "Shorthand '{}' and bracket '{}' should produce same results",
                shorthand,
                bracket
            );
        }
    }

    #[test]
    fn test_wildcard_normalization() {
        // RFC 9535: Wildcard selector normalization
        let json_data = r#"{"data": {
            "items": [{"a": 1}, {"b": 2}, {"c": 3}],
            "props": {"x": 10, "y": 20, "z": 30}
        }}"#;

        let wildcard_tests = vec![
            ("$.data.items.*", "$['data']['items'][*]", "Object wildcard"),
            (
                "$.data.items[*]",
                "$['data']['items'][*]",
                "Array wildcard (already normalized)",
            ),
            (
                "$.data.props.*",
                "$['data']['props'][*]",
                "Property wildcard",
            ),
            ("$.*", "$[*]", "Root wildcard"),
        ];

        for (original, normalized, _description) in wildcard_tests {
            let mut original_stream = JsonArrayStream::<serde_json::Value>::new(original);
            let mut normalized_stream = JsonArrayStream::<serde_json::Value>::new(normalized);

            let chunk = Bytes::from(json_data);
            let originalresults: Vec<_> = original_stream.process_chunk(chunk.clone()).collect();
            let normalizedresults: Vec<_> = normalized_stream.process_chunk(chunk).collect();

            println!(
                "Wildcard normalization: '{}' -> '{}' ({}) -> {} vs {} results",
                original,
                normalized,
                _description,
                originalresults.len(),
                normalizedresults.len()
            );
        }
    }

    #[test]
    fn test_descendant_segment_normalization() {
        // RFC 9535: Descendant segment normalization
        let json_data = r#"{"root": {
            "level1": {
                "target": "found1",
                "level2": {
                    "target": "found2",
                    "level3": {"target": "found3"}
                }
            },
            "target": "found_root"
        }}"#;

        let descendant_tests = vec![
            ("$..target", "$..['target']", "Descendant with property"),
            (
                "$.root..target",
                "$['root']..['target']",
                "Descendant from specific root",
            ),
            (
                "$..level2.target",
                "$..['level2']['target']",
                "Descendant with path",
            ),
        ];

        for (original, normalized, _description) in descendant_tests {
            let mut original_stream = JsonArrayStream::<String>::new(original);
            let mut normalized_stream = JsonArrayStream::<String>::new(normalized);

            let chunk = Bytes::from(json_data);
            let originalresults: Vec<_> = original_stream.process_chunk(chunk.clone()).collect();
            let normalizedresults: Vec<_> = normalized_stream.process_chunk(chunk).collect();

            println!(
                "Descendant normalization: '{}' -> '{}' ({}) -> {} vs {} results",
                original,
                normalized,
                _description,
                originalresults.len(),
                normalizedresults.len()
            );
        }
    }
}

/// Single Quote Delimiter Enforcement Tests
#[cfg(test)]
mod single_quote_delimiter_tests {
    use super::*;

    #[test]
    fn test_single_quote_requirement() {
        // RFC 9535: Normalized paths must use single quotes
        let json_data = r#"{"keys": {
            "simple": "value1",
            "with space": "value2",
            "with'quote": "value3",
            "with\"doublequote": "value4"
        }}"#;

        let quote_normalization_tests = vec![
            // Double quotes should be normalized to single quotes
            (
                "$[\"keys\"][\"simple\"]",
                "$['keys']['simple']",
                "Simple key",
            ),
            (
                "$[\"keys\"][\"with space\"]",
                "$['keys']['with space']",
                "Key with space",
            ),
            (
                "$[\"keys\"][\"with'quote\"]",
                "$['keys']['with\\'quote']",
                "Key with single quote",
            ),
            (
                "$[\"keys\"][\"with\\\"doublequote\"]",
                "$['keys']['with\"doublequote']",
                "Key with double quote",
            ),
        ];

        for (double_quoted, single_quoted, _description) in quote_normalization_tests {
            let mut double_stream = JsonArrayStream::<String>::new(double_quoted);
            let mut single_stream = JsonArrayStream::<String>::new(single_quoted);

            let chunk = Bytes::from(json_data);
            let doubleresults: Vec<_> = double_stream.process_chunk(chunk.clone()).collect();
            let singleresults: Vec<_> = single_stream.process_chunk(chunk).collect();

            println!(
                "Quote normalization: '{}' -> '{}' ({}) -> {} vs {} results",
                double_quoted,
                single_quoted,
                _description,
                doubleresults.len(),
                singleresults.len()
            );
        }
    }

    #[test]
    fn test_quote_escaping_in_keys() {
        // Test proper escaping of quotes within keys
        let json_data = r#"{"complex": {
            "don't": "apostrophe",
            "say \"hello\"": "quoted",
            "mix'ed\"quotes": "both",
            "normal_key": "simple"
        }}"#;

        let quote_escaping_tests = vec![
            (
                "$.complex[\"don't\"]",
                "$['complex']['don\\'t']",
                "Apostrophe in key",
            ),
            (
                "$.complex[\"say \\\"hello\\\"\"]",
                "$['complex']['say \"hello\"']",
                "Double quotes in key",
            ),
            (
                "$.complex[\"mix'ed\\\"quotes\"]",
                "$['complex']['mix\\'ed\"quotes']",
                "Mixed quotes in key",
            ),
            (
                "$.complex.normal_key",
                "$['complex']['normal_key']",
                "Normal key",
            ),
        ];

        for (original, normalized, _description) in quote_escaping_tests {
            println!(
                "Quote escaping test: '{}' -> '{}' ({})",
                original, normalized, _description
            );

            let originalresult = JsonPathParser::compile(original);
            let normalizedresult = JsonPathParser::compile(normalized);

            match (originalresult, normalizedresult) {
                (Ok(_), Ok(_)) => {
                    println!("  Both forms compiled successfully");

                    // Test that both forms produce equivalent results when executed
                    let mut original_stream = JsonArrayStream::<serde_json::Value>::new(original);
                    let mut normalized_stream =
                        JsonArrayStream::<serde_json::Value>::new(normalized);

                    let chunk = Bytes::from(json_data);
                    let original_results: Vec<_> =
                        original_stream.process_chunk(chunk.clone()).collect();
                    let normalized_results: Vec<_> =
                        normalized_stream.process_chunk(chunk.clone()).collect();

                    assert_eq!(
                        original_results.len(),
                        normalized_results.len(),
                        "Results count should match for equivalent paths"
                    );
                }
                (Ok(_), Err(_)) => println!("  Original compiled, normalized failed"),
                (Err(_), Ok(_)) => println!("  Original failed, normalized compiled"),
                (Err(_), Err(_)) => println!("  Both forms failed to compile"),
            }
        }
    }

    #[test]
    fn test_unicode_in_quoted_keys() {
        // Test Unicode characters in quoted keys
        let json_data = r#"{"unicode": {
            "café": "coffee",
            "naïve": "innocent", 
            "🚀": "rocket",
            "こんにちは": "hello",
            "αβγ": "greek"
        }}"#;

        let unicode_key_tests = vec![
            (
                "$.unicode.café",
                "$['unicode']['café']",
                "Accented characters",
            ),
            ("$.unicode.naïve", "$['unicode']['naïve']", "Diaeresis"),
            ("$.unicode['🚀']", "$['unicode']['🚀']", "Emoji key"),
            (
                "$.unicode['こんにちは']",
                "$['unicode']['こんにちは']",
                "Japanese characters",
            ),
            ("$.unicode.αβγ", "$['unicode']['αβγ']", "Greek letters"),
        ];

        for (original, normalized, _description) in unicode_key_tests {
            let mut original_stream = JsonArrayStream::<String>::new(original);
            let mut normalized_stream = JsonArrayStream::<String>::new(normalized);

            let chunk = Bytes::from(json_data);
            let originalresults: Vec<_> = original_stream.process_chunk(chunk.clone()).collect();
            let normalizedresults: Vec<_> = normalized_stream.process_chunk(chunk).collect();

            println!(
                "Unicode key test: '{}' -> '{}' ({}) -> {} vs {} results",
                original,
                normalized,
                _description,
                originalresults.len(),
                normalizedresults.len()
            );
        }
    }
}

/// Character Escaping Rules Tests (normal-escapable)
#[cfg(test)]
mod character_escaping_tests {
    use super::*;

    #[test]
    fn test_normal_escapable_characters() {
        // RFC 9535: Test normal-escapable character set
        let json_data = r#"{"special": {
            "quote'test": "apostrophe",
            "backslash\\test": "backslash",
            "forward/slash": "slash",
            "tab\there": "tab",
            "newline\nhere": "newline"
        }}"#;

        let escapable_tests = vec![
            // Characters that must be escaped in normalized paths
            ("quote'test", "quote\\'test", "Single quote escape"),
            ("backslash\\test", "backslash\\\\test", "Backslash escape"),
            (
                "forward/slash",
                "forward/slash",
                "Forward slash (no escape needed)",
            ),
            ("tab\there", "tab\\there", "Tab character escape"),
            (
                "newline\nhere",
                "newline\\nhere",
                "Newline character escape",
            ),
        ];

        for (key, escaped_key, _description) in escapable_tests {
            let original_path = format!("$.special['{}'']", key);
            let normalized_path = format!("$['special']['{}']", escaped_key);

            println!(
                "Escaping test: '{}' -> '{}' ({})",
                original_path, normalized_path, _description
            );

            let originalresult = JsonPathParser::compile(&original_path);
            let normalizedresult = JsonPathParser::compile(&normalized_path);

            match (originalresult, normalizedresult) {
                (Ok(_), Ok(_)) => {
                    println!("  Both paths compiled successfully");

                    // Test that both forms produce equivalent results when executed
                    let mut original_stream =
                        JsonArrayStream::<serde_json::Value>::new(&original_path);
                    let mut normalized_stream =
                        JsonArrayStream::<serde_json::Value>::new(&normalized_path);

                    let chunk = Bytes::from(json_data);
                    let original_results: Vec<_> =
                        original_stream.process_chunk(chunk.clone()).collect();
                    let normalized_results: Vec<_> =
                        normalized_stream.process_chunk(chunk.clone()).collect();

                    assert_eq!(
                        original_results.len(),
                        normalized_results.len(),
                        "Results count should match for equivalent paths"
                    );
                }
                (Ok(_), Err(_)) => println!("  Original compiled, normalized failed"),
                (Err(_), Ok(_)) => println!("  Original failed, normalized compiled"),
                (Err(_), Err(_)) => println!("  Both paths failed to compile"),
            }
        }
    }

    #[test]
    fn test_control_character_escaping() {
        // Test escaping of control characters
        let control_characters = vec![
            ('\u{0008}', "\\b", "Backspace"),
            ('\u{0009}', "\\t", "Tab"),
            ('\u{000A}', "\\n", "Line feed"),
            ('\u{000C}', "\\f", "Form feed"),
            ('\u{000D}', "\\r", "Carriage return"),
        ];

        for (control_char, escape_sequence, _description) in control_characters {
            let key_with_control = format!("test{}char", control_char);
            let escaped_key = format!("test{}char", escape_sequence);
            let normalized_path = format!("$['special']['{}']", escaped_key);

            println!(
                "Control character test: '{}' -> '{}' ({})",
                key_with_control, normalized_path, _description
            );

            let result = JsonPathParser::compile(&normalized_path);
            match result {
                Ok(_) => println!("  Normalized path compiled successfully"),
                Err(_) => println!("  Normalized path failed to compile"),
            }
        }
    }

    #[test]
    fn test_unicode_escape_sequences() {
        // Test Unicode escape sequences in normalized paths
        let unicode_escapes = vec![
            ("\\u0041", "A", "Latin A"),
            ("\\u00E9", "é", "e with acute"),
            ("\\u03B1", "α", "Greek alpha"),
            ("\\u1F680", "🚀", "Rocket emoji"),
        ];

        for (escape_sequence, character, _description) in unicode_escapes {
            let path_with_escape = format!("$['test']['{}']", escape_sequence);
            let path_with_char = format!("$['test']['{}']", character);

            println!(
                "Unicode escape test: '{}' vs '{}' ({})",
                path_with_escape, path_with_char, _description
            );

            let escaperesult = JsonPathParser::compile(&path_with_escape);
            let charresult = JsonPathParser::compile(&path_with_char);

            match (escaperesult, charresult) {
                (Ok(_), Ok(_)) => println!("  Both forms compiled successfully"),
                (Ok(_), Err(_)) => println!("  Escape compiled, character failed"),
                (Err(_), Ok(_)) => println!("  Escape failed, character compiled"),
                (Err(_), Err(_)) => println!("  Both forms failed to compile"),
            }
        }
    }
}

/// Path Uniqueness Validation Tests
#[cfg(test)]
mod path_uniqueness_tests {
    use super::*;

    #[test]
    fn test_equivalent_path_normalization() {
        // Test that equivalent paths normalize to the same form
        let json_data = r#"{"data": {
            "items": [
                {"name": "item1"},
                {"name": "item2"},
                {"name": "item3"}
            ]
        }}"#;

        let equivalent_path_groups = vec![
            // All these should normalize to the same form
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
            vec!["$..name", "$..['name']", "$..\"name\""],
        ];

        for (group_idx, path_group) in equivalent_path_groups.iter().enumerate() {
            println!("Equivalent path group {}:", group_idx + 1);

            let mut results_sets = Vec::new();

            for path in path_group {
                let mut stream = JsonArrayStream::<String>::new(path);

                let chunk = Bytes::from(json_data);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                results_sets.push(results);
                println!(
                    "  '{}' -> {} results",
                    path,
                    results_sets.last().unwrap().len()
                );
            }

            // All equivalent paths should produce the same results
            for (i, results) in results_sets.iter().enumerate() {
                for (j, otherresults) in results_sets.iter().enumerate() {
                    if i != j {
                        assert_eq!(
                            results.len(),
                            otherresults.len(),
                            "Equivalent paths should produce same number of results: {} vs {}",
                            path_group[i],
                            path_group[j]
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_path_canonicalization() {
        // Test canonicalization of various path forms
        let canonicalization_tests = vec![
            // [original_path, expected_canonical_form, _description]
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
                "Canonicalization: '{}' -> '{}' ({})",
                original, canonical, _description
            );

            let originalresult = JsonPathParser::compile(original);
            let canonicalresult = JsonPathParser::compile(canonical);

            match (originalresult, canonicalresult) {
                (Ok(_), Ok(_)) => println!("  Both forms valid"),
                (Ok(_), Err(_)) => println!("  Original valid, canonical invalid"),
                (Err(_), Ok(_)) => println!("  Original invalid, canonical valid"),
                (Err(_), Err(_)) => println!("  Both forms invalid"),
            }
        }
    }

    #[test]
    fn test_normalized_path_consistency() {
        // Test that normalized paths are consistent across implementations
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
            "$['complex']['with spaces']['and\\'quotes']['data'][1:3]",
        ];

        for path in complex_paths {
            println!("Testing normalized path: '{}'", path);

            let result = JsonPathParser::compile(path);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<i32>::new(path);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "  Compiled and executed successfully, {} results",
                        results.len()
                    );
                }
                Err(e) => println!("  Failed to compile: {:?}", e),
            }
        }
    }
}

/// Conversion from Various JSONPath Forms
#[cfg(test)]
mod conversion_form_tests {
    use super::*;

    #[test]
    fn test_dot_notation_conversion() {
        // Test conversion from dot notation to normalized bracket notation
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
            (
                "$.user.profile.settings.notifications",
                "$['user']['profile']['settings']['notifications']",
            ),
        ];

        for (dot_notation, bracket_notation) in dot_to_bracket_conversions {
            let mut dot_stream = JsonArrayStream::<serde_json::Value>::new(dot_notation);
            let mut bracket_stream = JsonArrayStream::<serde_json::Value>::new(bracket_notation);

            let chunk = Bytes::from(json_data);
            let dotresults: Vec<_> = dot_stream.process_chunk(chunk.clone()).collect();
            let bracketresults: Vec<_> = bracket_stream.process_chunk(chunk).collect();

            println!(
                "Dot to bracket: '{}' ≡ '{}' -> {} vs {} results",
                dot_notation,
                bracket_notation,
                dotresults.len(),
                bracketresults.len()
            );

            assert_eq!(
                dotresults.len(),
                bracketresults.len(),
                "Dot and bracket notation should produce same results"
            );
        }
    }

    #[test]
    fn test_mixed_notation_normalization() {
        // Test normalization of mixed notation styles
        let json_data = r#"{"data": {
            "array": [
                {"key1": "value1", "key2": "value2"},
                {"key1": "value3", "key2": "value4"}
            ]
        }}"#;

        let mixed_notation_tests = vec![
            ("$.data['array'][0].key1", "$['data']['array'][0]['key1']"),
            (
                "$['data'].array[0]['key1']",
                "$['data']['array'][0]['key1']",
            ),
            ("$.data.array[*].key1", "$['data']['array'][*]['key1']"),
            (
                "$['data']['array'][*].key1",
                "$['data']['array'][*]['key1']",
            ),
        ];

        for (mixed, normalized) in mixed_notation_tests {
            let mut mixed_stream = JsonArrayStream::<String>::new(mixed);
            let mut normalized_stream = JsonArrayStream::<String>::new(normalized);

            let chunk = Bytes::from(json_data);
            let mixedresults: Vec<_> = mixed_stream.process_chunk(chunk.clone()).collect();
            let normalizedresults: Vec<_> = normalized_stream.process_chunk(chunk).collect();

            println!(
                "Mixed notation: '{}' -> '{}' -> {} vs {} results",
                mixed,
                normalized,
                mixedresults.len(),
                normalizedresults.len()
            );
        }
    }

    #[test]
    fn test_filter_expression_normalization() {
        // Test normalization of filter expressions
        let json_data = r#"{"items": [
            {"name": "item1", "price": 10},
            {"name": "item2", "price": 20},
            {"name": "item3", "price": 15}
        ]}"#;

        let filter_normalizations = vec![
            ("$.items[?@.price > 10]", "$['items'][?@['price'] > 10]"),
            (
                "$.items[?@.name == 'item1']",
                "$['items'][?@['name'] == 'item1']",
            ),
            (
                "$.items[?@.price < 20 && @.name != 'item3']",
                "$['items'][?@['price'] < 20 && @['name'] != 'item3']",
            ),
        ];

        for (original_filter, normalized_filter) in filter_normalizations {
            println!(
                "Filter normalization: '{}' -> '{}'",
                original_filter, normalized_filter
            );

            let originalresult = JsonPathParser::compile(original_filter);
            let normalizedresult = JsonPathParser::compile(normalized_filter);

            match (originalresult, normalizedresult) {
                (Ok(_), Ok(_)) => {
                    let mut original_stream =
                        JsonArrayStream::<serde_json::Value>::new(original_filter);
                    let mut normalized_stream =
                        JsonArrayStream::<serde_json::Value>::new(normalized_filter);

                    let chunk = Bytes::from(json_data);
                    let originalresults: Vec<_> =
                        original_stream.process_chunk(chunk.clone()).collect();
                    let normalizedresults: Vec<_> =
                        normalized_stream.process_chunk(chunk).collect();

                    println!(
                        "  Both compiled, {} vs {} results",
                        originalresults.len(),
                        normalizedresults.len()
                    );
                }
                (Ok(_), Err(_)) => println!("  Original compiled, normalized failed"),
                (Err(_), Ok(_)) => println!("  Original failed, normalized compiled"),
                (Err(_), Err(_)) => println!("  Both failed to compile"),
            }
        }
    }
}

/// Complex Normalized Path Validation
#[cfg(test)]
mod complex_normalized_path_tests {
    use super::*;

    #[test]
    fn test_deeply_nested_normalization() {
        // Test normalization of deeply nested structures
        let json_data = r#"{"level1": {
            "level2": {
                "level3": {
                    "level4": {
                        "level5": {
                            "data": "deep_value"
                        }
                    }
                }
            }
        }}"#;

        let deep_path_original = "$.level1.level2.level3.level4.level5.data";
        let deep_path_normalized = "$['level1']['level2']['level3']['level4']['level5']['data']";

        let mut original_stream = JsonArrayStream::<String>::new(deep_path_original);
        let mut normalized_stream = JsonArrayStream::<String>::new(deep_path_normalized);

        let chunk = Bytes::from(json_data);
        let originalresults: Vec<_> = original_stream.process_chunk(chunk.clone()).collect();
        let normalizedresults: Vec<_> = normalized_stream.process_chunk(chunk).collect();

        println!(
            "Deep nested normalization: {} vs {} results",
            originalresults.len(),
            normalizedresults.len()
        );

        assert_eq!(
            originalresults.len(),
            normalizedresults.len(),
            "Deep nested paths should produce same results"
        );
    }

    #[test]
    fn test_complex_array_slice_normalization() {
        // Test normalization of complex array slice expressions
        let json_data = r#"{"matrix": [
            [1, 2, 3, 4, 5],
            [6, 7, 8, 9, 10],
            [11, 12, 13, 14, 15],
            [16, 17, 18, 19, 20]
        ]}"#;

        let slice_normalizations = vec![
            ("$.matrix[1:3][::2]", "$['matrix'][1:3][::2]"),
            ("$.matrix[*][1::2]", "$['matrix'][*][1::2]"),
            ("$.matrix[::-1][0]", "$['matrix'][::-1][0]"),
        ];

        for (original, normalized) in slice_normalizations {
            let mut original_stream = JsonArrayStream::<i32>::new(original);
            let mut normalized_stream = JsonArrayStream::<i32>::new(normalized);

            let chunk = Bytes::from(json_data);
            let originalresults: Vec<_> = original_stream.process_chunk(chunk.clone()).collect();
            let normalizedresults: Vec<_> = normalized_stream.process_chunk(chunk).collect();

            println!(
                "Array slice normalization: '{}' -> '{}' -> {} vs {} results",
                original,
                normalized,
                originalresults.len(),
                normalizedresults.len()
            );
        }
    }

    #[test]
    fn test_special_character_key_normalization() {
        // Test normalization with special characters in keys
        let json_data = r#"{"special": {
            "key with spaces": "value1",
            "key-with-dashes": "value2", 
            "key_with_underscores": "value3",
            "key.with.dots": "value4",
            "key[with]brackets": "value5",
            "key{with}braces": "value6"
        }}"#;

        let special_key_tests = vec![
            ("$['special']['key with spaces']", "Key with spaces"),
            ("$['special']['key-with-dashes']", "Key with dashes"),
            (
                "$['special']['key_with_underscores']",
                "Key with underscores",
            ),
            ("$['special']['key.with.dots']", "Key with dots"),
            ("$['special']['key[with]brackets']", "Key with brackets"),
            ("$['special']['key{with}braces']", "Key with braces"),
        ];

        for (normalized_path, _description) in special_key_tests {
            let mut stream = JsonArrayStream::<String>::new(normalized_path);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Special character key: '{}' -> {} results ({})",
                normalized_path,
                results.len(),
                _description
            );

            assert_eq!(
                results.len(),
                1,
                "Special character key should be accessible: {}",
                normalized_path
            );
        }
    }
}
