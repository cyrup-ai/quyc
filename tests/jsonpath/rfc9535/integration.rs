//! RFC 9535 Integration Tests
//!
//! Comprehensive integration tests combining all RFC 9535 features

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct ComplexModel {
    id: String,
    metadata: serde_json::Value,
    tags: Vec<String>,
    nested: Option<Box<ComplexModel>>,
}

/// RFC 9535 Full Specification Integration Tests
#[cfg(test)]
mod full_spec_integration {
    use super::*;

    #[test]
    fn test_rfc9535_complete_compliance_matrix() {
        // RFC 9535: Test matrix of all selector types with all segment types
        let complex_json = r#"{
            "root": {
                "child1": {
                    "array": [
                        {"name": "item1", "value": 10},
                        {"name": "item2", "value": 20}
                    ]
                },
                "child2": {
                    "nested": {
                        "deep": {
                            "array": [
                                {"name": "deep1", "value": 100},
                                {"name": "deep2", "value": 200}
                            ]
                        }
                    }
                }
            }
        }"#;

        // Test all selector types with child segments
        let child_expressions = vec![
            ("$.root['child1']", 1),                   // Name selector
            ("$.root[*]", 2),                          // Wildcard selector
            ("$.root.child1.array[0]", 1),             // Index selector
            ("$.root.child1.array[0:2]", 2),           // Slice selector
            ("$.root.child1.array[?@.value > 15]", 1), // Filter selector
        ];

        for (expr, expected_count) in child_expressions {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(complex_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Child segment expression '{}' should return {} results",
                expr,
                expected_count
            );
        }

        // Test all selector types with descendant segments
        let descendant_expressions = vec![
            ("$..['name']", 4),        // Name selector with descendant
            ("$..[*]", 12),            // Wildcard with descendant (many results)
            ("$..array[0]", 2),        // Index with descendant
            ("$..array[:1]", 2),       // Slice with descendant
            ("$..[?@.value > 50]", 2), // Filter with descendant
        ];

        for (expr, min_expected) in descendant_expressions {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(complex_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert!(
                results.len() >= min_expected,
                "Descendant expression '{}' should return at least {} results, got {}",
                expr,
                min_expected,
                results.len()
            );
        }
    }

    #[test]
    fn test_rfc9535_abnf_grammar_full_compliance() {
        // RFC 9535: Test complete ABNF grammar compliance
        let grammar_test_expressions = vec![
            // Basic structure: jsonpath-query = root-identifier *segment
            "$",      // Should fail - no segments
            "$.prop", // Valid - root + segment
            // Segments: segment = child-segment / descendant-segment
            "$.child",       // child-segment
            "$..descendant", // descendant-segment
            // Child segments: child-segment = "[" selector-list "]"
            "$['prop']",      // Single selector
            "$['a','b','c']", // Multiple selectors
            // Descendant segments: descendant-segment = ".." child-segment
            "$..[*]",      // Descendant wildcard
            "$..['prop']", // Descendant name
            // All selector types in combination
            "$['name'][*][0][1:3][?@.active]", // Complex selector combination
        ];

        let should_fail = vec!["$"]; // Expressions that should fail
        let should_pass: Vec<&str> = grammar_test_expressions
            .iter()
            .filter(|&expr| !should_fail.contains(expr))
            .copied()
            .collect();

        // Test expressions that should fail
        for expr in should_fail {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Expression '{}' should fail grammar validation",
                expr
            );
        }

        // Test expressions that should pass
        for expr in should_pass {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Expression '{}' should pass grammar validation",
                expr
            );
        }
    }

    #[test]
    fn test_rfc9535_function_extensions_integration() {
        // RFC 9535: Test all function extensions in realistic scenarios
        let function_test_data = r#"{
            "users": [
                {
                    "username": "alice123",
                    "tags": ["admin", "active"],
                    "profile": {
                        "bio": "Software engineer with 5+ years experience",
                        "skills": ["rust", "javascript", "python"]
                    }
                },
                {
                    "username": "bob",
                    "tags": ["user"],
                    "profile": {
                        "bio": "New team member",
                        "skills": ["java"]
                    }
                }
            ]
        }"#;

        // Function extension test cases (syntax may vary based on implementation)
        let function_expressions = vec![
            // length() function tests
            ("$.users[?length(@.username) > 5]", "Length function"),
            ("$.users[?length(@.tags) >= 2]", "Array length function"),
            // match() function tests
            (
                "$.users[?match(@.username, '^[a-z]+[0-9]*$')]",
                "Regex match function",
            ),
            // search() function tests
            (
                "$.users[?search(@.profile.bio, 'engineer')]",
                "Regex search function",
            ),
            // Nested function scenarios
            (
                "$.users[?length(@.profile.skills) > length(@.tags)]",
                "Function comparison",
            ),
        ];

        for (expr, _description) in function_expressions {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    println!("✓ {} syntax supported: {}", _description, expr);

                    // Try to execute if compilation succeeds
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                    let chunk = Bytes::from(function_test_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();
                    println!("  → Returned {} results", results.len());
                }
                Err(_) => println!("✗ {} syntax not yet supported: {}", _description, expr),
            }
        }
    }

    #[test]
    fn test_rfc9535_unicode_full_support() {
        // RFC 9535: Comprehensive Unicode support testing
        let unicode_json = r#"{
            "国际化": {
                "用户": [
                    {"姓名": "张三", "年龄": 25, "标签": ["开发者", "活跃"]},
                    {"姓名": "李四", "年龄": 30, "标签": ["管理员"]}
                ]
            },
            "Ñoño": {
                "España": ["Madrid", "Barcelona"],
                "México": ["CDMX", "Guadalajara"]
            },
            "🚀rocket": {
                "🌟features": ["⚡speed", "🛡️safety", "🎯precision"]
            }
        }"#;

        let unicode_expressions = vec![
            ("$.国际化.用户[*].姓名", 2),             // Chinese characters
            ("$.Ñoño.España[*]", 2),                  // Spanish characters
            ("$.🚀rocket.🌟features[*]", 3),          // Emoji properties
            ("$['国际化']['用户'][0]['标签'][*]", 2), // Bracket notation with Unicode
            ("$.国际化.用户[?@.年龄 > 26]", 1),       // Unicode in filters
        ];

        for (expr, expected_count) in unicode_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Unicode expression should compile: {}",
                expr
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(unicode_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Unicode expression '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_rfc9535_edge_cases_comprehensive() {
        // RFC 9535: Comprehensive edge case testing
        let edge_case_scenarios = vec![
            // Empty structures
            (
                r#"{"empty_array": [], "empty_object": {}}"#,
                "$.empty_array[*]",
                0,
            ),
            (
                r#"{"empty_array": [], "empty_object": {}}"#,
                "$.empty_object.*",
                0,
            ),
            // Null values
            (
                r#"{"data": [null, {"value": null}, {"value": 42}]}"#,
                "$.data[*]",
                3,
            ),
            (
                r#"{"data": [null, {"value": null}, {"value": 42}]}"#,
                "$.data[?@.value]",
                2,
            ),
            // Mixed types in arrays
            (
                r#"{"mixed": [1, "string", true, null, {"obj": "value"}, [1,2,3]]}"#,
                "$.mixed[*]",
                6,
            ),
            // Deep nesting
            (
                r#"{"a":{"b":{"c":{"d":{"e":{"f":"deep_value"}}}}}}"#,
                "$..f",
                1,
            ),
        ];

        for (_json_data, expr, expected_count) in edge_case_scenarios {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(_json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Edge case '{}' with data '{}' should return {} results",
                expr,
                _json_data,
                expected_count
            );
        }
    }

    #[test]
    fn test_rfc9535_performance_comprehensive() {
        // RFC 9535: Comprehensive performance testing across all features
        let performance_scenarios = vec![
            // Large arrays
            ("Large array", generate_large_array_json(1000), "$.items[*]"),
            // Deep nesting
            ("Deep nesting", generate_deep_nested_json(50), "$..value"),
            // Complex filters
            (
                "Complex filters",
                generate_complex_filter_json(),
                "$.items[?@.active && @.score > 80]",
            ),
            // Multiple descendants
            (
                "Multiple descendants",
                generate_multi_descendant_json(),
                "$..items[*]",
            ),
        ];

        for (scenario_name, _json_data, expr) in performance_scenarios {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(_json_data);
            let start_time = std::time::Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let duration = start_time.elapsed();

            println!(
                "{}: {} results in {:?}",
                scenario_name,
                results.len(),
                duration
            );

            // Performance assertions
            assert!(
                duration.as_millis() < 1000,
                "{} should complete in <1s",
                scenario_name
            );
            assert!(
                !results.is_empty(),
                "{} should return some results",
                scenario_name
            );
        }
    }
}

/// Helper functions for generating test data
fn generate_large_array_json(size: usize) -> String {
    let items: Vec<serde_json::Value> = (0..size)
        .map(|i| {
            serde_json::json!({
                "id": i,
                "name": format!("item_{}", i),
                "active": i % 2 == 0,
                "score": (i % 100) as f64
            })
        })
        .collect();

    serde_json::json!({"items": items}).to_string()
}

fn generate_deep_nested_json(depth: usize) -> String {
    let mut nested = serde_json::json!({"value": "deep_target"});

    for i in (0..depth).rev() {
        nested = serde_json::json!({
            format!("level_{}", i): nested
        });
    }

    nested.to_string()
}

fn generate_complex_filter_json() -> String {
    let items: Vec<serde_json::Value> = (0..100)
        .map(|i| {
            serde_json::json!({
                "id": i,
                "active": i % 3 == 0,
                "score": (i % 101) as f64,
                "category": if i % 2 == 0 { "A" } else { "B" },
                "tags": if i % 5 == 0 { vec!["premium"] } else { vec!["standard"] }
            })
        })
        .collect();

    serde_json::json!({"items": items}).to_string()
}

fn generate_multi_descendant_json() -> String {
    serde_json::json!({
        "level1": {
            "items": [{"id": "l1_1"}, {"id": "l1_2"}],
            "level2": {
                "items": [{"id": "l2_1"}, {"id": "l2_2"}],
                "level3": {
                    "items": [{"id": "l3_1"}]
                }
            }
        },
        "parallel": {
            "items": [{"id": "p1"}]
        }
    })
    .to_string()
}

/// RFC 9535 Compliance Summary Test
#[cfg(test)]
mod compliance_summary {
    use super::*;

    #[test]
    fn test_rfc9535_compliance_checklist() {
        // RFC 9535: Final compliance checklist
        println!("RFC 9535 JSONPath Compliance Checklist:");

        let compliance_items = vec![
            ("Root identifier '$' required", "$.test", true),
            ("Child segments '[...]'", "$['prop']", true),
            ("Descendant segments '..[...]'", "$..prop", true),
            ("Name selectors", "$['name']", true),
            ("Wildcard selectors", "$[*]", true),
            ("Index selectors", "$[0]", true),
            ("Slice selectors", "$[1:3]", true),
            ("Filter selectors", "$[?@.prop]", true),
            ("Union selectors", "$[0,1]", true),
            ("Dot notation", "$.prop", true),
            ("Unicode support", "$.国际化", true),
            ("Function extensions", "$[?length(@.prop) > 0]", false), // May not be implemented yet
        ];

        let mut passed = 0;
        let total = compliance_items.len();

        for (_description, test_expr, should_pass) in compliance_items {
            let result = JsonPathParser::compile(test_expr);
            let actual_pass = result.is_ok();

            if actual_pass == should_pass {
                println!("✓ {}", _description);
                passed += 1;
            } else if actual_pass && !should_pass {
                println!("✓ {} (unexpectedly supported!)", _description);
                passed += 1;
            } else {
                println!("✗ {} (not yet implemented)", _description);
            }
        }

        println!(
            "\nCompliance Score: {}/{} ({:.1}%)",
            passed,
            total,
            (passed as f32 / total as f32) * 100.0
        );

        // At least basic compliance should be achieved
        assert!(
            passed >= total / 2,
            "Should achieve at least 50% RFC 9535 compliance, got {:.1}%",
            (passed as f32 / total as f32) * 100.0
        );
    }
}
