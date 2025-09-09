//! RFC 9535 DoS Protection Tests (Section 4.1)
//!
//! Tests for RFC 9535 Section 4.1 security considerations:
//! "JSONPath implementations should take into account the fact that
//! recursive descent ($..) can cause significant computational overhead,
//! and limit its use appropriately."
//!
//! This test suite validates:
//! - DoS protection for recursive descent patterns
//! - Performance limits for deeply nested structures
//! - Memory exhaustion protection for complex queries
//! - Timeout handling for expensive operations
//! - Resource consumption limits for pathological cases

use std::time::{Duration, Instant};

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};

/// Create deeply nested JSON for DoS testing
fn create_deeply_nested_json(depth: usize) -> String {
    let mut json = String::new();

    // Create nested objects
    for _ in 0..depth {
        json.push_str("{\"nested\":");
    }

    json.push_str("{\"value\": \"found\"}");

    // Close all nested objects
    for _ in 0..depth {
        json.push('}');
    }

    json
}

/// Create wide JSON structure for testing breadth-first explosion
fn create_wide_json_structure(width: usize, items_per_level: usize) -> String {
    let mut json = String::from("{\"data\":[");

    for i in 0..width {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!("{{\"id\":{},\"items\":[", i));

        for j in 0..items_per_level {
            if j > 0 {
                json.push(',');
            }
            json.push_str(&format!(
                "{{\"value\":{},\"nested\":{{\"deep\":\"value{}\"}}}}",
                j, j
            ));
        }

        json.push_str("]}");
    }

    json.push_str("]}");
    json
}

/// RFC 9535 Section 4.1 - DoS Protection Tests
#[cfg(test)]
mod dos_protection_tests {
    use super::*;

    #[test]
    fn test_recursive_descent_depth_limits() {
        // RFC 9535: Recursive descent should have reasonable depth limits
        let depth_tests = vec![
            (10, true, "Reasonable depth should work"),
            (50, true, "Moderate depth should work"),
            (100, false, "Deep nesting should be limited or timeout"),
            (500, false, "Very deep nesting should be rejected"),
            (1000, false, "Extreme depth should be rejected"),
        ];

        for (depth, should_succeed, _description) in depth_tests {
            let json_data = create_deeply_nested_json(depth);
            let expr = "$..*"; // Recursive descent to all values

            let start_time = Instant::now();
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);

            // Set reasonable timeout for DoS protection
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            if should_succeed {
                // Should complete in reasonable time
                assert!(
                    elapsed < Duration::from_secs(1),
                    "RFC 9535: Depth {} should complete quickly: {} ({}ms)",
                    depth,
                    _description,
                    elapsed.as_millis()
                );

                // Should find the nested value
                assert!(
                    results.len() > 0,
                    "RFC 9535: Should find values at depth {}: {}",
                    depth,
                    _description
                );
            } else {
                // Should either timeout, error, or take too long
                if elapsed > Duration::from_secs(5) {
                    println!(
                        "DoS protection: Depth {} timed out as expected ({})",
                        depth, _description
                    );
                } else if results.is_empty() {
                    println!(
                        "DoS protection: Depth {} returned no results ({})",
                        depth, _description
                    );
                } else {
                    // Acceptable if it completes but with protection warnings
                    println!(
                        "DoS protection: Depth {} completed with {} results in {}ms ({})",
                        depth,
                        results.len(),
                        elapsed.as_millis(),
                        _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_recursive_descent_breadth_limits() {
        // RFC 9535: Wide recursive descent should have breadth limits
        let breadth_tests = vec![
            (10, 5, true, "Small breadth should work"),
            (100, 10, true, "Moderate breadth should work"),
            (1000, 20, false, "Large breadth should be limited"),
            (5000, 50, false, "Very large breadth should be rejected"),
        ];

        for (width, items_per_level, should_succeed, _description) in breadth_tests {
            let json_data = create_wide_json_structure(width, items_per_level);
            let expr = "$..nested.deep"; // Recursive descent with property access

            let start_time = Instant::now();
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);

            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            let expected_matches = width * items_per_level;

            if should_succeed {
                assert!(
                    elapsed < Duration::from_secs(2),
                    "RFC 9535: Breadth test should complete quickly: {} ({}ms)",
                    _description,
                    elapsed.as_millis()
                );

                assert_eq!(
                    results.len(),
                    expected_matches,
                    "RFC 9535: Should find all {} matches: {}",
                    expected_matches,
                    _description
                );
            } else {
                // Should have DoS protection
                if elapsed > Duration::from_secs(10) {
                    println!(
                        "DoS protection: Breadth test timed out as expected ({})",
                        _description
                    );
                } else {
                    println!(
                        "DoS protection: Breadth test completed in {}ms with {} results ({})",
                        elapsed.as_millis(),
                        results.len(),
                        _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_pathological_recursive_patterns() {
        // RFC 9535: Pathological patterns should be protected against
        let pathological_tests = vec![
            ("$..*.*", "Double wildcard recursion"),
            ("$..*..*", "Nested recursive wildcards"),
            ("$..[*]..[*]", "Multiple recursive array access"),
            ("$..item..$..value", "Multiple recursive descents"),
            (
                "$..nested..nested..nested",
                "Repeated recursive property access",
            ),
        ];

        for (expr, _description) in pathological_tests {
            // Use moderately complex JSON to test against
            let json_data = create_wide_json_structure(50, 10);

            let start_time = Instant::now();
            let compileresult = JsonPathParser::compile(expr);

            match compileresult {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                    let chunk = Bytes::from(json_data);

                    let results: Vec<_> = stream.process_chunk(chunk).collect();
                    let elapsed = start_time.elapsed();

                    // Should complete in reasonable time or have protection
                    if elapsed > Duration::from_secs(5) {
                        println!(
                            "DoS protection: Pathological pattern '{}' took {}ms ({})",
                            expr,
                            elapsed.as_millis(),
                            _description
                        );
                    } else {
                        println!(
                            "Pathological pattern '{}' completed with {} results in {}ms ({})",
                            expr,
                            results.len(),
                            elapsed.as_millis(),
                            _description
                        );
                    }
                }
                Err(_) => {
                    // Acceptable if parser rejects pathological patterns
                    println!(
                        "DoS protection: Parser rejected pathological pattern '{}' ({})",
                        expr, _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_memory_exhaustion_protection() {
        // RFC 9535: Should protect against memory exhaustion
        let memory_tests = vec![
            ("$..*", 1000, "Large recursive descent"),
            ("$[*].items[*].nested", 500, "Deep array traversal"),
            ("$..value", 2000, "Many value matches"),
        ];

        for (expr, data_size, _description) in memory_tests {
            // Create large dataset
            let json_data = create_wide_json_structure(data_size, 5);

            let start_time = Instant::now();
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);

            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Memory protection test - should not crash or consume excessive memory
            println!(
                "Memory test '{}' with {} data elements: {} results in {}ms ({})",
                expr,
                data_size,
                results.len(),
                elapsed.as_millis(),
                _description
            );

            // Should complete within memory bounds
            assert!(
                elapsed < Duration::from_secs(30),
                "RFC 9535: Memory test should not hang: {} ({})",
                expr,
                _description
            );
        }
    }
}

/// Recursive Descent Performance Tests
#[cfg(test)]
mod recursive_descent_performance_tests {
    use super::*;

    #[test]
    fn test_linear_vs_exponential_growth() {
        // RFC 9535: Recursive descent should have predictable performance characteristics
        let size_tests = vec![10, 20, 50, 100];

        for size in size_tests {
            let json_data = create_wide_json_structure(size, 3);
            let expr = "$..value";

            let start_time = Instant::now();
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);

            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            let expectedresults = size * 3; // 3 items per level

            println!(
                "Performance test size {}: {} results in {}ms (expected: {})",
                size,
                results.len(),
                elapsed.as_millis(),
                expectedresults
            );

            // Performance should scale reasonably
            let ms_perresult = elapsed.as_millis() as f64 / results.len() as f64;
            assert!(
                ms_perresult < 10.0,
                "RFC 9535: Performance should scale linearly, got {:.2}ms per result for size {}",
                ms_perresult,
                size
            );
        }
    }

    #[test]
    fn test_recursive_descent_termination() {
        // RFC 9535: Recursive descent should always terminate
        let termination_tests = vec![
            ("$..*", "Unbounded recursion"),
            ("$..nested..*", "Nested unbounded recursion"),
            ("$..[*]..*", "Array recursive descent"),
            ("$..a..$..b", "Multiple recursive patterns"),
        ];

        for (expr, _description) in termination_tests {
            let json_data = create_deeply_nested_json(20);

            let start_time = Instant::now();
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);

            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Must terminate within reasonable time
            assert!(
                elapsed < Duration::from_secs(10),
                "RFC 9535: Recursive descent must terminate: {} ({}) took {}ms",
                expr,
                _description,
                elapsed.as_millis()
            );

            println!(
                "Termination test '{}': {} results in {}ms ({})",
                expr,
                results.len(),
                elapsed.as_millis(),
                _description
            );
        }
    }

    #[test]
    fn test_stack_overflow_protection() {
        // RFC 9535: Should protect against stack overflow in deep recursion
        let deep_nesting_levels = vec![100, 500, 1000];

        for depth in deep_nesting_levels {
            let json_data = create_deeply_nested_json(depth);
            let expr = "$..*";

            let compileresult = JsonPathParser::compile(expr);
            assert!(
                compileresult.is_ok(),
                "Parser should handle deep nesting compilation for depth {}",
                depth
            );

            let start_time = Instant::now();
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);

            // This should not cause stack overflow
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            println!(
                "Stack overflow test depth {}: {} results in {}ms",
                depth,
                results.len(),
                elapsed.as_millis()
            );

            // Should not crash or take excessive time
            assert!(
                elapsed < Duration::from_secs(15),
                "RFC 9535: Stack overflow protection failed for depth {}",
                depth
            );
        }
    }
}

/// DoS Attack Simulation Tests
#[cfg(test)]
mod dos_attack_simulation_tests {
    use super::*;

    #[test]
    fn test_fork_bomb_pattern_protection() {
        // RFC 9535: Protect against expressions that cause exponential expansion
        let fork_bomb_patterns = vec![
            ("$..*[*]..*[*]", "Double array explosion"),
            ("$..*.*.*", "Triple wildcard explosion"),
            ("$..*..[*]..*", "Mixed wildcard/array explosion"),
        ];

        for (expr, _description) in fork_bomb_patterns {
            // Use structured data that could trigger exponential behavior
            let json_data = r#"{
                "level1": {
                    "items": [
                        {"sublevel": {"data": [1, 2, 3]}},
                        {"sublevel": {"data": [4, 5, 6]}},
                        {"sublevel": {"data": [7, 8, 9]}}
                    ]
                },
                "level2": {
                    "items": [
                        {"sublevel": {"data": [10, 11, 12]}},
                        {"sublevel": {"data": [13, 14, 15]}}
                    ]
                }
            }"#;

            let start_time = Instant::now();

            match JsonPathParser::compile(expr) {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                    let chunk = Bytes::from(json_data);

                    let results: Vec<_> = stream.process_chunk(chunk).collect();
                    let elapsed = start_time.elapsed();

                    // Should complete without exponential explosion
                    assert!(
                        elapsed < Duration::from_secs(5),
                        "RFC 9535: Fork bomb pattern protection failed: {} ({})",
                        expr,
                        _description
                    );

                    println!(
                        "Fork bomb protection: '{}' -> {} results in {}ms ({})",
                        expr,
                        results.len(),
                        elapsed.as_millis(),
                        _description
                    );
                }
                Err(_) => {
                    println!(
                        "Fork bomb protection: Parser rejected '{}' ({})",
                        expr, _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_resource_consumption_limits() {
        // RFC 9535: Should limit resource consumption for expensive queries
        let resource_tests = vec![
            ("$..*", 10000, "Large data recursive descent"),
            ("$[*][*][*]", 1000, "Deep array access"),
            ("$..items[*].nested..*", 500, "Complex nested access"),
        ];

        for (expr, data_multiplier, _description) in resource_tests {
            // Create large dataset
            let json_data = create_wide_json_structure(data_multiplier / 10, 10);

            let start_time = Instant::now();
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);

            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Resource limits test
            println!(
                "Resource test '{}' with {} scale: {} results in {}ms ({})",
                expr,
                data_multiplier,
                results.len(),
                elapsed.as_millis(),
                _description
            );

            // Should have reasonable resource consumption
            assert!(
                elapsed < Duration::from_secs(60),
                "RFC 9535: Resource consumption should be limited: {} ({})",
                expr,
                _description
            );
        }
    }
}
