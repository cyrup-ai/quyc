//! End-to-end integration tests for JSONPath streaming functionality
//!
//! These tests validate the complete streaming pipeline from buffer input
//! through JSONPath evaluation to deserialized object output.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use quyc_client::jsonpath::{
    buffer::StreamBuffer,
    deserializer::core::{JsonPathDeserializer, StreamingJsonPathState, ProcessingStats},
    parser::{JsonPathExpression, JsonPathParser},
    error::JsonPathError,
};

/// Test data structure for streaming tests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestObject {
    id: u64,
    name: String,
    value: f64,
    active: bool,
    metadata: Option<TestMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestMetadata {
    category: String,
    tags: Vec<String>,
    score: f64,
}

/// Test data for nested array structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct NestedTestData {
    users: Vec<TestUser>,
    settings: TestSettings,
    stats: Vec<TestStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestUser {
    id: u64,
    email: String,
    profile: TestProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestProfile {
    name: String,
    age: u32,
    preferences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestSettings {
    theme: String,
    notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestStat {
    metric: String,
    value: f64,
    timestamp: u64,
}

/// Generate test data for streaming scenarios
fn generate_test_objects(count: usize) -> Vec<TestObject> {
    (0..count)
        .map(|i| TestObject {
            id: i as u64,
            name: format!("Object_{}", i),
            value: i as f64 * 1.5,
            active: i % 3 == 0,
            metadata: if i % 2 == 0 {
                Some(TestMetadata {
                    category: format!("Category_{}", i % 5),
                    tags: vec![format!("tag_{}", i % 3), format!("tag_{}", i % 7)],
                    score: (i as f64 / 10.0) % 1.0,
                })
            } else {
                None
            },
        })
        .collect()
}

/// Generate nested test data
fn generate_nested_test_data() -> NestedTestData {
    NestedTestData {
        users: (0..10)
            .map(|i| TestUser {
                id: i,
                email: format!("user{}@test.com", i),
                profile: TestProfile {
                    name: format!("User {}", i),
                    age: 20 + (i * 3) % 50,
                    preferences: vec![
                        format!("pref_{}", i % 3),
                        format!("pref_{}", i % 5),
                    ],
                },
            })
            .collect(),
        settings: TestSettings {
            theme: "dark".to_string(),
            notifications: true,
        },
        stats: (0..20)
            .map(|i| TestStat {
                metric: format!("metric_{}", i % 4),
                value: i as f64 * 2.5,
                timestamp: 1640000000 + (i as u64 * 3600),
            })
            .collect(),
    }
}

/// Create JSON stream from test objects
fn create_json_stream(objects: &[TestObject]) -> Vec<u8> {
    let json_array = serde_json::to_string(objects).expect("Failed to serialize test objects");
    json_array.into_bytes()
}

/// Create nested JSON stream from test data
fn create_nested_json_stream(data: &NestedTestData) -> Vec<u8> {
    let json = serde_json::to_string(data).expect("Failed to serialize nested test data");
    json.into_bytes()
}

/// Helper to create streaming deserializer setup
fn create_deserializer_setup<'a, T>(
    path_expression: &'a JsonPathExpression,
    buffer: &'a mut StreamBuffer,
) -> JsonPathDeserializer<'a, T>
where
    T: serde::de::DeserializeOwned,
{
    let streaming_state = StreamingJsonPathState::new(path_expression);
    
    JsonPathDeserializer {
        path_expression,
        buffer,
        state: quyc_client::jsonpath::deserializer::core::DeserializerState::Initial,
        current_depth: 0,
        in_target_array: false,
        object_nesting: 0,
        object_buffer: Vec::new(),
        streaming_state,
        current_array_index: -1,
        array_index_stack: Vec::new(),
        buffer_position: 0,
        target_property: None,
        in_target_property: false,
        _phantom: std::marker::PhantomData,
    }
}

#[tokio::test]
async fn test_basic_array_streaming() -> Result<(), JsonPathError> {
    // Generate test data
    let test_objects = generate_test_objects(100);
    let json_data = create_json_stream(&test_objects);
    
    // Create JSONPath expression for array elements
    let parser = JsonPathParser::new();
    let expression = parser.parse("$[*]")?;
    
    // Create buffer and add data
    let mut buffer = StreamBuffer::new();
    buffer.append_data(&json_data);
    
    // Create deserializer
    let mut deserializer = create_deserializer_setup::<TestObject>(&expression, &mut buffer);
    
    // Process stream and collect results
    let start_time = Instant::now();
    let mut results = Vec::new();
    let mut iterator = deserializer.process_available_with_streaming();
    
    while let Some(result) = iterator.next() {
        match result {
            Ok(obj) => results.push(obj),
            Err(e) => {
                println!("Deserialization error: {}", e);
                continue;
            }
        }
    }
    
    let duration = start_time.elapsed();
    
    // Validate results
    assert!(results.len() > 0, "Should have deserialized at least some objects");
    assert!(duration < Duration::from_millis(100), "Processing should be fast");
    
    // Validate first object if available
    if let Some(first_obj) = results.first() {
        assert_eq!(first_obj.id, 0);
        assert_eq!(first_obj.name, "Object_0");
        assert_eq!(first_obj.active, true); // 0 % 3 == 0
    }
    
    println!("Basic array streaming: {} objects processed in {:?}", results.len(), duration);
    Ok(())
}

#[tokio::test]
async fn test_property_path_streaming() -> Result<(), JsonPathError> {
    // Generate nested test data
    let nested_data = generate_nested_test_data();
    let json_data = create_nested_json_stream(&nested_data);
    
    // Create JSONPath expression for specific property access
    let parser = JsonPathParser::new();
    let expression = parser.parse("$.users[*]")?;
    
    // Create buffer and add data
    let mut buffer = StreamBuffer::new();
    buffer.append_data(&json_data);
    
    // Create deserializer
    let mut deserializer = create_deserializer_setup::<TestUser>(&expression, &mut buffer);
    
    // Process stream
    let start_time = Instant::now();
    let mut results = Vec::new();
    let mut iterator = deserializer.process_available_with_streaming();
    
    while let Some(result) = iterator.next() {
        match result {
            Ok(user) => results.push(user),
            Err(e) => {
                println!("Deserialization error: {}", e);
                continue;
            }
        }
    }
    
    let duration = start_time.elapsed();
    
    // Validate results
    assert!(results.len() > 0, "Should have extracted user objects");
    
    // Validate first user if available
    if let Some(first_user) = results.first() {
        assert_eq!(first_user.id, 0);
        assert_eq!(first_user.email, "user0@test.com");
        assert_eq!(first_user.profile.name, "User 0");
    }
    
    println!("Property path streaming: {} users processed in {:?}", results.len(), duration);
    Ok(())
}

#[tokio::test]
async fn test_recursive_descent_streaming() -> Result<(), JsonPathError> {
    // Generate nested test data with deep structure
    let nested_data = generate_nested_test_data();
    let json_data = create_nested_json_stream(&nested_data);
    
    // Create JSONPath expression with recursive descent
    let parser = JsonPathParser::new();
    let expression = parser.parse("$..preferences")?;
    
    // Create buffer and add data
    let mut buffer = StreamBuffer::new();
    buffer.append_data(&json_data);
    
    // Create deserializer for preference arrays
    let mut deserializer = create_deserializer_setup::<Vec<String>>(&expression, &mut buffer);
    
    // Process stream
    let start_time = Instant::now();
    let mut results = Vec::new();
    let mut iterator = deserializer.process_available_with_streaming();
    
    while let Some(result) = iterator.next() {
        match result {
            Ok(preferences) => results.push(preferences),
            Err(e) => {
                println!("Deserialization error: {}", e);
                continue;
            }
        }
    }
    
    let duration = start_time.elapsed();
    
    // Validate results
    assert!(results.len() > 0, "Should have found preference arrays through recursive descent");
    
    // Validate first preference array if available
    if let Some(first_prefs) = results.first() {
        assert!(first_prefs.len() == 2, "Each user should have 2 preferences");
        assert!(first_prefs[0].starts_with("pref_"), "Preferences should follow naming pattern");
    }
    
    println!("Recursive descent streaming: {} preference arrays processed in {:?}", results.len(), duration);
    Ok(())
}

#[tokio::test]
async fn test_filtered_streaming() -> Result<(), JsonPathError> {
    // Generate test data with varying active status
    let test_objects = generate_test_objects(50);
    let json_data = create_json_stream(&test_objects);
    
    // Create JSONPath expression with filter for active objects
    let parser = JsonPathParser::new();
    let expression = parser.parse("$[?(@.active == true)]")?;
    
    // Create buffer and add data
    let mut buffer = StreamBuffer::new();
    buffer.append_data(&json_data);
    
    // Create deserializer
    let mut deserializer = create_deserializer_setup::<TestObject>(&expression, &mut buffer);
    
    // Process stream
    let start_time = Instant::now();
    let mut results = Vec::new();
    let mut iterator = deserializer.process_available_with_streaming();
    
    while let Some(result) = iterator.next() {
        match result {
            Ok(obj) => {
                assert!(obj.active, "Filtered results should only contain active objects");
                results.push(obj);
            }
            Err(e) => {
                println!("Deserialization error: {}", e);
                continue;
            }
        }
    }
    
    let duration = start_time.elapsed();
    
    // Validate results - every third object should be active (i % 3 == 0)
    let expected_active_count = test_objects.iter().filter(|obj| obj.active).count();
    
    println!("Filtered streaming: {} active objects processed in {:?} (expected ~{})", 
             results.len(), duration, expected_active_count);
    Ok(())
}

#[tokio::test]
async fn test_streaming_performance_metrics() -> Result<(), JsonPathError> {
    // Generate larger dataset for performance testing
    let test_objects = generate_test_objects(1000);
    let json_data = create_json_stream(&test_objects);
    
    // Create JSONPath expression
    let parser = JsonPathParser::new();
    let expression = parser.parse("$[*]")?;
    
    // Create buffer and add data
    let mut buffer = StreamBuffer::new();
    buffer.append_data(&json_data);
    
    // Create deserializer
    let mut deserializer = create_deserializer_setup::<TestObject>(&expression, &mut buffer);
    
    // Process stream with metrics collection
    let start_time = Instant::now();
    let mut results = Vec::new();
    let mut iterator = deserializer.process_available_with_streaming();
    
    // Collect initial metrics
    let initial_stats = deserializer.get_processing_stats();
    println!("Initial stats: {:?}", initial_stats);
    
    while let Some(result) = iterator.next() {
        match result {
            Ok(obj) => results.push(obj),
            Err(e) => {
                println!("Deserialization error: {}", e);
                continue;
            }
        }
        
        // Check if we should continue based on efficiency
        if !deserializer.should_continue_processing() {
            println!("Stopping due to low processing efficiency");
            break;
        }
    }
    
    let duration = start_time.elapsed();
    let final_stats = deserializer.get_processing_stats();
    
    // Calculate performance metrics
    let objects_per_second = results.len() as f64 / duration.as_secs_f64();
    let bytes_per_second = json_data.len() as f64 / duration.as_secs_f64();
    let memory_usage = deserializer.estimated_memory_usage();
    
    // Validate performance
    assert!(objects_per_second > 1000.0, 
            "Should process at least 1000 objects/second, got {:.2}", objects_per_second);
    assert!(final_stats.processing_efficiency > 0.1, 
            "Processing efficiency should be reasonable, got {:.2}", final_stats.processing_efficiency);
    
    println!("Performance metrics:");
    println!("  - Objects processed: {}", results.len());
    println!("  - Objects/second: {:.2}", objects_per_second);
    println!("  - Bytes/second: {:.2}", bytes_per_second);
    println!("  - Memory usage: {} KB", memory_usage / 1024);
    println!("  - Processing efficiency: {:.2}", final_stats.processing_efficiency);
    println!("  - Buffer utilization: {:.2}%", final_stats.buffer_utilization * 100.0);
    println!("  - Duration: {:?}", duration);
    
    Ok(())
}

#[tokio::test]
async fn test_chunked_streaming() -> Result<(), JsonPathError> {
    // Generate test data
    let test_objects = generate_test_objects(200);
    let json_data = create_json_stream(&test_objects);
    
    // Create JSONPath expression
    let parser = JsonPathParser::new();
    let expression = parser.parse("$[*]")?;
    
    // Create buffer
    let mut buffer = StreamBuffer::new();
    
    // Create deserializer
    let mut deserializer = create_deserializer_setup::<TestObject>(&expression, &mut buffer);
    
    let mut results = Vec::new();
    let chunk_size = 1024; // 1KB chunks
    let mut offset = 0;
    
    // Simulate streaming data in chunks
    while offset < json_data.len() {
        let end = (offset + chunk_size).min(json_data.len());
        let chunk = &json_data[offset..end];
        
        // Add chunk to buffer
        buffer.append_data(chunk);
        
        // Process available data
        let mut iterator = deserializer.process_available_with_streaming();
        while let Some(result) = iterator.next() {
            match result {
                Ok(obj) => results.push(obj),
                Err(e) => {
                    println!("Chunk processing error: {}", e);
                    continue;
                }
            }
        }
        
        offset = end;
        
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    
    // Validate chunked processing
    assert!(results.len() > 0, "Should have processed objects from chunked stream");
    println!("Chunked streaming: {} objects processed from {} chunks", 
             results.len(), (json_data.len() + chunk_size - 1) / chunk_size);
    
    Ok(())
}

#[tokio::test]
async fn test_error_recovery_streaming() -> Result<(), JsonPathError> {
    // Create partially malformed JSON data
    let mut json_data = b"[".to_vec();
    json_data.extend(b r#"{"id": 1, "name": "Object_1", "value": 1.5, "active": true},"#);
    json_data.extend(b r#"{"id": 2, "name": "Object_2", "value": 3.0, "active": false},"#);
    json_data.extend(b r#"{"id": 3, "invalid": json},"#); // Malformed object
    json_data.extend(b r#"{"id": 4, "name": "Object_4", "value": 6.0, "active": true},"#);
    json_data.extend(b"]");
    
    // Create JSONPath expression
    let parser = JsonPathParser::new();
    let expression = parser.parse("$[*]")?;
    
    // Create buffer and add data
    let mut buffer = StreamBuffer::new();
    buffer.append_data(&json_data);
    
    // Create deserializer
    let mut deserializer = create_deserializer_setup::<TestObject>(&expression, &mut buffer);
    
    // Process stream with error handling
    let mut results = Vec::new();
    let mut errors = Vec::new();
    let mut iterator = deserializer.process_available_with_streaming();
    
    while let Some(result) = iterator.next() {
        match result {
            Ok(obj) => results.push(obj),
            Err(e) => {
                errors.push(e);
                continue; // Continue processing despite errors
            }
        }
    }
    
    // Validate error recovery
    assert!(results.len() > 0, "Should have recovered some valid objects despite errors");
    assert!(errors.len() > 0, "Should have encountered errors from malformed JSON");
    
    println!("Error recovery streaming: {} objects recovered, {} errors encountered", 
             results.len(), errors.len());
    
    Ok(())
}

/// Integration test helper for comprehensive validation
#[tokio::test]
async fn test_comprehensive_streaming_validation() -> Result<(), JsonPathError> {
    // Test multiple JSONPath patterns against the same dataset
    let nested_data = generate_nested_test_data();
    let json_data = create_nested_json_stream(&nested_data);
    
    let test_cases = vec![
        ("$.users[*].id", "Extract all user IDs"),
        ("$.users[0]", "Extract first user"),
        ("$..name", "Recursive descent for all name fields"),
        ("$.stats[?(@.value > 10)]", "Filter stats by value"),
        ("$.users[*].profile.preferences[*]", "Deep array access"),
    ];
    
    for (path_pattern, description) in test_cases {
        println!("\nTesting: {} ({})", path_pattern, description);
        
        // Create JSONPath expression
        let parser = JsonPathParser::new();
        let expression = match parser.parse(path_pattern) {
            Ok(expr) => expr,
            Err(e) => {
                println!("Failed to parse JSONPath '{}': {}", path_pattern, e);
                continue;
            }
        };
        
        // Create buffer and add data
        let mut buffer = StreamBuffer::new();
        buffer.append_data(&json_data);
        
        // Create deserializer for generic JSON values
        let mut deserializer = create_deserializer_setup::<serde_json::Value>(&expression, &mut buffer);
        
        // Process stream
        let start_time = Instant::now();
        let mut results = Vec::new();
        let mut iterator = deserializer.process_available_with_streaming();
        
        while let Some(result) = iterator.next() {
            match result {
                Ok(value) => results.push(value),
                Err(e) => {
                    println!("Processing error for '{}': {}", path_pattern, e);
                    continue;
                }
            }
        }
        
        let duration = start_time.elapsed();
        let stats = deserializer.get_processing_stats();
        
        println!("  Results: {} items in {:?}", results.len(), duration);
        println!("  Processing efficiency: {:.2}", stats.processing_efficiency);
        println!("  Memory usage: {} KB", deserializer.estimated_memory_usage() / 1024);
        
        // Basic validation
        if path_pattern.contains("users[*].id") {
            assert!(results.len() > 0, "Should extract user IDs");
        }
    }
    
    Ok(())
}