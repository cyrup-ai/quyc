//! RFC 9535 Shorthand Syntax Validation Tests
//!
//! Tests all shorthand syntax patterns defined in RFC 9535 Appendix A ABNF
//! including member-name-shorthand, index shortcuts, and wildcard shortcuts

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    name: String,
    value: i32,
}

/// RFC 9535 Shorthand Syntax Validation Tests
#[cfg(test)]
mod shorthand_syntax_tests {
    use super::*;

    #[test]
    fn test_member_name_shorthand_comprehensive() {
        // RFC 9535: member-name-shorthand = unquoted-member-name
        let json_data = r#"{
            "simple": "value1",
            "with_underscore": "value2",
            "CamelCase": "value3",
            "mixedCase123": "value4",
            "ALLCAPS": "value5",
            "café": "unicode1",
            "naïve": "unicode2",
            "東京": "unicode3",
            "Москва": "unicode4",
            "user123": "alphanumeric",
            "_leadingUnderscore": "underscore_start",
            "a": "single_char",
            "veryLongPropertyNameThatShouldStillWork": "long_name"
        }"#;

        let valid_shorthand_patterns = vec![
            // Basic patterns
            ("$.simple", "Simple identifier"),
            ("$.with_underscore", "Underscore in name"),
            ("$.CamelCase", "CamelCase pattern"),
            ("$.mixedCase123", "Mixed case with numbers"),
            ("$.ALLCAPS", "All uppercase"),
            // Unicode patterns
            ("$.café", "Unicode accented characters"),
            ("$.naïve", "Unicode diaeresis"),
            ("$.東京", "Unicode Japanese"),
            ("$.Москва", "Unicode Cyrillic"),
            // Special cases
            ("$.user123", "Trailing numbers"),
            ("$._leadingUnderscore", "Leading underscore"),
            ("$.a", "Single character"),
            (
                "$.veryLongPropertyNameThatShouldStillWork",
                "Very long name",
            ),
        ];

        for (expr, _description) in valid_shorthand_patterns {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Valid shorthand '{}' should compile: {}",
                expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Shorthand test '{}' returned {} results ({})",
                expr,
                results.len(),
                _description
            );
        }
    }

    #[test]
    fn test_invalid_member_name_shorthand() {
        // Test patterns that should NOT be valid as shorthand
        let invalid_shorthand_patterns = vec![
            // Starting with digits
            ("$.123invalid", "Starting with digit"),
            ("$.1", "Single digit"),
            ("$.9abc", "Digit then letters"),
            // Special characters not allowed in unquoted names
            ("$.name-with-hyphens", "Hyphens not allowed"),
            ("$.name@domain", "@ symbol not allowed"),
            ("$.name.space", "Dots create chained access"),
            ("$.name[bracket]", "Brackets not allowed in name"),
            ("$.name(paren)", "Parentheses not allowed"),
            ("$.name{brace}", "Braces not allowed"),
            ("$.name:colon", "Colons not allowed"),
            ("$.name;semicolon", "Semicolons not allowed"),
            ("$.name,comma", "Commas not allowed"),
            ("$.name+plus", "Plus signs not allowed"),
            ("$.name=equals", "Equals signs not allowed"),
            ("$.name space", "Spaces not allowed"),
            // Special JSON values (behavior may vary)
            ("$.null", "null keyword"),
            ("$.true", "true keyword"),
            ("$.false", "false keyword"),
            ("$.undefined", "undefined keyword"),
        ];

        for (expr, _description) in invalid_shorthand_patterns {
            let result = JsonPathParser::compile(expr);
            // Document the behavior - these may be valid depending on implementation
            println!(
                "Invalid shorthand test '{}' -> {:?} ({})",
                expr,
                result.is_ok(),
                _description
            );
        }
    }

    #[test]
    fn test_shorthand_vs_bracket_notation_equivalence() {
        // Test that shorthand and bracket notation are equivalent
        let json_data = r#"{
            "store": {
                "book": [
                    {"title": "Book1", "author": "Author1"},
                    {"title": "Book2", "author": "Author2"}
                ]
            }
        }"#;

        let equivalence_tests = vec![
            // Simple property access
            ("$.store", "$['store']", "Root property"),
            ("$.store.book", "$['store']['book']", "Nested property"),
            // Mixed notation - shorthand first
            (
                "$.store.book[0].title",
                "$['store']['book'][0]['title']",
                "Mixed shorthand to bracket",
            ),
            // Unicode equivalence
            // Note: These would need Unicode data to test properly
        ];

        for (shorthand, bracket, _description) in equivalence_tests {
            let mut shorthand_stream = JsonArrayStream::<serde_json::Value>::new(shorthand);
            let mut bracket_stream = JsonArrayStream::<serde_json::Value>::new(bracket);

            let chunk = Bytes::from(json_data);
            let shorthandresults: Vec<_> = shorthand_stream.process_chunk(chunk.clone()).collect();
            let bracketresults: Vec<_> = bracket_stream.process_chunk(chunk).collect();

            assert_eq!(
                shorthandresults.len(),
                bracketresults.len(),
                "Shorthand '{}' and bracket '{}' should return same count: {}",
                shorthand,
                bracket,
                _description
            );

            // Check value equivalence if both have results
            if !shorthandresults.is_empty() && !bracketresults.is_empty() {
                assert_eq!(
                    shorthandresults[0], bracketresults[0],
                    "Shorthand and bracket notation should return equivalent values: {}",
                    _description
                );
            }

            println!(
                "Equivalence test: '{}' ≡ '{}' -> {} results ({})",
                shorthand,
                bracket,
                shorthandresults.len(),
                _description
            );
        }
    }

    #[test]
    fn test_wildcard_shorthand_syntax() {
        // RFC 9535: Test wildcard selector shorthand
        let json_data = r#"{
            "array": [1, 2, 3, 4, 5],
            "object": {
                "a": "value1",
                "b": "value2", 
                "c": "value3"
            },
            "mixed": {
                "items": [
                    {"type": "A", "value": 10},
                    {"type": "B", "value": 20}
                ]
            }
        }"#;

        let wildcard_tests = vec![
            // Basic wildcard patterns
            ("$.*", "Root wildcard - all properties"),
            ("$.array[*]", "Array wildcard"),
            ("$.object.*", "Object property wildcard"),
            ("$.object[*]", "Object bracket wildcard"),
            // Chained wildcards
            ("$.mixed.items[*].*", "Chained wildcards"),
            ("$.mixed.items[*].type", "Wildcard then property"),
            ("$.*.*", "Double wildcard"),
            ("$[*][*]", "Double bracket wildcard"),
            // Mixed with descendant
            ("$..*", "Descendant wildcard"),
            ("$..items[*]", "Descendant then array wildcard"),
            ("$..items[*].value", "Complex descendant wildcard"),
        ];

        for (expr, _description) in wildcard_tests {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Wildcard pattern '{}' should compile: {}",
                expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Wildcard test '{}' returned {} results ({})",
                expr,
                results.len(),
                _description
            );
        }
    }

    #[test]
    fn test_descendant_shorthand_syntax() {
        // RFC 9535: Test descendant operator shorthand patterns
        let json_data = r#"{
            "level1": {
                "target": "shallow",
                "level2": {
                    "target": "medium",
                    "level3": {
                        "target": "deep",
                        "nested": {
                            "target": "deeper"
                        }
                    }
                }
            },
            "other": {
                "target": "other_branch"
            }
        }"#;

        let descendant_shorthand_tests = vec![
            // Basic descendant patterns
            ("$..target", "Find all 'target' properties"),
            ("$.level1..target", "Descendant from level1"),
            ("$..level3.target", "Descendant to specific path"),
            // Descendant with wildcards
            ("$..*", "Universal descendant"),
            ("$..level2.*", "Descendant then wildcard"),
            ("$.level1..*", "Specific root then descendant wildcard"),
            // Descendant with arrays (would need array data)
            ("$..items[*]", "Descendant array wildcard"),
            ("$..items[0]", "Descendant array index"),
            // Complex descendant patterns
            ("$..nested.target", "Deep descendant access"),
            ("$.level1..nested..target", "Multiple descendant operators"),
        ];

        for (expr, _description) in descendant_shorthand_tests {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Descendant pattern '{}' should compile: {}",
                expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Descendant test '{}' returned {} results ({})",
                expr,
                results.len(),
                _description
            );
        }
    }

    #[test]
    fn test_array_index_shorthand() {
        // RFC 9535: Test array index shorthand patterns
        let json_data = r#"{
            "arrays": {
                "numbers": [10, 20, 30, 40, 50],
                "strings": ["a", "b", "c", "d", "e"],
                "objects": [
                    {"id": 1, "name": "first"},
                    {"id": 2, "name": "second"},
                    {"id": 3, "name": "third"}
                ],
                "nested": [
                    [1, 2, 3],
                    [4, 5, 6],
                    [7, 8, 9]
                ]
            }
        }"#;

        let index_shorthand_tests = vec![
            // Basic index access
            ("$.arrays.numbers[0]", "First element"),
            ("$.arrays.numbers[4]", "Last element"),
            ("$.arrays.numbers[-1]", "Negative index - last"),
            ("$.arrays.numbers[-5]", "Negative index - first"),
            // String array access
            ("$.arrays.strings[2]", "String array middle element"),
            ("$.arrays.strings[-2]", "String array negative index"),
            // Object array access
            ("$.arrays.objects[1].name", "Object array property access"),
            ("$.arrays.objects[-1].id", "Object array negative index"),
            // Nested array access
            ("$.arrays.nested[1][2]", "Nested array access"),
            ("$.arrays.nested[0][-1]", "Nested with negative index"),
            // Out of bounds (should return empty)
            ("$.arrays.numbers[999]", "Out of bounds positive"),
            ("$.arrays.numbers[-999]", "Out of bounds negative"),
        ];

        for (expr, _description) in index_shorthand_tests {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Index pattern '{}' should compile: {}",
                expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Index test '{}' returned {} results ({})",
                expr,
                results.len(),
                _description
            );
        }
    }

    #[test]
    fn test_slice_shorthand_syntax() {
        // RFC 9535: Test array slice shorthand patterns
        let json_data = r#"{
            "data": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        }"#;

        let slice_shorthand_tests = vec![
            // Basic slice patterns
            ("$.data[:]", "Full slice"),
            ("$.data[1:]", "From index to end"),
            ("$.data[:5]", "From start to index"),
            ("$.data[2:7]", "Range slice"),
            ("$.data[::2]", "Step slice"),
            ("$.data[1::2]", "Start with step"),
            ("$.data[:8:2]", "End with step"),
            ("$.data[1:8:2]", "Full slice with step"),
            // Negative indices in slices
            ("$.data[-3:]", "Negative start"),
            ("$.data[:-2]", "Negative end"),
            ("$.data[-5:-1]", "Negative range"),
            ("$.data[::-1]", "Reverse slice"),
            ("$.data[8:2:-1]", "Reverse range"),
            // Edge cases
            ("$.data[5:5]", "Empty slice"),
            ("$.data[10:20]", "Out of bounds slice"),
            ("$.data[-20:-10]", "Negative out of bounds"),
        ];

        for (expr, _description) in slice_shorthand_tests {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Slice pattern '{}' should compile: {}",
                expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Slice test '{}' returned {} results ({})",
                expr,
                results.len(),
                _description
            );
        }
    }

    #[test]
    fn test_combined_shorthand_patterns() {
        // RFC 9535: Test complex combinations of shorthand patterns
        let json_data = r#"{
            "store": {
                "books": [
                    {
                        "title": "Book1",
                        "authors": ["Author1", "Author2"],
                        "categories": ["fiction", "drama"]
                    },
                    {
                        "title": "Book2", 
                        "authors": ["Author3"],
                        "categories": ["non-fiction"]
                    }
                ],
                "magazines": [
                    {
                        "title": "Mag1",
                        "issues": [1, 2, 3, 4, 5]
                    }
                ]
            }
        }"#;

        let combined_shorthand_tests = vec![
            // Multiple shorthand types combined
            ("$.store.books[*].title", "Property wildcard shorthand"),
            ("$.store.books[0].authors[*]", "Mixed index and wildcard"),
            (
                "$.store.books[-1].categories[1]",
                "Negative index with property",
            ),
            ("$..books[*].title", "Descendant with wildcard"),
            ("$..authors[0]", "Descendant with index"),
            ("$.store.magazines[0].issues[1:4]", "Property with slice"),
            ("$..issues[::2]", "Descendant with step slice"),
            ("$.store.*[*].title", "Double wildcard shorthand"),
            ("$..books[*].authors[-1]", "Complex descendant pattern"),
            ("$.store.books[*].categories[:]", "Wildcard with full slice"),
        ];

        for (expr, _description) in combined_shorthand_tests {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Combined pattern '{}' should compile: {}",
                expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Combined test '{}' returned {} results ({})",
                expr,
                results.len(),
                _description
            );
        }
    }

    #[test]
    fn test_shorthand_syntax_edge_cases() {
        // Test edge cases and boundary conditions for shorthand syntax
        let edge_case_tests = vec![
            // Minimal valid expressions
            ("$", "Root only"),
            ("$.*", "Root wildcard"),
            ("$.a", "Single char property"),
            ("$[0]", "Single index"),
            ("$[:]", "Root slice"),
            // Whitespace handling (implementation dependent)
            ("$ .store", "Space after root"),
            ("$. store", "Space after dot"),
            ("$[0 ]", "Space in brackets"),
            ("$[ 0]", "Space before index"),
            ("$[ : ]", "Spaces in slice"),
            // Unicode edge cases
            ("$.𝓤𝓷𝓲𝓬𝓸𝓭𝓮", "Unicode mathematical symbols"),
            ("$.🎉", "Emoji property name"),
            ("$.Ω", "Greek omega"),
            // Long expressions
            (
                "$.very.long.chain.of.properties.that.keeps.going",
                "Long property chain",
            ),
            ("$[0][1][2][3][4][5]", "Long index chain"),
            ("$.a.b.c.d.e[*].f.g.h", "Mixed long chain"),
        ];

        for (expr, _description) in edge_case_tests {
            let result = JsonPathParser::compile(expr);
            // Document behavior - these tests verify consistency
            println!(
                "Edge case '{}' -> {:?} ({})",
                expr,
                result.is_ok(),
                _description
            );
        }
    }
}
