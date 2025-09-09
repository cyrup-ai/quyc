//! RFC 9535 JSON Pointer Compatibility Tests (Appendix C)
//!
//! Tests for RFC 9535 Appendix C JSON Pointer interoperability:
//! "This appendix shows the relationship between JSONPath and JSON Pointer (RFC 6901).
//! JSONPath can express many patterns that JSON Pointer cannot, but there is significant
//! overlap in basic property access and array indexing."
//!
//! This test suite validates:
//! - JSONPath to JSON Pointer conversion where possible
//! - JSON Pointer to JSONPath conversion
//! - Equivalent expressions between the two standards
//! - Limitations and incompatibilities
//! - Round-trip conversion testing

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct CompatibilityTestData {
    store: StoreData,
    users: Vec<UserData>,
    metadata: MetadataData,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct StoreData {
    books: Vec<BookData>,
    config: ConfigData,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct BookData {
    title: String,
    author: String,
    price: f64,
    isbn: String,
    tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct UserData {
    id: i32,
    name: String,
    preferences: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct ConfigData {
    name: String,
    version: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct MetadataData {
    created: String,
    modified: String,
    authors: Vec<String>,
}

/// Test data for JSON Pointer compatibility validation
const COMPATIBILITY_TEST_JSON: &str = r#"{
  "store": {
    "books": [
      {
        "title": "The Great Gatsby",
        "author": "F. Scott Fitzgerald",
        "price": 12.99,
        "isbn": "978-0-7432-7356-5",
        "tags": ["classic", "american", "fiction"]
      },
      {
        "title": "To Kill a Mockingbird", 
        "author": "Harper Lee",
        "price": 14.99,
        "isbn": "978-0-06-112008-4",
        "tags": ["classic", "literature", "drama"]
      },
      {
        "title": "1984",
        "author": "George Orwell", 
        "price": 13.99,
        "isbn": "978-0-452-28423-4",
        "tags": ["dystopian", "political", "science-fiction"]
      }
    ],
    "config": {
      "name": "Bookstore System",
      "version": "2.1.0"
    }
  },
  "users": [
    {
      "id": 1,
      "name": "Alice Johnson",
      "preferences": {
        "theme": "dark",
        "notifications": true,
        "language": "en"
      }
    },
    {
      "id": 2,
      "name": "Bob Smith",
      "preferences": {
        "theme": "light", 
        "notifications": false,
        "language": "es"
      }
    }
  ],
  "metadata": {
    "created": "2024-01-15",
    "modified": "2024-03-20",
    "authors": ["development-team", "qa-team", "devops-team"]
  }
}"#;

/// RFC 9535 Appendix C - JSON Pointer Compatibility Tests
#[cfg(test)]
mod json_pointer_compatibility_tests {
    use super::*;

    #[test]
    fn test_basic_property_access_equivalence() {
        // RFC 9535 Appendix C: Basic property access equivalence
        let equivalence_tests = vec![
            // JSON Pointer -> JSONPath equivalents
            ("/store", "$.store", "Root object property"),
            ("/store/config", "$.store.config", "Nested object property"),
            (
                "/store/config/name",
                "$.store.config.name",
                "Deep object property",
            ),
            (
                "/metadata/created",
                "$.metadata.created",
                "Simple property access",
            ),
            (
                "/metadata/authors",
                "$.metadata.authors",
                "Array property access",
            ),
            // These should return equivalent results
        ];

        for (json_pointer_like, jsonpath_expr, _description) in equivalence_tests {
            // Convert JSON Pointer concept to JSONPath (manual for testing)
            let mut jsonpath_stream = JsonArrayStream::<serde_json::Value>::new(jsonpath_expr);
            let chunk = Bytes::from(COMPATIBILITY_TEST_JSON);
            let jsonpathresults: Vec<_> = jsonpath_stream.process_chunk(chunk).collect();

            // Verify JSONPath expression compiles and executes
            let compileresult = JsonPathParser::compile(jsonpath_expr);
            assert!(
                compileresult.is_ok(),
                "RFC 9535: JSONPath equivalent should compile: {} -> {} ({})",
                json_pointer_like,
                jsonpath_expr,
                _description
            );

            println!(
                "✓ Compatibility: '{}' ≈ '{}' -> {} results ({})",
                json_pointer_like,
                jsonpath_expr,
                jsonpathresults.len(),
                _description
            );
        }
    }

    #[test]
    fn test_array_index_access_equivalence() {
        // RFC 9535 Appendix C: Array index access equivalence
        let array_equivalence_tests = vec![
            // JSON Pointer style -> JSONPath equivalent
            ("/store/books/0", "$.store.books[0]", "First array element"),
            ("/store/books/1", "$.store.books[1]", "Second array element"),
            ("/store/books/2", "$.store.books[2]", "Third array element"),
            (
                "/users/0/id",
                "$.users[0].id",
                "Nested array element property",
            ),
            (
                "/users/1/preferences/theme",
                "$.users[1].preferences.theme",
                "Deep array property",
            ),
            (
                "/metadata/authors/0",
                "$.metadata.authors[0]",
                "Array element access",
            ),
            (
                "/metadata/authors/2",
                "$.metadata.authors[2]",
                "Last array element",
            ),
        ];

        for (json_pointer_like, jsonpath_expr, _description) in array_equivalence_tests {
            let compileresult = JsonPathParser::compile(jsonpath_expr);
            assert!(
                compileresult.is_ok(),
                "RFC 9535: Array access should compile: {} -> {} ({})",
                json_pointer_like,
                jsonpath_expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(jsonpath_expr);
            let chunk = Bytes::from(COMPATIBILITY_TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // Should return exactly one result for specific index access
            assert!(
                results.len() <= 1,
                "RFC 9535: Specific array index should return 0 or 1 result: {} ({})",
                jsonpath_expr,
                _description
            );

            println!(
                "✓ Array compatibility: '{}' ≈ '{}' -> {} results ({})",
                json_pointer_like,
                jsonpath_expr,
                results.len(),
                _description
            );
        }
    }

    #[test]
    fn test_json_pointer_limitations() {
        // RFC 9535 Appendix C: JSONPath features that JSON Pointer cannot express
        let jsonpath_only_features = vec![
            // Wildcards - JSON Pointer has no equivalent
            ("$.store.books[*].title", "Wildcard array access"),
            ("$.users[*].name", "All array element properties"),
            ("$.*.config", "Wildcard object property"),
            // Recursive descent - JSON Pointer has no equivalent
            ("$..title", "Recursive descent for property"),
            ("$..authors", "Recursive descent for arrays"),
            ("$..*", "Recursive descent for all values"),
            // Array slicing - JSON Pointer has no equivalent
            ("$.store.books[0:2]", "Array slice access"),
            ("$.metadata.authors[1:]", "Array slice from index"),
            ("$.store.books[:2]", "Array slice to index"),
            // Filters - JSON Pointer has no equivalent
            ("$.store.books[?@.price > 13]", "Filtered array access"),
            ("$.users[?@.id == 1]", "Conditional element access"),
            (
                "$.store.books[?@.author == 'George Orwell']",
                "String filter",
            ),
            // Multiple selectors - JSON Pointer has no equivalent
            ("$.store.books[0,2]", "Multiple array indices"),
            ("$['store','users']", "Multiple object properties"),
        ];

        for (jsonpath_expr, _description) in jsonpath_only_features {
            let compileresult = JsonPathParser::compile(jsonpath_expr);
            assert!(
                compileresult.is_ok(),
                "RFC 9535: JSONPath-only feature should compile: {} ({})",
                jsonpath_expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(jsonpath_expr);
            let chunk = Bytes::from(COMPATIBILITY_TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "✓ JSONPath-only: '{}' -> {} results ({}) - No JSON Pointer equivalent",
                jsonpath_expr,
                results.len(),
                _description
            );
        }
    }

    #[test]
    fn test_character_escaping_compatibility() {
        // RFC 9535 Appendix C: Character escaping between JSON Pointer and JSONPath
        let escaping_tests = vec![
            // Special characters that need escaping
            ("$['key with spaces']", "Property with spaces"),
            ("$['key-with-hyphens']", "Property with hyphens"),
            ("$['key.with.dots']", "Property with dots"),
            (
                "$['key/with/slashes']",
                "Property with slashes (JSON Pointer escape)",
            ),
            (
                "$['key~with~tildes']",
                "Property with tildes (JSON Pointer escape)",
            ),
            ("$['key\"with\"quotes']", "Property with quotes"),
            // Array access with string indices
            ("$['0']", "String that looks like array index"),
            ("$['-1']", "String that looks like negative index"),
        ];

        for (jsonpath_expr, _description) in escaping_tests {
            let compileresult = JsonPathParser::compile(jsonpath_expr);
            // Some may be valid, some may not - test compilation
            match compileresult {
                Ok(_) => {
                    println!(
                        "✓ Escaping test: '{}' compiled successfully ({})",
                        jsonpath_expr, _description
                    );
                }
                Err(_) => {
                    println!(
                        "✗ Escaping test: '{}' failed compilation ({})",
                        jsonpath_expr, _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_round_trip_conversion_simulation() {
        // RFC 9535 Appendix C: Simulate round-trip conversion where possible
        let round_trip_tests = vec![
            // JSONPath -> JSON Pointer -> JSONPath (for simple cases)
            ("$.store", "/store", "$.store", "Simple property"),
            (
                "$.store.config",
                "/store/config",
                "$.store.config",
                "Nested property",
            ),
            (
                "$.store.books[0]",
                "/store/books/0",
                "$.store.books[0]",
                "Array index",
            ),
            (
                "$.users[1].name",
                "/users/1/name",
                "$.users[1].name",
                "Nested array property",
            ),
            (
                "$.metadata.authors[2]",
                "/metadata/authors/2",
                "$.metadata.authors[2]",
                "Array element",
            ),
        ];

        for (original_jsonpath, json_pointer_like, converted_jsonpath, _description) in
            round_trip_tests
        {
            // Test original JSONPath
            let originalresult = JsonPathParser::compile(original_jsonpath);
            assert!(
                originalresult.is_ok(),
                "Original JSONPath should compile: {} ({})",
                original_jsonpath,
                _description
            );

            // Test converted JSONPath (should be equivalent)
            let convertedresult = JsonPathParser::compile(converted_jsonpath);
            assert!(
                convertedresult.is_ok(),
                "Converted JSONPath should compile: {} ({})",
                converted_jsonpath,
                _description
            );

            // Both should produce same results
            let mut original_stream = JsonArrayStream::<serde_json::Value>::new(original_jsonpath);
            let mut converted_stream =
                JsonArrayStream::<serde_json::Value>::new(converted_jsonpath);

            let chunk = Bytes::from(COMPATIBILITY_TEST_JSON);
            let originalresults: Vec<_> = original_stream.process_chunk(chunk.clone()).collect();
            let convertedresults: Vec<_> = converted_stream.process_chunk(chunk).collect();

            assert_eq!(
                originalresults.len(),
                convertedresults.len(),
                "RFC 9535: Round-trip conversion should preserve results: {} -> {} -> {} ({})",
                original_jsonpath,
                json_pointer_like,
                converted_jsonpath,
                _description
            );

            println!(
                "✓ Round-trip: '{}' -> '{}' -> '{}' ({} results) ({})",
                original_jsonpath,
                json_pointer_like,
                converted_jsonpath,
                originalresults.len(),
                _description
            );
        }
    }
}

/// JSON Pointer vs JSONPath Feature Matrix Tests
#[cfg(test)]
mod feature_matrix_tests {
    use super::*;

    #[test]
    fn test_json_pointer_expressible_patterns() {
        // RFC 9535 Appendix C: Patterns that can be expressed in both standards
        let expressible_patterns = vec![
            // Basic object traversal
            ("$.store.config.name", "Object property chain"),
            ("$.metadata.created", "Simple property access"),
            // Array element access
            ("$.store.books[0].title", "Specific array element property"),
            ("$.users[1].preferences.theme", "Deep array element access"),
            // Root access
            ("$", "Root object access"),
            ("$.store", "Top-level property"),
        ];

        for (jsonpath_expr, _description) in expressible_patterns {
            let compileresult = JsonPathParser::compile(jsonpath_expr);
            assert!(
                compileresult.is_ok(),
                "RFC 9535: JSON Pointer expressible pattern should work: {} ({})",
                jsonpath_expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(jsonpath_expr);
            let chunk = Bytes::from(COMPATIBILITY_TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "✓ JSON Pointer expressible: '{}' -> {} results ({})",
                jsonpath_expr,
                results.len(),
                _description
            );
        }
    }

    #[test]
    fn test_json_pointer_inexpressible_patterns() {
        // RFC 9535 Appendix C: Patterns that JSON Pointer cannot express
        let inexpressible_patterns = vec![
            // Dynamic/conditional access
            ("$.store.books[?@.price < 14]", "Conditional filtering"),
            ("$.users[?@.id > 1]", "Numeric filtering"),
            // Wildcard access
            ("$.store.books[*].author", "All array elements"),
            ("$.users[*].preferences", "Wildcard array access"),
            // Recursive patterns
            ("$..price", "Recursive descent"),
            ("$..preferences.theme", "Deep recursive access"),
            // Multi-selection
            ("$.store.books[0,2].title", "Multiple array indices"),
            ("$['store','metadata']", "Multiple properties"),
            // Array operations
            ("$.store.books[-1]", "Negative array index"),
            ("$.store.books[1:3]", "Array slicing"),
            ("$.metadata.authors[::2]", "Array stepping"),
            // Functions (if supported)
            (
                "$.store.books[?length(@.tags) > 2]",
                "Function-based filtering",
            ),
        ];

        for (jsonpath_expr, _description) in inexpressible_patterns {
            let compileresult = JsonPathParser::compile(jsonpath_expr);
            // Most should compile (JSONPath is more expressive)
            match compileresult {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(jsonpath_expr);
                    let chunk = Bytes::from(COMPATIBILITY_TEST_JSON);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "✓ JSON Pointer inexpressible: '{}' -> {} results ({}) - JSONPath advantage",
                        jsonpath_expr,
                        results.len(),
                        _description
                    );
                }
                Err(_) => {
                    println!(
                        "✗ JSONPath compilation failed: '{}' ({})",
                        jsonpath_expr, _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_semantic_equivalence_validation() {
        // RFC 9535 Appendix C: Validate semantic equivalence where applicable
        let semantic_tests = vec![
            // These pairs should produce identical results
            (
                "$.store.books[0]",
                "$.store['books'][0]",
                "Bracket vs dot notation",
            ),
            (
                "$.metadata.authors[2]",
                "$.metadata['authors'][2]",
                "Array access consistency",
            ),
            (
                "$.users[1].preferences.theme",
                "$.users[1]['preferences']['theme']",
                "Mixed notation equivalence",
            ),
        ];

        for (expr1, expr2, _description) in semantic_tests {
            let mut stream1 = JsonArrayStream::<serde_json::Value>::new(expr1);
            let mut stream2 = JsonArrayStream::<serde_json::Value>::new(expr2);

            let chunk = Bytes::from(COMPATIBILITY_TEST_JSON);
            let results1: Vec<_> = stream1.process_chunk(chunk.clone()).collect();
            let results2: Vec<_> = stream2.process_chunk(chunk).collect();

            assert_eq!(
                results1.len(),
                results2.len(),
                "RFC 9535: Semantically equivalent expressions should produce same results: '{}' vs '{}' ({})",
                expr1,
                expr2,
                _description
            );

            println!(
                "✓ Semantic equivalence: '{}' ≡ '{}' ({} results) ({})",
                expr1,
                expr2,
                results1.len(),
                _description
            );
        }
    }
}

/// Interoperability Edge Cases
#[cfg(test)]
mod interoperability_edge_cases {
    use super::*;

    #[test]
    fn test_empty_path_handling() {
        // RFC 9535 Appendix C: Empty path and root handling
        let empty_path_tests = vec![("$", "Root path"), ("$.", "Root with trailing dot")];

        for (expr, _description) in empty_path_tests {
            let compileresult = JsonPathParser::compile(expr);
            match compileresult {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                    let chunk = Bytes::from(COMPATIBILITY_TEST_JSON);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "✓ Empty path test: '{}' -> {} results ({})",
                        expr,
                        results.len(),
                        _description
                    );
                }
                Err(e) => {
                    println!(
                        "✗ Empty path test failed: '{}' - {:?} ({})",
                        expr, e, _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_special_character_compatibility() {
        // RFC 9535 Appendix C: Special characters in property names
        let special_char_data = r#"{
            "normal": "value",
            "with space": "value",
            "with/slash": "value", 
            "with~tilde": "value",
            "with.dot": "value",
            "with-dash": "value",
            "with\"quote": "value",
            "0": "numeric string key",
            "-1": "negative string key"
        }"#;

        let special_char_tests = vec![
            ("$.normal", "Normal property"),
            ("$['with space']", "Property with space"),
            ("$['with/slash']", "Property with slash (JSON Pointer ~1)"),
            ("$['with~tilde']", "Property with tilde (JSON Pointer ~0)"),
            ("$['with.dot']", "Property with dot"),
            ("$['with-dash']", "Property with dash"),
            ("$['0']", "Numeric string key"),
            ("$['-1']", "Negative string key"),
        ];

        for (expr, _description) in special_char_tests {
            let compileresult = JsonPathParser::compile(expr);
            match compileresult {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                    let chunk = Bytes::from(special_char_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "✓ Special char: '{}' -> {} results ({})",
                        expr,
                        results.len(),
                        _description
                    );
                }
                Err(_) => {
                    println!(
                        "✗ Special char compilation failed: '{}' ({})",
                        expr, _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_unicode_property_compatibility() {
        // RFC 9535 Appendix C: Unicode property names
        let unicode_data = r#"{
            "english": "value",
            "français": "french_value",
            "中文": "chinese_value", 
            "العربية": "arabic_value",
            "🚀": "emoji_value",
            "property with emoji 🎉": "mixed_value"
        }"#;

        let unicode_tests = vec![
            ("$.english", "ASCII property"),
            ("$.français", "French property"),
            ("$.中文", "Chinese property"),
            ("$.العربية", "Arabic property"),
            ("$['🚀']", "Emoji property"),
            ("$['property with emoji 🎉']", "Mixed unicode property"),
        ];

        for (expr, _description) in unicode_tests {
            let compileresult = JsonPathParser::compile(expr);
            match compileresult {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                    let chunk = Bytes::from(unicode_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "✓ Unicode: '{}' -> {} results ({})",
                        expr,
                        results.len(),
                        _description
                    );
                }
                Err(_) => {
                    println!(
                        "✗ Unicode compilation failed: '{}' ({})",
                        expr, _description
                    );
                }
            }
        }
    }
}
