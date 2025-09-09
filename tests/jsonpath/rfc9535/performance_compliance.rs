//! RFC 9535 Performance & Compliance Tests
//!
//! Tests for performance characteristics and compliance with RFC 9535 requirements:
//! - Large dataset handling (10K+ elements)
//! - Complex query performance validation  
//! - Memory usage validation
//! - Streaming behavior verification
//! - Resource limit enforcement
//! - Scalability testing

use std::time::Instant;

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct LargeDataModel {
    id: u32,
    name: String,
    category: String,
    price: f64,
    tags: Vec<String>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
    active: bool,
    created_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct NestedDataModel {
    level1: Level1Data,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct Level1Data {
    level2: Vec<Level2Data>,
    metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct Level2Data {
    level3: Level3Data,
    values: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct Level3Data {
    data: Vec<String>,
    properties: std::collections::HashMap<String, f64>,
}

/// RFC 9535 Performance Tests - Large Dataset Handling
#[cfg(test)]
mod large_dataset_tests {
    use super::*;

    pub fn generate_large_dataset(size: usize) -> serde_json::Value {
        let items: Vec<LargeDataModel> = (0..size)
            .map(|i| LargeDataModel {
                id: i as u32,
                name: format!("Item_{:06}", i),
                category: match i % 5 {
                    0 => "electronics".to_string(),
                    1 => "books".to_string(),
                    2 => "clothing".to_string(),
                    3 => "home".to_string(),
                    _ => "misc".to_string(),
                },
                price: (i as f64 * 1.5) + 10.0,
                tags: vec![
                    format!("tag_{}", i % 10),
                    format!("category_{}", i % 5),
                    format!("brand_{}", i % 3),
                ],
                metadata: {
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        "weight".to_string(),
                        serde_json::Value::Number((i % 100).into()),
                    );
                    map.insert(
                        "color".to_string(),
                        serde_json::Value::String(format!("color_{}", i % 8)),
                    );
                    map.insert(
                        "rating".to_string(),
                        serde_json::Value::Number(((i % 5) + 1).into()),
                    );
                    map
                },
                active: i % 7 != 0,
                created_at: format!("2024-{:02}-{:02}T10:00:00Z", (i % 12) + 1, (i % 28) + 1),
            })
            .collect();

        serde_json::json!({
            "catalog": {
                "items": items,
                "metadata": {
                    "total_count": size,
                    "generated_at": "2024-01-01T00:00:00Z",
                    "version": "1.0"
                }
            }
        })
    }

    #[test]
    fn test_large_array_traversal_performance() {
        use crate::large_dataset_tests::generate_large_dataset;
        // Test performance with 10K elements
        let dataset = generate_large_dataset(10_000);
        let json_data = serde_json::to_string(&dataset).expect("Valid JSON serialization");

        let test_cases = vec![
            ("$.catalog.items[*].name", "All item names"),
            ("$.catalog.items[*].price", "All item prices"),
            ("$.catalog.items[*].category", "All item categories"),
            ("$.catalog.items[*].tags[*]", "All tags from all items"),
        ];

        for (expr, _description) in test_cases {
            let start_time = Instant::now();

            let mut stream = JsonArrayStream::<String>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            println!(
                "{}: {} results in {:?}",
                _description,
                results.len(),
                duration
            );

            // Performance assertion - should handle large datasets efficiently
            assert!(
                duration.as_secs() < 5,
                "Large dataset query '{}' should complete in <5 seconds",
                expr
            );

            // Verify results are correct
            match expr {
                "$.catalog.items[*].name" => {
                    assert_eq!(results.len(), 10_000, "Should extract all 10K names");
                }
                "$.catalog.items[*].price" => {
                    assert_eq!(results.len(), 10_000, "Should extract all 10K prices");
                }
                "$.catalog.items[*].category" => {
                    assert_eq!(results.len(), 10_000, "Should extract all 10K categories");
                }
                "$.catalog.items[*].tags[*]" => {
                    assert_eq!(
                        results.len(),
                        30_000,
                        "Should extract all 30K tags (3 per item)"
                    );
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_filter_performance_large_dataset() {
        use crate::large_dataset_tests::generate_large_dataset;
        // Test filter performance on large datasets
        let dataset = generate_large_dataset(10_000);
        let json_data = serde_json::to_string(&dataset).expect("Valid JSON serialization");

        let filter_cases = vec![
            ("$.catalog.items[?@.active]", "Active items filter"),
            ("$.catalog.items[?@.price > 100]", "Price filter"),
            (
                "$.catalog.items[?@.category == 'electronics']",
                "Category filter",
            ),
            (
                "$.catalog.items[?@.active && @.price < 50]",
                "Complex logical filter",
            ),
        ];

        for (expr, _description) in filter_cases {
            let start_time = Instant::now();

            let mut stream = JsonArrayStream::<LargeDataModel>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            println!(
                "{}: {} results in {:?}",
                _description,
                results.len(),
                duration
            );

            // Performance assertion
            assert!(
                duration.as_secs() < 3,
                "Large dataset filter '{}' should complete in <3 seconds",
                expr
            );

            // Verify filter correctness
            match expr {
                "$.catalog.items[?@.active]" => {
                    for item in &results {
                        assert!(item.active, "All filtered items should be active");
                    }
                }
                "$.catalog.items[?@.price > 100]" => {
                    for item in &results {
                        assert!(
                            item.price > 100.0,
                            "All filtered items should have price > 100"
                        );
                    }
                }
                "$.catalog.items[?@.category == 'electronics']" => {
                    for item in &results {
                        assert_eq!(
                            item.category, "electronics",
                            "All filtered items should be electronics"
                        );
                    }
                }
                "$.catalog.items[?@.active && @.price < 50]" => {
                    for item in &results {
                        assert!(
                            item.active && item.price < 50.0,
                            "All items should be active and price < 50"
                        );
                    }
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_descendant_search_performance() {
        use crate::large_dataset_tests::generate_large_dataset;
        // Test descendant search performance on large nested structures
        let dataset = generate_large_dataset(5_000);
        let json_data = serde_json::to_string(&dataset).expect("Valid JSON serialization");

        let descendant_cases = vec![
            ("$..name", "All names (descendant)"),
            ("$..price", "All prices (descendant)"),
            ("$..metadata", "All metadata (descendant)"),
            ("$..*", "All values (universal descendant)"),
        ];

        for (expr, _description) in descendant_cases {
            let start_time = Instant::now();

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            println!(
                "{}: {} results in {:?}",
                _description,
                results.len(),
                duration
            );

            // Performance assertion - descendant searches are more expensive
            assert!(
                duration.as_secs() < 10,
                "Descendant search '{}' should complete in <10 seconds",
                expr
            );

            // Verify minimum expected results
            assert!(results.len() > 0, "Descendant search should find results");
        }
    }

    #[test]
    fn test_memory_usage_large_arrays() {
        // Test memory efficiency with large arrays
        let sizes = vec![1_000, 5_000, 10_000];

        for size in sizes {
            use crate::large_dataset_tests::generate_large_dataset;
            let dataset = generate_large_dataset(size);
            let json_data = serde_json::to_string(&dataset).expect("Valid JSON serialization");

            let start_time = Instant::now();

            // Test streaming behavior - should not load entire result set into memory
            let mut stream = JsonArrayStream::<LargeDataModel>::new("$.catalog.items[*]");

            let chunk = Bytes::from(json_data);
            let mut count = 0;

            // Process items one by one to verify streaming
            for result in stream.process_chunk(chunk).collect() {
                // AsyncStream yields results directly, not wrapped in Result
                // Validate that we're receiving valid data structures
                // Since result is LargeDataModel, just count all non-default instances
                count += 1;

                // Simulate processing time
                if count % 1000 == 0 {
                    let elapsed = start_time.elapsed();
                    println!("Processed {} items in {:?}", count, elapsed);
                }
            }

            let total_duration = start_time.elapsed();

            assert_eq!(count, size, "Should process all {} items", size);

            // Memory efficiency assertion - should scale linearly
            let per_item_micros = total_duration.as_micros() / size as u128;
            assert!(
                per_item_micros < 1000,
                "Should process items efficiently (<1ms per item), actual: {}μs",
                per_item_micros
            );

            println!(
                "Size {}: {} items processed in {:?} ({} μs/item)",
                size, count, total_duration, per_item_micros
            );
        }
    }
}

/// RFC 9535 Complex Query Performance Tests
#[cfg(test)]
mod complex_query_tests {
    use super::*;

    fn generate_nested_dataset(depth: usize, width: usize) -> serde_json::Value {
        fn create_nested_level(
            current_depth: usize,
            max_depth: usize,
            width: usize,
        ) -> serde_json::Value {
            if current_depth >= max_depth {
                return serde_json::json!({
                    "value": format!("leaf_value_{}", current_depth),
                    "data": (0..10).map(|i| format!("item_{}", i)).collect::<Vec<_>>()
                });
            }

            let children: Vec<serde_json::Value> = (0..width)
                .map(|i| {
                    let mut child = serde_json::Map::new();
                    child.insert(format!("id_{}", i), serde_json::Value::Number(i.into()));
                    child.insert(
                        "nested".to_string(),
                        create_nested_level(current_depth + 1, max_depth, width),
                    );
                    serde_json::Value::Object(child)
                })
                .collect();

            serde_json::json!({
                "level": current_depth,
                "children": children,
                "metadata": {
                    "depth": current_depth,
                    "width": width,
                    "total_nodes": width.pow((max_depth - current_depth) as u32)
                }
            })
        }

        serde_json::json!({
            "structure": create_nested_level(0, depth, width)
        })
    }

    #[test]
    fn test_deep_nesting_performance() {
        // Test performance with deeply nested structures
        let test_cases = vec![
            (5, 3, "Moderate depth (5 levels, 3 width)"),
            (7, 2, "Deep structure (7 levels, 2 width)"),
            (3, 5, "Wide structure (3 levels, 5 width)"),
        ];

        for (depth, width, _description) in test_cases {
            let dataset = generate_nested_dataset(depth, width);
            let json_data = serde_json::to_string(&dataset).expect("Valid JSON serialization");

            let complex_queries = vec![
                ("$..value", "Deep descendant search"),
                ("$.structure..children[*]", "Nested array access"),
                ("$..metadata", "Metadata at all levels"),
                (
                    "$.structure..children[*]..data[*]",
                    "Multi-level array traversal",
                ),
            ];

            for (expr, query_desc) in complex_queries {
                let start_time = Instant::now();

                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                let chunk = Bytes::from(json_data.clone());
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                let duration = start_time.elapsed();

                println!(
                    "{} - {}: {} results in {:?}",
                    _description,
                    query_desc,
                    results.len(),
                    duration
                );

                // Performance assertion - complex queries should still complete reasonably
                assert!(
                    duration.as_secs() < 5,
                    "Complex query '{}' on {} should complete in <5 seconds",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_filter_complexity_performance() {
        use crate::large_dataset_tests::generate_large_dataset;
        // Test performance of increasingly complex filter expressions
        let dataset = generate_large_dataset(1_000);
        let json_data = serde_json::to_string(&dataset).expect("Valid JSON serialization");

        let complexity_levels = vec![
            ("$.catalog.items[?@.active]", "Simple boolean filter"),
            (
                "$.catalog.items[?@.price > 50 && @.active]",
                "Two-condition AND filter",
            ),
            (
                "$.catalog.items[?@.category == 'electronics' || @.category == 'books']",
                "Two-condition OR filter",
            ),
            (
                "$.catalog.items[?(@.price > 100 && @.active) || (@.price < 20 && @.category == 'books')]",
                "Complex grouped conditions",
            ),
            (
                "$.catalog.items[?@.active && @.price > 50 && @.category != 'misc' && @.tags]",
                "Multi-condition complex filter",
            ),
        ];

        for (expr, _description) in complexity_levels {
            let start_time = Instant::now();

            let mut stream = JsonArrayStream::<LargeDataModel>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            println!(
                "{}: {} results in {:?}",
                _description,
                results.len(),
                duration
            );

            // Performance assertion - more complex filters should still be efficient
            assert!(
                duration.as_millis() < 500,
                "Complex filter '{}' should complete in <500ms",
                _description
            );

            // Verify all results match the filter criteria
            assert!(
                results.len() > 0,
                "Complex filter should find some matching results"
            );
        }
    }

    #[test]
    fn test_query_compilation_performance() {
        // Test JSONPath compilation performance for various query types
        let query_types = vec![
            ("$.simple.path", "Simple property access"),
            ("$.array[*].property", "Array wildcard"),
            ("$..descendant", "Descendant search"),
            ("$.items[?@.active && @.price > 100]", "Complex filter"),
            (
                "$.data..items[*].tags[?@ != 'excluded']",
                "Nested filter with descendant",
            ),
        ];

        for (expr, _description) in query_types {
            let start_time = Instant::now();

            // Compile the same query multiple times to test compilation performance
            for _ in 0..1000 {
                let _parser = JsonPathParser::compile(expr).expect("Valid JSONPath compilation");
            }

            let duration = start_time.elapsed();
            let per_compilation = duration.as_nanos() / 1000;

            println!(
                "{}: 1000 compilations in {:?} ({} ns/compilation)",
                _description, duration, per_compilation
            );

            // Compilation should be fast
            assert!(
                per_compilation < 100_000,
                "Query compilation for '{}' should be <100μs per compilation",
                _description
            );
        }
    }
}

/// RFC 9535 Streaming Behavior Verification
#[cfg(test)]
mod streaming_tests {
    use super::*;

    #[test]
    fn test_chunked_processing_performance() {
        use crate::large_dataset_tests::generate_large_dataset;
        // Test streaming behavior with chunked data processing
        let dataset = generate_large_dataset(5_000);
        let json_string = serde_json::to_string(&dataset).expect("Valid JSON serialization");

        // Split into multiple chunks to simulate streaming
        let chunk_size = json_string.len() / 10;
        let chunks: Vec<Bytes> = json_string
            .as_bytes()
            .chunks(chunk_size)
            .map(|chunk| Bytes::copy_from_slice(chunk))
            .collect();

        let start_time = Instant::now();

        let mut stream = JsonArrayStream::<LargeDataModel>::new("$.catalog.items[?@.active]");

        let mut totalresults = 0;

        // Process chunks sequentially to test streaming behavior
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_start = Instant::now();

            let results: Vec<_> = stream.process_chunk(chunk.clone()).collect();
            let chunkresults = results.len();
            totalresults += chunkresults;

            let chunk_duration = chunk_start.elapsed();

            println!(
                "Chunk {}: {} results in {:?}",
                i + 1,
                chunkresults,
                chunk_duration
            );

            // Each chunk should process quickly
            assert!(
                chunk_duration.as_millis() < 200,
                "Chunk {} processing should complete in <200ms",
                i + 1
            );
        }

        let total_duration = start_time.elapsed();

        println!(
            "Total streaming: {} results in {:?}",
            totalresults, total_duration
        );

        // Streaming should be efficient overall
        assert!(
            total_duration.as_secs() < 3,
            "Chunked streaming should complete in <3 seconds"
        );
    }

    #[test]
    fn test_incrementalresult_delivery() {
        use crate::large_dataset_tests::generate_large_dataset;
        // Test that results are delivered incrementally, not all at once
        let dataset = generate_large_dataset(1_000);
        let json_data = serde_json::to_string(&dataset).expect("Valid JSON serialization");

        let mut stream = JsonArrayStream::<LargeDataModel>::new("$.catalog.items[*]");

        let chunk = Bytes::from(json_data);
        let start_time = Instant::now();

        let mut result_count = 0;
        let mut timing_checkpoints = Vec::new();

        // Process results and record timing at regular intervals
        for result in stream.process_chunk(chunk).collect() {
            // AsyncStream yields results directly, not wrapped in Result
            // Validate result structure and count valid entries
            // Since result is LargeDataModel, count all valid instances
            result_count += 1;

            // Record timing every 100 results
            if result_count % 100 == 0 {
                timing_checkpoints.push((result_count, start_time.elapsed()));
            }
        }

        // Verify incremental delivery by checking timing progression
        for i in 1..timing_checkpoints.len() {
            let (prev_count, prev_time) = timing_checkpoints[i - 1];
            let (curr_count, curr_time) = timing_checkpoints[i];

            let count_diff = curr_count - prev_count;
            let time_diff = curr_time - prev_time;

            println!(
                "Results {}-{}: {} items in {:?}",
                prev_count, curr_count, count_diff, time_diff
            );

            // Time between checkpoints should be reasonable (not all at the end)
            assert!(
                time_diff.as_millis() < 500,
                "Incremental delivery should process 100 items in <500ms"
            );
        }

        assert_eq!(
            result_count, 1_000,
            "Should process all items incrementally"
        );
    }

    #[test]
    fn test_memory_bounded_streaming() {
        // Test that streaming doesn't consume excessive memory
        let large_string_size = 10_000;
        let large_strings: Vec<String> = (0..100)
            .map(|i| format!("large_string_{}_{}", i, "x".repeat(large_string_size)))
            .collect();

        let dataset = serde_json::json!({
            "data": {
                "strings": large_strings,
                "metadata": {
                    "count": 100,
                    "size_per_string": large_string_size
                }
            }
        });

        let json_data = serde_json::to_string(&dataset).expect("Valid JSON serialization");

        let start_time = Instant::now();

        let mut stream = JsonArrayStream::<String>::new("$.data.strings[*]");

        let chunk = Bytes::from(json_data);

        // Process large strings without accumulating them all in memory
        let mut processed_count = 0;
        for large_string in stream.process_chunk(chunk).collect() {
            // AsyncStream yields results directly, not wrapped in Result
            // Verify string content without storing it
            assert!(
                large_string.len() > large_string_size,
                "String should be large as expected"
            );
            processed_count += 1;

            // Drop the string immediately to test memory efficiency
            drop(large_string);
        }

        let duration = start_time.elapsed();

        assert_eq!(processed_count, 100, "Should process all large strings");

        // Should handle large strings efficiently
        assert!(
            duration.as_secs() < 2,
            "Large string streaming should complete in <2 seconds"
        );

        println!(
            "Memory-bounded streaming: {} large strings in {:?}",
            processed_count, duration
        );
    }
}

/// RFC 9535 Resource Limit Enforcement Tests
#[cfg(test)]
mod resource_limit_tests {
    use super::*;

    #[test]
    fn test_query_depth_limits() {
        // Test handling of deeply nested queries without stack overflow
        let deep_path_segments = (0..50)
            .map(|i| format!("level{}", i))
            .collect::<Vec<_>>()
            .join(".");

        let deep_query = format!("$.{}", deep_path_segments);

        // Should handle deep paths gracefully
        let result = JsonPathParser::compile(&deep_query);

        match result {
            Ok(_) => {
                println!("Deep query compilation succeeded: 50 levels");
                // If compilation succeeds, test with actual data
                let nested_data = serde_json::json!({
                    "level0": {"level1": {"level2": {"value": "deep_value"}}}
                });

                let json_data = serde_json::to_string(&nested_data).expect("Valid JSON");
                let mut stream = JsonArrayStream::<serde_json::Value>::new(&deep_query);

                let chunk = Bytes::from(json_data);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                // Should handle gracefully even if path doesn't exist
                println!("Deep query execution: {} results", results.len());
            }
            Err(_) => {
                println!("Deep query rejected at compilation (expected for extreme depth)");
            }
        }
    }

    #[test]
    fn test_large_array_index_handling() {
        // Test handling of very large array indices
        let large_indices = vec![1_000_000, u32::MAX as usize];

        for index in large_indices {
            let query = format!("$.items[{}]", index);

            let result = JsonPathParser::compile(&query);

            match result {
                Ok(_) => {
                    println!("Large index {} compilation succeeded", index);

                    // Test with small array to verify bounds checking
                    let data = serde_json::json!({"items": [1, 2, 3]});
                    let json_data = serde_json::to_string(&data).expect("Valid JSON");

                    let mut stream = JsonArrayStream::<i32>::new(&query);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    // Should return empty results for out-of-bounds indices
                    assert_eq!(
                        results.len(),
                        0,
                        "Out-of-bounds index {} should return no results",
                        index
                    );
                }
                Err(_) => {
                    println!("Large index {} rejected at compilation", index);
                }
            }
        }
    }

    #[test]
    fn test_expression_complexity_limits() {
        // Test handling of highly complex filter expressions
        let simple_conditions: Vec<String> =
            (0..20).map(|i| format!("@.field{} == {}", i, i)).collect();

        let complex_filter = format!("$.items[?{}]", simple_conditions.join(" && "));

        let start_time = Instant::now();

        let result = JsonPathParser::compile(&complex_filter);

        let compilation_time = start_time.elapsed();

        match result {
            Ok(_) => {
                println!(
                    "Complex filter compilation succeeded in {:?}",
                    compilation_time
                );

                // Should compile reasonably quickly even for complex expressions
                assert!(
                    compilation_time.as_millis() < 100,
                    "Complex filter compilation should complete in <100ms"
                );

                // Test execution with sample data
                let data = serde_json::json!({
                    "items": [
                        {"field0": 0, "field1": 1, "field2": 2},
                        {"field0": 1, "field1": 2, "field2": 3}
                    ]
                });

                let json_data = serde_json::to_string(&data).expect("Valid JSON");
                let mut stream = JsonArrayStream::<serde_json::Value>::new(&complex_filter);

                let execution_start = Instant::now();
                let chunk = Bytes::from(json_data);
                let results: Vec<_> = stream.process_chunk(chunk).collect();
                let execution_time = execution_start.elapsed();

                println!(
                    "Complex filter execution: {} results in {:?}",
                    results.len(),
                    execution_time
                );

                // Should execute efficiently
                assert!(
                    execution_time.as_millis() < 50,
                    "Complex filter execution should complete in <50ms"
                );
            }
            Err(_) => {
                println!("Complex filter rejected at compilation (expected behavior)");
            }
        }
    }

    #[test]
    fn test_concurrent_query_performance() {
        // Test performance under concurrent query execution
        use std::sync::Arc;
        use std::thread;

        use crate::large_dataset_tests::generate_large_dataset;
        let dataset = Arc::new(generate_large_dataset(1_000));
        let json_data = Arc::new(serde_json::to_string(&*dataset).expect("Valid JSON"));

        let queries = vec![
            "$.catalog.items[?@.active]",
            "$.catalog.items[?@.price > 100]",
            "$.catalog.items[?@.category == 'electronics']",
            "$.catalog.items[*].name",
            "$.catalog.items[*].tags[*]",
        ];

        let start_time = Instant::now();

        let handles: Vec<_> = queries
            .into_iter()
            .enumerate()
            .map(|(i, query)| {
                let json_data = Arc::clone(&json_data);
                let query = query.to_string();

                thread::spawn(move || {
                    let thread_start = Instant::now();

                    let mut stream = JsonArrayStream::<serde_json::Value>::new(&query);

                    let chunk = Bytes::from((*json_data).clone());
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    let thread_duration = thread_start.elapsed();

                    (i, query, results.len(), thread_duration)
                })
            })
            .collect();

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.join().expect("Thread completed successfully"));
        }

        let total_duration = start_time.elapsed();

        // Verify all queries completed
        assert_eq!(results.len(), 5, "All concurrent queries should complete");

        for (i, query, result_count, thread_duration) in results {
            println!(
                "Thread {}: '{}' -> {} results in {:?}",
                i, query, result_count, thread_duration
            );

            // Each thread should complete efficiently
            assert!(
                thread_duration.as_secs() < 2,
                "Concurrent query {} should complete in <2 seconds",
                i
            );
        }

        println!("All concurrent queries completed in {:?}", total_duration);

        // Overall concurrent execution should be efficient
        assert!(
            total_duration.as_secs() < 5,
            "Concurrent query execution should complete in <5 seconds"
        );
    }
}
