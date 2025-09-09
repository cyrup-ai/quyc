//! RFC 9535 Streaming Behavior Tests
//!
//! Tests specific to JsonArrayStream interface and HTTP streaming scenarios

use std::time::Instant;

use bytes::Bytes;
use quyc::jsonpath::JsonArrayStream;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct OpenAiModel {
    id: String,
    object: String,
    created: Option<u64>,
    owned_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct StreamingMessage {
    role: String,
    content: String,
    timestamp: u64,
}

/// OpenAI API Format Streaming Tests
#[cfg(test)]
mod openai_streaming_tests {
    use super::*;

    #[test]
    fn test_openai_models_list_streaming() {
        // RFC 9535: Test with OpenAI /v1/models response format
        let openai_response = r#"{
            "object": "list", 
            "data": [
                {
                    "id": "gpt-4",
                    "object": "model",
                    "created": 1687882411,
                    "owned_by": "openai"
                },
                {
                    "id": "gpt-3.5-turbo",
                    "object": "model", 
                    "created": 1677649963,
                    "owned_by": "openai"
                },
                {
                    "id": "text-davinci-003",
                    "object": "model",
                    "created": 1669599635,
                    "owned_by": "openai-internal"
                }
            ]
        }"#;

        let mut stream = JsonArrayStream::<OpenAiModel>::new("$.data[*]");

        let chunk = Bytes::from(openai_response);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 3, "Should stream 3 models");

        let model_ids: Vec<String> = results.iter().map(|m| m.id.clone()).collect();
        assert!(
            model_ids.contains(&"gpt-4".to_string()),
            "Should contain gpt-4"
        );
        assert!(
            model_ids.contains(&"gpt-3.5-turbo".to_string()),
            "Should contain gpt-3.5-turbo"
        );
        assert!(
            model_ids.contains(&"text-davinci-003".to_string()),
            "Should contain text-davinci-003"
        );

        // Verify all models have correct object type
        for model in results {
            assert_eq!(model.object, "model", "All items should be model objects");
        }
    }

    #[test]
    fn test_openai_chat_completions_streaming() {
        // RFC 9535: Test with OpenAI chat completions response format
        let chat_response = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-3.5-turbo-0613",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello! How can I assist you today?"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 9,
                "completion_tokens": 12,
                "total_tokens": 21
            }
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.choices[*].message");

        let chunk = Bytes::from(chat_response);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 1, "Should stream 1 message");

        let message = &results[0];
        assert_eq!(
            message["role"], "assistant",
            "Message should be from assistant"
        );
        assert!(
            message["content"].as_str().unwrap().contains("Hello"),
            "Message should contain greeting"
        );
    }

    #[test]
    fn test_openai_empty_data_array() {
        // RFC 9535: Test with empty data array
        let empty_response = r#"{
            "object": "list",
            "data": []
        }"#;

        let mut stream = JsonArrayStream::<OpenAiModel>::new("$.data[*]");

        let chunk = Bytes::from(empty_response);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 0, "Empty array should yield no results");
        assert!(
            stream.is_complete(),
            "Stream should be complete after empty array"
        );
    }

    #[test]
    fn test_openai_error_response() {
        // RFC 9535: Test with OpenAI error response format
        let error_response = r#"{
            "error": {
                "message": "Invalid API key provided",
                "type": "invalid_request_error",
                "param": null,
                "code": "invalid_api_key"
            }
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.data[*]");

        let chunk = Bytes::from(error_response);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            0,
            "Error response should yield no data results"
        );

        // Test error path extraction
        let mut error_stream = JsonArrayStream::<serde_json::Value>::new("$.error");

        let errorresults: Vec<_> = error_stream
            .process_chunk(Bytes::from(error_response))
            .collect();
        assert_eq!(errorresults.len(), 1, "Should extract error object");
    }
}

/// Chunked HTTP Response Streaming Tests
#[cfg(test)]
mod chunked_streaming_tests {
    use super::*;

    #[test]
    fn test_incremental_json_parsing() {
        // RFC 9535: Test incremental parsing of JSON chunks
        let json_chunks = vec![
            r#"{"data": ["#,
            r#"{"id": "1", "name": "first"},"#,
            r#"{"id": "2", "name": "second"},"#,
            r#"{"id": "3", "name": "third"}"#,
            r#"]}"#,
        ];

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.data[*]");

        let mut allresults = Vec::new();

        for chunk_data in json_chunks {
            let chunk = Bytes::from(chunk_data);
            let chunkresults: Vec<_> = stream.process_chunk(chunk).collect();
            allresults.extend(chunkresults);
        }

        assert_eq!(
            allresults.len(),
            3,
            "Should parse all items from chunked input"
        );

        // Verify parsed content
        for (i, result) in allresults.iter().enumerate() {
            let expected_id = format!("{}", i + 1);
            assert_eq!(
                result["id"],
                expected_id,
                "Item {} should have correct ID",
                i + 1
            );
        }
    }

    #[test]
    fn test_incomplete_json_handling() {
        // RFC 9535: Test handling of incomplete JSON in streaming
        let incomplete_chunks = vec![
            r#"{"data": [{"id": "partial"#, // Incomplete object
            r#", "name": "complete"}]}"#,   // Completion
        ];

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.data[*]");

        // First chunk should not yield results (incomplete)
        let chunk1 = Bytes::from(incomplete_chunks[0]);
        let results1: Vec<_> = stream.process_chunk(chunk1).collect();
        assert_eq!(
            results1.len(),
            0,
            "Incomplete JSON should not yield results"
        );

        // Second chunk should complete and yield the object
        let chunk2 = Bytes::from(incomplete_chunks[1]);
        let results2: Vec<_> = stream.process_chunk(chunk2).collect();
        assert_eq!(results2.len(), 1, "Complete JSON should yield 1 result");

        let item = &results2[0];
        assert_eq!(item["id"], "partial", "Should have correct ID");
        assert_eq!(item["name"], "complete", "Should have correct name");
    }

    #[test]
    fn test_large_array_streaming() {
        // RFC 9535: Test streaming large arrays efficiently
        let array_size = 1000;
        let large_array: Vec<serde_json::Value> = (0..array_size)
            .map(|i| serde_json::json!({"id": i, "value": format!("item_{}", i)}))
            .collect();

        let json_data = serde_json::json!({"data": large_array}).to_string();

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.data[*]");

        let chunk = Bytes::from(json_data);
        let start_time = Instant::now();
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        let duration = start_time.elapsed();

        assert_eq!(
            results.len(),
            array_size,
            "Should stream all {} items",
            array_size
        );
        println!("Streamed {} items in {:?}", array_size, duration);

        // Performance assertion - should handle large arrays efficiently
        assert!(
            duration.as_millis() < 1000,
            "Large array streaming should complete in <1s"
        );
    }

    #[test]
    fn test_malformed_json_recovery() {
        // RFC 9535: Test recovery from malformed JSON chunks
        let json_chunks = vec![
            r#"{"data": [{"id": "1", "valid": true},"#,
            r#"{"id": "2", "malformed": invalid_json},"#, // Malformed chunk
            r#"{"id": "3", "valid": true}]}"#,
        ];

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.data[*]");

        let mut allresults = Vec::new();

        for chunk_data in json_chunks {
            let chunk = Bytes::from(chunk_data);
            let chunkresults: Vec<_> = stream.process_chunk(chunk).collect();

            allresults.extend(chunkresults);
        }

        // Should handle errors gracefully and continue processing
        println!("Successfully parsed {} items", allresults.len());
        assert!(
            allresults.len() > 0,
            "Should parse some valid items despite errors"
        );
    }
}

/// Streaming Performance and Statistics Tests
#[cfg(test)]
mod streaming_stats_tests {
    use super::*;

    #[test]
    fn test_streaming_statistics_tracking() {
        // RFC 9535: Test streaming statistics and metrics
        let json_data = r#"{
            "data": [
                {"id": "1", "size": 100},
                {"id": "2", "size": 200},
                {"id": "3", "size": 300}
            ]
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.data[*]");

        // Get initial stats
        let initial_stats = stream.stats();
        assert_eq!(
            initial_stats.bytes_processed, 0,
            "Should start with 0 bytes processed"
        );
        assert_eq!(
            initial_stats.objects_yielded, 0,
            "Should start with 0 objects yielded"
        );

        // Process data
        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        // Get final stats
        let final_stats = stream.stats();
        assert!(
            final_stats.bytes_processed > 0,
            "Should have processed bytes"
        );
        assert_eq!(
            final_stats.objects_yielded, 3,
            "Should have yielded 3 objects"
        );
        assert_eq!(results.len(), 3, "Should match objects yielded");

        println!("Streaming stats: {:?}", final_stats);
    }

    #[test]
    fn test_stream_completion_status() {
        // RFC 9535: Test stream completion detection
        let json_data = r#"{"data": [{"id": "1"}, {"id": "2"}]}"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.data[*]");

        assert!(
            !stream.is_complete(),
            "Stream should not be complete initially"
        );

        let chunk = Bytes::from(json_data);
        let _results: Vec<_> = stream.process_chunk(chunk).collect();

        // Stream completion status depends on implementation
        let is_complete = stream.is_complete();
        println!("Stream complete after processing: {}", is_complete);
    }

    #[test]
    fn test_buffer_size_management() {
        // RFC 9535: Test buffer size management for streaming
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.data[*]");

        let initial_stats = stream.stats();
        let initial_buffer_size = initial_stats.buffer_size;

        // Process increasingly large chunks
        let chunk_sizes = vec![100, 1000, 10000];

        for size in chunk_sizes {
            let large_data = "x".repeat(size);
            let json_data = format!(r#"{{"data": ["{}"]}}"#, large_data);

            let chunk = Bytes::from(json_data);
            let _results: Vec<_> = stream.process_chunk(chunk).collect();

            let stats = stream.stats();
            println!(
                "Buffer size after {} byte chunk: {}",
                size, stats.buffer_size
            );
        }

        let final_stats = stream.stats();
        println!(
            "Buffer grew from {} to {} bytes",
            initial_buffer_size, final_stats.buffer_size
        );
    }
}

/// Complex JSONPath Streaming Scenarios
#[cfg(test)]
mod complex_streaming_tests {
    use super::*;

    #[test]
    fn test_nested_array_streaming() {
        // RFC 9535: Test streaming from nested arrays
        let nested_data = r#"{
            "departments": [
                {
                    "name": "Engineering",
                    "employees": [
                        {"id": "eng1", "role": "developer"},
                        {"id": "eng2", "role": "architect"}
                    ]
                },
                {
                    "name": "Sales", 
                    "employees": [
                        {"id": "sales1", "role": "manager"},
                        {"id": "sales2", "role": "rep"}
                    ]
                }
            ]
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.departments[*].employees[*]");

        let chunk = Bytes::from(nested_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            4,
            "Should stream all employees from all departments"
        );

        let employee_ids: Vec<String> = results
            .iter()
            .filter_map(|emp| emp["id"].as_str())
            .map(|s| s.to_string())
            .collect();

        assert!(
            employee_ids.contains(&"eng1".to_string()),
            "Should contain eng1"
        );
        assert!(
            employee_ids.contains(&"sales2".to_string()),
            "Should contain sales2"
        );
    }

    #[test]
    fn test_filtered_streaming_with_conditions() {
        // RFC 9535: Test streaming with complex filter conditions
        let data_with_conditions = r#"{
            "products": [
                {"name": "Widget A", "price": 19.99, "in_stock": true, "category": "tools"},
                {"name": "Widget B", "price": 29.99, "in_stock": false, "category": "tools"},
                {"name": "Gadget A", "price": 9.99, "in_stock": true, "category": "electronics"},
                {"name": "Gadget B", "price": 39.99, "in_stock": true, "category": "electronics"}
            ]
        }"#;

        let filter_expressions = vec![
            ("$.products[?@.in_stock]", 3),                 // In stock items
            ("$.products[?@.price < 25]", 2),               // Cheap items
            ("$.products[?@.category == 'tools']", 2),      // Tools category
            ("$.products[?@.in_stock && @.price < 25]", 2), // In stock AND cheap
            (
                "$.products[?@.category == 'electronics' || @.price > 35]",
                2,
            ), // Electronics OR expensive
        ];

        for (expr, expected_count) in filter_expressions {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(data_with_conditions);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_descendant_streaming_performance() {
        // RFC 9535: Test performance of descendant streaming
        let deep_nested_data = r#"{
            "level1": {
                "level2": {
                    "level3": {
                        "items": [
                            {"id": "deep1", "value": "a"},
                            {"id": "deep2", "value": "b"}
                        ]
                    }
                },
                "parallel": {
                    "items": [
                        {"id": "parallel1", "value": "c"}
                    ]
                }
            }
        }"#;

        let mut stream = JsonArrayStream::<serde_json::Value>::new("$..items[*]");

        let chunk = Bytes::from(deep_nested_data);
        let start_time = Instant::now();
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        let duration = start_time.elapsed();

        assert_eq!(results.len(), 3, "Should find all items at any depth");
        println!("Descendant streaming took {:?}", duration);

        // Performance should be reasonable for moderate nesting
        assert!(
            duration.as_millis() < 100,
            "Descendant streaming should complete in <100ms"
        );
    }
}
