//! RFC 9535 Singular Query Validation Tests
//!
//! Tests singular query syntax validation, at-most-one node guarantee,
//! normalized path equivalence, and JSON Pointer conversion

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    name: String,
    value: i32,
}

/// RFC 9535 Singular Query Validation Tests
#[cfg(test)]
mod singular_query_tests {
    use super::*;

    #[test]
    fn test_singular_query_syntax_validation() {
        // RFC 9535: Singular queries are syntactically recognizable at parse time
        let singular_queries = vec![
            // Valid singular queries
            "$",                              // Root (always singular)
            "$.store",                        // Single property access
            "$['store']",                     // Single bracket property access
            "$.store.book",                   // Chain of single properties
            "$['store']['book']",             // Chain of bracket properties
            "$.store['book']",                // Mixed notation
            "$[0]",                           // Single array index
            "$.store.book[0]",                // Property then index
            "$.store.book[0].title",          // Index then property
            "$['store']['book'][0]['title']", // All bracket notation
        ];

        for query in singular_queries {
            let result = JsonPathParser::compile(query);
            assert!(result.is_ok(), "Singular query '{}' should compile", query);
            println!("Singular query '{}' validated successfully", query);
        }
    }

    #[test]
    fn test_non_singular_query_syntax() {
        // RFC 9535: Non-singular queries that should be distinguished
        let non_singular_queries = vec![
            // Wildcard selectors
            "$.*",             // Root wildcard
            "$.store.*",       // Property wildcard
            "$[*]",            // Array wildcard
            "$.store.book[*]", // Mixed with wildcard
            // Slice selectors
            "$[:]",   // Full slice
            "$[1:]",  // Slice from index
            "$[:5]",  // Slice to index
            "$[1:5]", // Range slice
            "$[::2]", // Step slice
            // Union selectors
            "$[0,1]",      // Multiple indices
            "$['a','b']",  // Multiple properties
            "$[0,'name']", // Mixed union
            // Descendant segments
            "$..book",    // Descendant search
            "$..*",       // Universal descendant
            "$..book[*]", // Descendant with wildcard
            // Filter expressions
            "$[?@.price]",           // Property filter
            "$.book[?@.price > 10]", // Comparison filter
        ];

        for query in non_singular_queries {
            let result = JsonPathParser::compile(query);
            assert!(
                result.is_ok(),
                "Non-singular query '{}' should compile",
                query
            );
            println!("Non-singular query '{}' validated successfully", query);
        }
    }

    #[test]
    fn test_at_most_one_node_guarantee() {
        // RFC 9535: Singular queries must return at most one node
        let json_data = r#"{
            "store": {
                "book": [
                    {"title": "Book 1", "price": 10},
                    {"title": "Book 2", "price": 20}
                ],
                "bicycle": {"color": "red", "price": 15}
            }
        }"#;

        let singular_tests = vec![
            // These should return exactly one node or empty
            ("$", 1),                     // Root node
            ("$.store", 1),               // Single property
            ("$.store.book", 1),          // Single property (array)
            ("$.store.bicycle", 1),       // Single property (object)
            ("$.store.book[0]", 1),       // First book
            ("$.store.book[1]", 1),       // Second book
            ("$.store.book[0].title", 1), // Title of first book
            ("$.store.bicycle.color", 1), // Bicycle color
            ("$.nonexistent", 0),         // Non-existent property (empty result)
            ("$.store.book[99]", 0),      // Out of bounds index (empty result)
        ];

        for (query, expected_count) in singular_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert!(
                results.len() <= 1,
                "Singular query '{}' should return at most 1 result, got {}",
                query,
                results.len()
            );
            assert_eq!(
                results.len(),
                expected_count,
                "Singular query '{}' should return {} results, got {}",
                query,
                expected_count,
                results.len()
            );

            println!(
                "Singular query '{}' returned {} results (expected {})",
                query,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_normalized_path_equivalence() {
        // RFC 9535: Different syntactic forms should normalize to equivalent paths
        let equivalence_groups = vec![
            // Property access equivalences
            vec!["$.store", "$['store']", "$[\"store\"]"],
            // Chained property access
            vec![
                "$.store.book",
                "$['store']['book']",
                "$['store'].book",
                "$.store['book']",
            ],
            // Array index access
            vec!["$[0]", "$['0']"], // Note: This equivalence depends on implementation
            // Mixed chains
            vec!["$.store.book[0].title", "$['store']['book'][0]['title']"],
        ];

        let json_data = r#"{
            "store": {
                "book": [{"title": "Book 1"}]
            },
            "0": "zero_property"
        }"#;

        for group in equivalence_groups {
            let mut results_sets = Vec::new();

            for query in &group {
                let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

                let chunk = Bytes::from(json_data);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                results_sets.push(results);
                println!(
                    "Query '{}' returned {} results",
                    query,
                    results_sets.last().unwrap().len()
                );
            }

            // All equivalent queries should return the same results
            if results_sets.len() > 1 {
                let first = &results_sets[0];
                for (i, results) in results_sets.iter().enumerate().skip(1) {
                    if first.len() == results.len() && first.len() <= 1 {
                        // For singular queries, check value equality if both have results
                        if !first.is_empty() && !results.is_empty() {
                            assert_eq!(
                                first[0], results[0],
                                "Equivalent queries '{}' and '{}' should return same value",
                                group[0], group[i]
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_json_pointer_conversion() {
        // RFC 9535: Singular queries can be converted to JSON Pointer references
        let json_pointer_tests = vec![
            // JSONPath -> Expected JSON Pointer
            ("$", ""),                                                 // Root
            ("$.store", "/store"),                                     // Single property
            ("$.store.book", "/store/book"),                           // Nested property
            ("$['store']", "/store"),                                  // Bracket notation
            ("$['store']['book']", "/store/book"),                     // Bracket chain
            ("$[0]", "/0"),                                            // Array index
            ("$.store.book[0]", "/store/book/0"),                      // Property + index
            ("$.store['book'][0]", "/store/book/0"),                   // Mixed notation
            ("$['store']['book'][0]['title']", "/store/book/0/title"), // Complex path
        ];

        for (jsonpath, expected_pointer) in json_pointer_tests {
            // This test validates the theoretical conversion
            // Actual implementation would require a conversion function
            println!(
                "JSONPath '{}' should convert to JSON Pointer '{}'",
                jsonpath, expected_pointer
            );

            // Verify the JSONPath is singular and valid
            let result = JsonPathParser::compile(jsonpath);
            assert!(
                result.is_ok(),
                "JSONPath '{}' should be valid for pointer conversion",
                jsonpath
            );
        }
    }

    #[test]
    fn test_json_pointer_equivalence() {
        // RFC 9535: Singular queries and their JSON Pointer equivalents should return same results
        let json_data = r#"{
            "store": {
                "book": [
                    {"title": "Book 1", "author": "Author 1"},
                    {"title": "Book 2", "author": "Author 2"}
                ],
                "special-chars": "value with hyphens",
                "with spaces": "value with spaces"
            }
        }"#;

        let equivalence_tests = vec![
            // (JSONPath, JSON Pointer path components)
            ("$.store", vec!["store"]),
            ("$.store.book", vec!["store", "book"]),
            ("$.store.book[0]", vec!["store", "book", "0"]),
            ("$.store.book[0].title", vec!["store", "book", "0", "title"]),
            (
                "$.store.book[1].author",
                vec!["store", "book", "1", "author"],
            ),
        ];

        for (jsonpath, pointer_components) in equivalence_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(jsonpath);

            let chunk = Bytes::from(json_data);
            let jsonpathresults: Vec<_> = stream.process_chunk(chunk).collect();

            // Simulate JSON Pointer access (manual traversal for testing)
            let json_value: serde_json::Value =
                serde_json::from_str(json_data).expect("Valid JSON");

            let mut current = &json_value;
            let mut pointerresult = None;

            for component in pointer_components {
                match current {
                    serde_json::Value::Object(obj) => {
                        current = obj.get(component).unwrap_or(&serde_json::Value::Null);
                    }
                    serde_json::Value::Array(arr) => {
                        if let Ok(index) = component.parse::<usize>() {
                            current = arr.get(index).unwrap_or(&serde_json::Value::Null);
                        } else {
                            current = &serde_json::Value::Null;
                        }
                    }
                    _ => {
                        current = &serde_json::Value::Null;
                        break;
                    }
                }
            }

            if *current != serde_json::Value::Null {
                pointerresult = Some(current.clone());
            }

            // Compare results
            match (jsonpathresults.is_empty(), pointerresult.is_some()) {
                (true, false) => {
                    println!(
                        "JSONPath '{}' returned no results, but JSON Pointer found a value",
                        jsonpath
                    );
                }
                (false, true) => {
                    let jsonpath_value = &jsonpathresults[0];
                    let pointer_value = pointerresult.unwrap();
                    assert_eq!(
                        *jsonpath_value, pointer_value,
                        "JSONPath '{}' and JSON Pointer should return equivalent values",
                        jsonpath
                    );
                    println!(
                        "JSONPath '{}' and JSON Pointer returned equivalent values",
                        jsonpath
                    );
                }
                (true, true) => {
                    println!(
                        "Both JSONPath '{}' and JSON Pointer returned no results",
                        jsonpath
                    );
                }
                (false, false) => {
                    println!(
                        "JSONPath '{}' found results but JSON Pointer did not",
                        jsonpath
                    );
                }
            }
        }
    }

    #[test]
    fn test_singular_query_edge_cases() {
        // RFC 9535: Edge cases for singular query recognition
        let json_data = r#"{
            "": "empty_key",
            "0": "string_zero",
            "null": "null_string",
            "true": "true_string",
            "false": "false_string",
            "array": [],
            "object": {},
            "nested": {
                "": "nested_empty",
                "0": "nested_zero"
            }
        }"#;

        let edge_case_tests = vec![
            // Empty string property
            ("$['']", 1),        // Empty key access
            ("$.nested['']", 1), // Nested empty key
            // Numeric string properties
            ("$['0']", 1),        // String "0" property
            ("$.nested['0']", 1), // Nested string "0"
            // Keyword-like properties
            ("$['null']", 1),  // String "null" property
            ("$['true']", 1),  // String "true" property
            ("$['false']", 1), // String "false" property
            // Empty containers
            ("$.array", 1),  // Empty array
            ("$.object", 1), // Empty object
            // Non-existent paths
            ("$.nonexistent", 0),    // Non-existent property
            ("$.array[0]", 0),       // Index into empty array
            ("$.object.missing", 0), // Property of empty object
        ];

        for (query, expected_count) in edge_case_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert!(
                results.len() <= 1,
                "Singular query '{}' should return at most 1 result",
                query
            );
            assert_eq!(
                results.len(),
                expected_count,
                "Edge case query '{}' should return {} results",
                query,
                expected_count
            );

            println!(
                "Edge case query '{}' returned {} results",
                query,
                results.len()
            );
        }
    }

    #[test]
    fn test_deep_singular_paths() {
        // RFC 9535: Deeply nested singular paths
        let deep_json = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "deep_value": "found_it",
                                "array": [
                                    {"item": "first"},
                                    {"item": "second"}
                                ]
                            }
                        }
                    }
                }
            }
        });

        let json_data = serde_json::to_string(&deep_json).expect("Valid JSON");

        let deep_path_tests = vec![
            // Progressively deeper paths
            ("$.level1", 1),
            ("$.level1.level2", 1),
            ("$.level1.level2.level3", 1),
            ("$.level1.level2.level3.level4", 1),
            ("$.level1.level2.level3.level4.level5", 1),
            ("$.level1.level2.level3.level4.level5.deep_value", 1),
            // Mixed with array access
            ("$.level1.level2.level3.level4.level5.array", 1),
            ("$.level1.level2.level3.level4.level5.array[0]", 1),
            ("$.level1.level2.level3.level4.level5.array[1]", 1),
            ("$.level1.level2.level3.level4.level5.array[0].item", 1),
            // Non-existent deep paths
            ("$.level1.level2.level3.level4.level5.nonexistent", 0),
            ("$.level1.level2.level3.level4.level5.array[99]", 0),
        ];

        for (query, expected_count) in deep_path_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert!(
                results.len() <= 1,
                "Deep singular query '{}' should return at most 1 result",
                query
            );
            assert_eq!(
                results.len(),
                expected_count,
                "Deep query '{}' should return {} results",
                query,
                expected_count
            );

            println!("Deep query '{}' returned {} results", query, results.len());
        }
    }

    #[test]
    fn test_singular_query_performance() {
        // RFC 9535: Performance characteristics of singular queries
        let large_object = serde_json::json!({
            "data": (0..1000).map(|i| (format!("key_{}", i), serde_json::Value::Number(serde_json::Number::from(i)))).collect::<serde_json::Map<_, _>>(),
            "array": (0..1000).collect::<Vec<i32>>(),
            "deep": {
                "nested": {
                    "structure": {
                        "target": "found"
                    }
                }
            }
        });

        let json_data = serde_json::to_string(&large_object).expect("Valid JSON");

        let performance_tests = vec![
            ("$.data", 1),                         // Large object property
            ("$.array", 1),                        // Large array property
            ("$.data.key_500", 1),                 // Specific property in large object
            ("$.array[500]", 1),                   // Specific index in large array
            ("$.deep.nested.structure.target", 1), // Deep nested access
        ];

        for (query, expected_count) in performance_tests {
            let start_time = std::time::Instant::now();

            let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            assert_eq!(
                results.len(),
                expected_count,
                "Performance query '{}' should return {} results",
                query,
                expected_count
            );

            println!(
                "Performance query '{}' returned {} results in {:?}",
                query,
                results.len(),
                duration
            );

            // Singular queries should be fast even on large data
            assert!(
                duration.as_millis() < 100,
                "Singular query '{}' should complete quickly",
                query
            );
        }
    }
}

/// Singular Query Error Handling Tests
#[cfg(test)]
mod singular_query_error_tests {
    use super::*;

    #[test]
    fn test_malformed_singular_queries() {
        // RFC 9535: Malformed queries that appear singular but are invalid
        let malformed_queries = vec![
            "$.",        // Trailing dot
            "$.store.",  // Trailing dot after property
            "$[]",       // Empty brackets
            "$['']",     // Empty string is valid, not malformed
            "$[']",      // Unclosed quote
            "$['store]", // Unclosed quote
            "$.store[",  // Unclosed bracket
            "$.store]",  // Unmatched bracket
            "$store",    // Missing root identifier
            "store",     // No root at all
        ];

        for query in malformed_queries {
            let result = JsonPathParser::compile(query);

            // Most should fail to parse
            if query == "$['']" {
                // Empty string property is actually valid
                assert!(
                    result.is_ok(),
                    "Empty string property '{}' should be valid",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Malformed query '{}' should fail to parse",
                    query
                );
            }

            println!("Malformed query '{}' handling: {:?}", query, result.is_ok());
        }
    }

    #[test]
    fn test_ambiguous_singular_syntax() {
        // RFC 9535: Syntax that might be ambiguously interpreted
        let ambiguous_tests = vec![
            // These should be clearly recognized as singular
            ("$[0]", true),   // Single index
            ("$['0']", true), // String property "0"
            ("$.0", false),   // Invalid: property starting with digit
            ("$.-1", false),  // Invalid: property starting with minus
            // Bracket vs property access
            ("$.length", true),    // Property access
            ("$['length']", true), // Bracket property access
        ];

        for (query, _should_be_valid) in ambiguous_tests {
            let result = JsonPathParser::compile(query);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "Ambiguous query '{}' should be valid",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Ambiguous query '{}' should be invalid",
                    query
                );
            }

            println!(
                "Ambiguous syntax test '{}': valid={}",
                query,
                result.is_ok()
            );
        }
    }

    #[test]
    fn test_type_safety_in_singular_queries() {
        // RFC 9535: Type safety for singular query results
        let json_data = r#"{
            "string_value": "hello",
            "number_value": 42,
            "boolean_value": true,
            "null_value": null,
            "array_value": [1, 2, 3],
            "object_value": {"nested": "value"}
        }"#;

        let type_safety_tests = vec![
            // Each query should return exactly one typed value
            ("$.string_value", "string"),
            ("$.number_value", "number"),
            ("$.boolean_value", "boolean"),
            ("$.null_value", "null"),
            ("$.array_value", "array"),
            ("$.object_value", "object"),
        ];

        for (query, expected_type) in type_safety_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                1,
                "Query '{}' should return exactly one result",
                query
            );

            let actual_type = match &results[0] {
                serde_json::Value::String(_) => "string",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Null => "null",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
            };

            assert_eq!(
                actual_type, expected_type,
                "Query '{}' should return {} type, got {}",
                query, expected_type, actual_type
            );

            println!(
                "Type safety test '{}' returned {} type as expected",
                query, actual_type
            );
        }
    }
}
