//! RFC 9535 Segment Compliance Tests (Section 2.5)
//!
//! Tests for child segments and descendant segments

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    name: String,
    value: i32,
}

/// RFC 9535 Section 2.5.1 - Child Segment Tests
#[cfg(test)]
mod child_segment_tests {
    use super::*;

    #[test]
    fn test_child_segment_syntax() {
        // RFC 9535: child-segment = "[" selector-list "]"
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
                "Valid child segment '{}' should compile",
                expr
            );
        }
    }

    #[test]
    fn test_child_segment_object_selection() {
        // RFC 9535: Child segment selects direct children of object
        let json_data = r#"{
            "store": {
                "book": [{"name": "book1"}],
                "bicycle": {"name": "bike1"}
            }
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store['book','bicycle']");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 2, "Should select both book and bicycle");
    }

    #[test]
    fn test_child_segment_array_selection() {
        // RFC 9535: Child segment selects direct elements of array
        let json_data = r#"[
            {"name": "item0"},
            {"name": "item1"}, 
            {"name": "item2"},
            {"name": "item3"}
        ]"#;

        let mut stream = JsonArrayStream::<TestModel>::new("$[0,2]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 2, "Should select items 0 and 2");
        assert_eq!(results[0].name, "item0");
        assert_eq!(results[1].name, "item2");
    }

    #[test]
    fn test_dot_notation_child_segment() {
        // RFC 9535: Dot notation is equivalent to bracket notation for names
        let json_data = r#"{"store": {"book": {"name": "test"}}}"#;

        let dot_stream = JsonArrayStream::<serde_json::Value>::new("$.store.book");
        let bracket_stream = JsonArrayStream::<serde_json::Value>::new("$['store']['book']");

        let chunk = Bytes::from(json_data);

        let mut dot_stream = dot_stream;
        let mut bracket_stream = bracket_stream;

        let dotresults: Vec<_> = dot_stream.process_chunk(chunk.clone()).collect();
        let bracketresults: Vec<_> = bracket_stream.process_chunk(chunk).collect();

        assert_eq!(
            dotresults.len(),
            bracketresults.len(),
            "Dot and bracket notation should yield same results"
        );
    }

    #[test]
    fn test_mixed_child_selectors() {
        // RFC 9535: Child segment can contain multiple selector types
        let json_data = r#"{
            "items": [
                {"name": "array_item_0"},
                {"name": "array_item_1"}
            ],
            "store": {"name": "store_item"},
            "count": 42
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$['items','store','count']");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 3, "Should select all three properties");
    }

    #[test]
    fn test_child_segment_emptyresults() {
        // RFC 9535: Child segment with no matches returns empty nodelist
        let json_data = r#"{"store": {"book": []}}"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$['nonexistent','missing']");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            0,
            "Nonexistent selectors should return empty nodelist"
        );
    }
}

/// RFC 9535 Section 2.5.2 - Descendant Segment Tests
#[cfg(test)]
mod descendant_segment_tests {
    use super::*;

    #[test]
    fn test_descendant_segment_syntax() {
        // RFC 9535: descendant-segment = ".." child-segment
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
                "Valid descendant segment '{}' should compile",
                expr
            );
        }
    }

    #[test]
    fn test_descendant_recursive_search() {
        // RFC 9535: Descendant segment selects all descendants
        let json_data = r#"{
            "store": {
                "book": [
                    {"title": "Book1", "author": {"name": "Author1"}},
                    {"title": "Book2", "author": {"name": "Author2"}}
                ],
                "bicycle": {
                    "brand": {"name": "Brand1"}
                }
            }
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$..['name']");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        // Should find all 'name' properties at any depth
        assert!(
            results.len() >= 3,
            "Should find multiple 'name' properties recursively"
        );
    }

    #[test]
    fn test_descendant_vs_child_difference() {
        // RFC 9535: Demonstrate difference between child and descendant
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

        assert_eq!(childresults.len(), 1, "Child should find only direct child");
        assert_eq!(
            descendantresults.len(),
            2,
            "Descendant should find all targets"
        );
    }

    #[test]
    fn test_descendant_array_search() {
        // RFC 9535: Descendant search in arrays
        let json_data = r#"{
            "data": [
                {"items": [{"value": 1}, {"value": 2}]},
                {"items": [{"value": 3}, {"value": 4}]}
            ]
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$..['value']");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            4,
            "Should find all value properties in nested arrays"
        );
    }

    #[test]
    fn test_descendant_wildcard() {
        // RFC 9535: Descendant wildcard selects all descendants
        let json_data = r#"{
            "a": {
                "b": {"c": 1, "d": 2},
                "e": [3, 4, 5]
            },
            "f": {"g": 6}
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$..[*]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        // Should find all values at all descendant levels
        assert!(
            results.len() >= 8,
            "Descendant wildcard should find many values"
        );
    }

    #[test]
    fn test_descendant_specific_paths() {
        // RFC 9535: Descendant search for specific property names
        let json_data = r#"{
            "store": {
                "book": [
                    {"price": 8.95},
                    {"price": 12.99}
                ],
                "bicycle": {"price": 19.95}
            }
        }"#;

        let mut stream = JsonArrayStream::<f64>::new("$..price");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 3, "Should find all price properties");
        assert!(results.contains(&8.95));
        assert!(results.contains(&12.99));
        assert!(results.contains(&19.95));
    }

    #[test]
    fn test_descendant_depth_unlimited() {
        // RFC 9535: Descendant search has unlimited depth
        let json_data = r#"{
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "target": "deep_value"
                            }
                        }
                    }
                }
            }
        }"#;

        let mut stream = JsonArrayStream::<String>::new("$..target");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 1, "Should find deeply nested target");
        assert_eq!(results[0], "deep_value");
    }
}

/// Segment Combination Tests
#[cfg(test)]
mod segment_combination_tests {
    use super::*;

    #[test]
    fn test_child_after_descendant() {
        // RFC 9535: Child segment after descendant segment
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
            "Should find first book in each books array"
        );
    }

    #[test]
    fn test_descendant_after_child() {
        // RFC 9535: Descendant segment after child segment
        let json_data = r#"{
            "store": {
                "section1": {
                    "item": {"price": 10},
                    "nested": {"item": {"price": 20}}
                },
                "section2": {
                    "item": {"price": 30}
                }
            }
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store..price");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 3, "Should find all prices under store");
    }

    #[test]
    fn test_multiple_descendant_segments() {
        // RFC 9535: Multiple descendant segments in sequence
        let json_data = r#"{
            "root": {
                "branch1": {
                    "leaf": {"data": {"value": 1}}
                },
                "branch2": {
                    "subbranch": {
                        "leaf": {"data": {"value": 2}}
                    }
                }
            }
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$..leaf..value");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 2, "Should find values under all leaf nodes");
    }

    #[test]
    fn test_segment_performance_deep_nesting() {
        // Test performance with deep nesting
        let mut nested_json = serde_json::json!({"value": 42});

        // Create deeply nested structure
        for i in 0..50 {
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

        assert_eq!(results.len(), 1, "Should find the deeply nested value");
        println!("Deep descendant search took {:?}", duration);

        // Performance should be reasonable even for deep nesting
        assert!(
            duration.as_millis() < 1000,
            "Deep search should complete in <1s"
        );
    }
}

/// Edge Cases and Error Conditions
#[cfg(test)]
mod segment_error_tests {
    use super::*;

    #[test]
    fn test_invalid_segment_syntax() {
        // Test invalid segment syntaxes
        let invalid_expressions = vec![
            "$.[store]",      // Invalid child segment syntax
            "$..[",           // Unclosed descendant segment
            "$...]",          // Invalid descendant syntax
            "$...store",      // Triple dot
            "$.store.[book]", // Invalid mixed syntax
        ];

        for expr in invalid_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_err(), "Invalid segment '{}' should fail", expr);
        }
    }

    #[test]
    fn test_empty_segments() {
        // Test empty segments
        let empty_segments = vec![
            "$[]",       // Empty child segment
            "$.[]",      // Empty child after dot
            "$.store[]", // Empty child after property
        ];

        for expr in empty_segments {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_err(), "Empty segment '{}' should fail", expr);
        }
    }

    #[test]
    fn test_segment_on_primitive_values() {
        // RFC 9535: Segments on primitive values should return empty nodelist
        let json_data = r#"{"value": 42}"#;

        let test_cases = vec![
            "$.value['property']", // Child segment on number
            "$.value..[*]",        // Descendant segment on number
        ];

        for expr in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                0,
                "Segment on primitive '{}' should return empty nodelist",
                expr
            );
        }
    }
}
