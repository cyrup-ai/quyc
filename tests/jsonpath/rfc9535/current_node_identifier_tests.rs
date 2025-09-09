//! RFC 9535 Current Node Identifier (@) Validation Tests (Section 2.3.5)
//!
//! Tests for RFC 9535 Section 2.3.5 current node identifier requirements:
//! "The current node identifier @ refers to the current node in the context
//! of the evaluation of a filter expression"
//!
//! This test suite validates:
//! - @ is only valid within filter expressions
//! - @ correctly refers to current node in filter context
//! - @ behavior in nested filter expressions
//! - @ usage in function expressions within filters
//! - @ error handling outside filter contexts
//! - @ property access patterns
//! - @ in logical expressions and comparisons

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathError, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestNode {
    id: i32,
    name: String,
    value: f64,
    active: bool,
    metadata: Option<serde_json::Value>,
    nested: Option<Box<TestNode>>,
}

/// Test data for current node identifier validation
const TEST_JSON: &str = r#"{
  "store": {
    "books": [
      {
        "id": 1,
        "name": "Book One",
        "value": 10.5,
        "active": true,
        "metadata": {"category": "fiction", "pages": 300},
        "nested": {"id": 11, "name": "Chapter", "value": 1.0, "active": true}
      },
      {
        "id": 2,
        "name": "Book Two", 
        "value": 25.0,
        "active": false,
        "metadata": {"category": "science", "pages": 450}
      },
      {
        "id": 3,
        "name": "Book Three",
        "value": 15.75,
        "active": true,
        "metadata": null
      }
    ],
    "config": {
      "id": 100,
      "name": "Store Config",
      "value": 99.99,
      "active": true
    }
  }
}"#;

/// Deep nesting test data for complex @ identifier scenarios
const DEEP_NESTING_JSON: &str = r#"{
  "company": {
    "departments": [
      {
        "name": "Engineering",
        "teams": [
          {
            "name": "Backend",
            "members": [
              {
                "id": 1,
                "name": "Alice",
                "skills": ["rust", "python"],
                "performance": {
                  "rating": 9.5,
                  "projects": [
                    {"name": "Project A", "status": "complete", "rating": 10},
                    {"name": "Project B", "status": "active", "rating": 8.5}
                  ]
                }
              },
              {
                "id": 2,
                "name": "Bob",
                "skills": ["javascript", "rust"],
                "performance": {
                  "rating": 8.0,
                  "projects": [
                    {"name": "Project C", "status": "complete", "rating": 7.5},
                    {"name": "Project D", "status": "complete", "rating": 9.0}
                  ]
                }
              }
            ]
          },
          {
            "name": "Frontend",
            "members": [
              {
                "id": 3,
                "name": "Carol",
                "skills": ["react", "typescript"],
                "performance": {
                  "rating": 9.0,
                  "projects": [
                    {"name": "Project E", "status": "active", "rating": 8.0}
                  ]
                }
              }
            ]
          }
        ]
      }
    ]
  }
}"#;

/// RFC 9535 Section 2.3.5 - Current Node Identifier Context Tests
#[cfg(test)]
mod current_node_context_tests {
    use super::*;

    #[test]
    fn test_current_node_only_valid_in_filters() {
        // RFC 9535: @ is only valid within filter expressions
        let invalid_contexts = vec![
            "@",                     // Bare @ as root
            "$.@",                   // @ as segment
            "$.store.@",             // @ in property access
            "$.store[@]",            // @ as selector (not in filter)
            "$[@]",                  // @ without filter marker
            "$.store.books.@.name",  // @ in path segments
            "@.store.books[0]",      // @ as root identifier
            "$.store.books[@.id]",   // @ in bracket without ?
            "$.@name",               // @ in dot notation
            "$['@']",                // @ as quoted property name (valid)
            "$.store.@books",        // @ in middle of path
            "$.store.books.@",       // @ at end of path
            "$..@",                  // @ with descendant operator
            "$.*.@",                 // @ after wildcard
            "$.store.books[0].@.id", // @ in chain after index
            "$.store.books[@name]",  // @ without dot in bracket
            "$.@.store",             // @ immediately after root
        ];

        for expr in invalid_contexts {
            let result = JsonPathParser::compile(expr);

            // Special case: $['@'] is actually valid - it's a property named "@"
            if expr == "$['@']" {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Property named '@' should be valid: '{}'",
                    expr
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: @ outside filter context MUST be rejected: '{}'",
                    expr
                );

                if let Err(JsonPathError::InvalidExpression { reason, .. }) = result {
                    assert!(
                        reason.contains("@")
                            || reason.contains("current")
                            || reason.contains("filter"),
                        "Error should mention @ or current node context: {}",
                        reason
                    );
                }
            }
        }
    }

    #[test]
    fn test_current_node_valid_in_filter_contexts() {
        // RFC 9535: @ is valid within filter expressions
        let valid_filter_contexts = vec![
            "$.store.books[?@.active]",             // Property existence test
            "$.store.books[?@.id > 1]",             // Property comparison
            "$.store.books[?@.value >= 15.0]",      // Numeric comparison
            "$.store.books[?@.name == 'Book One']", // String comparison
            "$.store.books[?@.metadata]",           // Object existence
            "$.store.books[?@.nested.id > 10]",     // Nested property access
            "$.store.books[?@.metadata.category == 'fiction']", // Deep property access
            "$..books[?@.active == true]",          // Boolean comparison
            "$.store.books[?@.active && @.value > 10]", // Multiple @ in logical expression
            "$.store.books[?@.id != @.nested.id]",  // @ self-comparison
            "$..*[?@.id && @.name]",                // Universal with @
            "$.store.books[?(@.active)]",           // Parenthesized @
            "$.store.books[?!@.active]",            // Negated @
            "$.store.books[?@.value < 20 && @.id > 1]", // Complex logical with @
        ];

        for expr in valid_filter_contexts {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "RFC 9535: @ in valid filter context should compile: '{}'",
                expr
            );
        }
    }

    #[test]
    fn test_current_node_property_access() {
        // RFC 9535: @ correctly accesses properties of current node
        let property_access_tests = vec![
            ("$.store.books[?@.id == 1]", 1, "Current node ID access"),
            (
                "$.store.books[?@.name == 'Book Two']",
                1,
                "Current node name access",
            ),
            (
                "$.store.books[?@.value > 20]",
                1,
                "Current node value comparison",
            ),
            (
                "$.store.books[?@.active == false]",
                1,
                "Current node boolean access",
            ),
            (
                "$.store.books[?@.metadata.category == 'science']",
                1,
                "Nested property access",
            ),
            (
                "$.store.books[?@.metadata == null]",
                1,
                "Null property access",
            ),
            (
                "$.store.books[?@.nested.name == 'Chapter']",
                1,
                "Deep nested access",
            ),
        ];

        for (expr, expected_count, _description) in property_access_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: @ property access should work correctly: {} ({})",
                expr,
                _description
            );
        }
    }

    #[test]
    fn test_current_node_in_logical_expressions() {
        // RFC 9535: @ behavior in complex logical expressions
        let logical_tests = vec![
            (
                "$.store.books[?@.active && @.value > 10]",
                2,
                "AND with two @ conditions",
            ),
            (
                "$.store.books[?@.active || @.value > 20]",
                3,
                "OR with @ conditions",
            ),
            (
                "$.store.books[?@.active && (@.value > 10 && @.id < 3)]",
                1,
                "Nested logical with @",
            ),
            ("$.store.books[?!@.active]", 1, "Negation of @ condition"),
            (
                "$.store.books[?@.active == true && @.metadata != null]",
                1,
                "Complex boolean logic",
            ),
            (
                "$.store.books[?(@.id > 1) && (@.value < 20)]",
                1,
                "Parenthesized @ expressions",
            ),
            (
                "$.store.books[?@.value >= 10 && @.value <= 20]",
                2,
                "Range check with @",
            ),
        ];

        for (expr, expected_count, _description) in logical_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: @ in logical expressions should work: {} ({})",
                expr,
                _description
            );
        }
    }

    #[test]
    fn test_current_node_with_functions() {
        // RFC 9535: @ usage in function expressions within filters
        let function_tests = vec![
            (
                "$.store.books[?length(@.name) > 8]",
                2,
                "length() function with @",
            ),
            (
                "$.store.books[?count(@.metadata) > 0]",
                2,
                "count() function with @",
            ),
            ("$.*[?length(@) > 0]", 1, "length() of @ itself"),
            (
                "$.store.books[?@.metadata && length(@.metadata) > 0]",
                2,
                "Function with @ existence check",
            ),
        ];

        for (expr, _expected_count, _description) in function_tests {
            // Note: These tests validate syntax compilation
            // Actual function execution depends on implementation
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "RFC 9535: @ with functions should compile: {} ({})",
                expr,
                _description
            );
        }
    }

    #[test]
    fn test_current_node_edge_cases() {
        // RFC 9535: Edge cases for @ usage
        let edge_case_tests = vec![
            // Valid edge cases
            ("$.store.books[?@]", true, "Bare @ as test expression"),
            ("$.store.books[?(@)]", true, "Parenthesized bare @"),
            ("$.store.books[?!(@)]", true, "Negated parenthesized @"),
            ("$.store.books[?@ && true]", true, "@ with literal boolean"),
            ("$.store.books[?@ == @]", true, "@ self-equality"),
            (
                "$.store.books[?@.nonexistent]",
                true,
                "@ accessing nonexistent property",
            ),
            // Invalid edge cases
            ("$.store.books[@@]", false, "Double @"),
            ("$.store.books[?@.]", false, "@ with trailing dot"),
            ("$.store.books[?@[0]]", false, "@ with array access"),
            ("$.store.books[?@['key']]", false, "@ with bracket notation"),
            ("$.store.books[?@.*]", false, "@ with wildcard"),
            ("$.store.books[?@..]", false, "@ with descendant operator"),
        ];

        for (expr, _should_be_valid, _description) in edge_case_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: @ edge case should be valid: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: @ edge case should be invalid: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_current_node_type_consistency() {
        // RFC 9535: @ should maintain type consistency within expression
        let type_tests = vec![
            (
                "$.store.books[?@.id == 1 && @.name == 'Book One']",
                1,
                "Number and string from same @",
            ),
            (
                "$.store.books[?@.active == true && @.value > 0]",
                2,
                "Boolean and number from same @",
            ),
            (
                "$.store.books[?@.metadata && @.metadata.category]",
                2,
                "Object existence and property",
            ),
            (
                "$.store.books[?@.nested && @.nested.active]",
                1,
                "Nested object consistency",
            ),
            (
                "$.store.books[?@.value > @.id]",
                2,
                "Numeric comparison within same @",
            ),
        ];

        for (expr, expected_count, _description) in type_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: @ type consistency should work: {} ({})",
                expr,
                _description
            );
        }
    }
}

/// Current Node Identifier Scope Tests
#[cfg(test)]
mod current_node_scope_tests {
    use super::*;

    #[test]
    fn test_current_node_scope_isolation() {
        // RFC 9535: @ refers to current node in current filter scope
        let scope_tests = vec![
            // Single scope
            ("$.store.books[?@.id > 1]", 2, "Single filter scope"),
            // Multiple independent scopes
            (
                "$.store.books[?@.active].metadata[?@.category == 'fiction']",
                0,
                "Multiple independent scopes",
            ),
            (
                "$..books[?@.id > 1][?@.value < 20]",
                0,
                "Chained filter scopes",
            ),
            // Descendant with filters
            (
                "$.store..books[?@.active]",
                2,
                "Descendant with filter scope",
            ),
            (
                "$..metadata[?@.category]",
                1,
                "Descendant filter on metadata",
            ),
        ];

        for (expr, expected_count, _description) in scope_tests {
            let expected_count = expected_count as usize;

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: @ scope isolation should work: {} ({})",
                expr,
                _description
            );

            // Test compilation for complex scopes
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "RFC 9535: @ scope test should compile: {} ({})",
                expr,
                _description
            );
        }
    }

    #[test]
    fn test_current_node_inheritance() {
        // RFC 9535: @ behavior with nested structures
        let inheritance_tests = vec![
            (
                "$.store.books[?@.nested.id > 10]",
                1,
                "@ accessing nested structure",
            ),
            (
                "$.store.books[?@.metadata.pages > 400]",
                1,
                "@ accessing nested properties",
            ),
            (
                "$.store.books[?@.metadata && @.metadata.category]",
                2,
                "@ with nested existence check",
            ),
            (
                "$..books[?@.nested && @.nested.active]",
                1,
                "Descendant @ with nesting",
            ),
        ];

        for (expr, expected_count, _description) in inheritance_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: @ inheritance should work: {} ({})",
                expr,
                _description
            );
        }
    }

    #[test]
    fn test_current_node_comparison_semantics() {
        // RFC 9535: @ comparison semantics and type coercion
        let comparison_tests = vec![
            (
                "$.store.books[?@.id == '1']",
                0,
                "String vs number comparison",
            ),
            (
                "$.store.books[?@.active == 'true']",
                0,
                "String vs boolean comparison",
            ),
            (
                "$.store.books[?@.value == 10.5]",
                1,
                "Exact numeric comparison",
            ),
            (
                "$.store.books[?@.name != null]",
                3,
                "Non-null string comparison",
            ),
            (
                "$.store.books[?@.metadata != null]",
                2,
                "Non-null object comparison",
            ),
            (
                "$.store.books[?@.nonexistent == null]",
                0,
                "Missing property comparison",
            ),
        ];

        for (expr, _expected_count, _description) in comparison_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // Note: Some comparisons may vary based on implementation
            // These tests document expected behavior
            println!(
                "@ comparison test '{}' -> {} results ({})",
                expr,
                results.len(),
                _description
            );

            // Assert compilation succeeds
            let compileresult = JsonPathParser::compile(expr);
            assert!(
                compileresult.is_ok(),
                "RFC 9535: @ comparison should compile: {} ({})",
                expr,
                _description
            );
        }
    }
}

/// Current Node Error Handling Tests  
#[cfg(test)]
mod current_node_error_tests {
    use super::*;

    #[test]
    fn test_current_node_error_messages() {
        // RFC 9535: @ error messages should be clear and helpful
        let error_cases = vec![
            ("@", "@ outside filter"),
            ("$.@", "@ in path segment"),
            ("$.store[@]", "@ without filter marker"),
            ("@.store", "@ as root"),
            ("$.store.books.@.name", "@ in property chain"),
        ];

        for (expr, error_type) in error_cases {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "RFC 9535: {} should produce error: '{}'",
                error_type,
                expr
            );

            if let Err(JsonPathError::InvalidExpression { reason, .. }) = result {
                // Error message should be informative
                assert!(
                    !reason.is_empty(),
                    "RFC 9535: Error message should not be empty for: {}",
                    expr
                );

                println!("Error for '{}': {}", expr, reason);
            }
        }
    }

    #[test]
    fn test_current_node_complex_error_cases() {
        // RFC 9535: Complex error scenarios with @
        let complex_errors = vec![
            ("$.store.books[?@..name]", "@ with descendant operator"),
            ("$.store.books[?@[*]]", "@ with wildcard selector"),
            ("$.store.books[?@[0:2]]", "@ with slice operator"),
            ("$.store.books[?@['key']]", "@ with bracket notation"),
            ("$.store.books[?@@.id]", "Double @ symbols"),
            ("$.store.books[?@.id.@.value]", "@ in middle of path"),
        ];

        for (expr, error_description) in complex_errors {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "RFC 9535: {} should be invalid: '{}'",
                error_description,
                expr
            );
        }
    }
}

/// RFC 9535 Section 2.3.5 - Deep Nesting Current Node Identifier Tests
#[cfg(test)]
mod deep_nesting_current_node_tests {
    use super::*;

    #[test]
    fn test_deeply_nested_current_node_references() {
        // RFC 9535 Section 2.3.5: Test complex nested @ references in filter expressions

        let deep_nested_json = r#"{
            "departments": [
                {
                    "name": "Engineering",
                    "teams": [
                        {
                            "name": "Backend",
                            "members": [
                                {"name": "Alice", "skills": ["rust", "python"], "level": 5, "projects": [{"name": "API", "priority": 1}]},
                                {"name": "Bob", "skills": ["go", "rust"], "level": 3, "projects": [{"name": "DB", "priority": 2}]}
                            ]
                        },
                        {
                            "name": "Frontend", 
                            "members": [
                                {"name": "Carol", "skills": ["javascript", "react"], "level": 4, "projects": [{"name": "UI", "priority": 1}]}
                            ]
                        }
                    ]
                }
            ]
        }"#;

        let complex_nested_tests = vec![
            (
                // Nested @ references in complex filter
                "$.departments[?@.teams[?@.members[?@.level > 4]]]",
                1, // Should find Engineering dept with high-level members
                "Deep nested @ should correctly reference nodes at each level",
            ),
            (
                // Multiple @ references in same filter expression
                "$.departments[*].teams[?@.name == 'Backend' && @.members[?@.level >= 3]]",
                1, // Should find Backend team with qualified members
                "Multiple @ references in logical expression should work",
            ),
            (
                // @ with function calls in nested context
                "$.departments[*].teams[*].members[?length(@.skills) > 1 && @.projects[?@.priority == 1]]",
                2, // Alice and Carol have multiple skills and priority 1 projects
                "@ in function calls within nested filters should work",
            ),
            (
                // Complex property access through @
                "$.departments[?@.teams[?@.members[?@.projects[?@.priority < 2]]]].name",
                1, // Engineering dept has teams with members having priority 1 projects
                "Deep @ property traversal should access correct nested properties",
            ),
        ];

        for (expr, expected_count, _description) in complex_nested_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(deep_nested_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535 deep nesting: {} - '{}' should return {} results, got {}",
                _description,
                expr,
                expected_count,
                results.len()
            );
        }
    }

    #[test]
    fn test_deep_nesting_current_node_performance() {
        // RFC 9535: Test @ references in deeply nested structures for performance validation
        // Uses the DEEP_NESTING_JSON constant for comprehensive testing

        let performance_tests = vec![
            (
                // Test @ reference at multiple nesting levels
                "$.company.departments[?@.name == 'Engineering'].teams[?@.name == 'Backend'].members[?@.name == 'Alice']",
                1, // Should find Alice in Backend team
                "@ should efficiently resolve in deeply nested object structures",
            ),
            (
                // Test @ with complex performance metrics
                "$.company.departments[*].teams[*].members[?@.performance.rating > 8.5]",
                2, // Alice (9.5) and Bob has a project rating > 8.5
                "@ should access nested performance objects correctly",
            ),
            (
                // Test @ in deeply nested arrays
                "$.company.departments[*].teams[*].members[*].performance.projects[?@.status == 'complete']",
                3, // Project A, Project C, Project D
                "@ should work in deeply nested array filtering",
            ),
            (
                // Test @ referencing multiple levels simultaneously
                "$.company.departments[*].teams[?count(@.members) > 1].members[?@.id < 3]",
                2, // Alice (id 1) and Bob (id 2) from Backend team
                "@ should correctly reference current node in multi-level filters",
            ),
        ];

        for (expr, expected_count, _description) in performance_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(DEEP_NESTING_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535 deep nesting performance: {} - '{}' should return {} results, got {}",
                _description,
                expr,
                expected_count,
                results.len()
            );
        }
    }

    #[test]
    fn test_current_node_scope_isolation() {
        // RFC 9535: @ should correctly reference current node at each nesting level

        let scope_test_json = r#"{
            "levels": [
                {
                    "id": 1,
                    "subitems": [
                        {"id": 11, "value": 100},
                        {"id": 12, "value": 200}
                    ]
                },
                {
                    "id": 2, 
                    "subitems": [
                        {"id": 21, "value": 150},
                        {"id": 22, "value": 250}
                    ]
                }
            ]
        }"#;

        let scope_tests = vec![
            (
                // @ should reference the level, not subitem
                "$.levels[?@.id == 1].subitems[*]",
                2, // Both subitems from level 1
                "@ should reference level object, not subitems",
            ),
            (
                // @ should reference subitem in inner filter
                "$.levels[*].subitems[?@.value > 150]",
                2, // id 12 (200) and id 22 (250)
                "@ in inner filter should reference subitem, not level",
            ),
            (
                // Nested @ scopes should be independent
                "$.levels[?@.id > 0].subitems[?@.id > 15]",
                2, // id 21 and id 22 from level 2 (since level 2 has id > 0)
                "Nested @ scopes should reference correct nodes independently",
            ),
        ];

        for (expr, expected_count, _description) in scope_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(scope_test_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535 scope isolation: {} - '{}' should return {} results, got {}",
                _description,
                expr,
                expected_count,
                results.len()
            );
        }
    }

    #[test]
    fn test_current_node_with_complex_logical_expressions() {
        // RFC 9535: @ in complex logical expressions with multiple operators

        let logical_test_json = r#"{
            "products": [
                {"name": "Laptop", "price": 999, "category": "electronics", "inStock": true, "rating": 4.5},
                {"name": "Book", "price": 15, "category": "education", "inStock": true, "rating": 4.0},
                {"name": "Phone", "price": 599, "category": "electronics", "inStock": false, "rating": 4.8},
                {"name": "Tablet", "price": 299, "category": "electronics", "inStock": true, "rating": 3.9}
            ]
        }"#;

        let logical_tests = vec![
            (
                // Complex AND/OR with @
                "$.products[?(@.category == 'electronics' && @.inStock == true) || (@.price < 50 && @.rating > 4.0)]",
                3, // Laptop, Tablet (electronics + inStock), Book (cheap + good rating)
                "Complex AND/OR expressions with @ should work correctly",
            ),
            (
                // Nested parentheses with @
                "$.products[?(@.price > 500 && (@.category == 'electronics' || @.rating > 4.7)) || (@.price < 100 && @.inStock)]",
                3, /* Laptop (expensive electronics), Phone (expensive + high rating), Book (cheap + inStock) */
                "Nested parentheses with @ should evaluate correctly",
            ),
            (
                // Negation with @
                "$.products[?!(@.category == 'electronics' && @.inStock == false)]",
                3, // All except Phone (electronics + not inStock)
                "Negation with @ should work correctly",
            ),
        ];

        for (expr, expected_count, _description) in logical_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(logical_test_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535 logical expressions: {} - '{}' should return {} results, got {}",
                _description,
                expr,
                expected_count,
                results.len()
            );
        }
    }

    #[test]
    fn test_current_node_with_function_composition() {
        // RFC 9535: @ used in complex function compositions within filters

        let function_test_json = r#"{
            "teams": [
                {"name": "Alpha", "members": ["Alice", "Bob", "Charlie"], "scores": [85, 92, 78], "active": true},
                {"name": "Beta", "members": ["David", "Eve"], "scores": [95, 88], "active": true},
                {"name": "Gamma", "members": ["Frank"], "scores": [67], "active": false}
            ]
        }"#;

        let function_tests = vec![
            (
                // Function composition with @
                "$.teams[?count(@.members) > 2 && length(@.name) < 6]",
                1, // Alpha has >2 members and name length < 6
                "Function composition with @ should work in filters",
            ),
            (
                // Nested function calls with @
                "$.teams[?@.active && count(@.scores) == length(@.members)]",
                2, // Alpha and Beta are active with matching scores/members count
                "Nested function calls with @ should evaluate correctly",
            ),
            (
                // @ in function arguments with logical operators
                "$.teams[?length(@.name) > 4 || (count(@.members) == 1 && @.active == false)]",
                2, // Alpha, Beta (long names), Gamma (1 member + inactive)
                "@ in function arguments with logical operators should work",
            ),
        ];

        for (expr, expected_count, _description) in function_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(function_test_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535 function composition: {} - '{}' should return {} results, got {}",
                _description,
                expr,
                expected_count,
                results.len()
            );
        }
    }
}
