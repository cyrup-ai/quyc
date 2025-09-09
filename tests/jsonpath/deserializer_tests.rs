//! JSON Path Deserializer Tests
//!
//! Tests for the JSONPath deserializer functionality, moved from src/json_path/deserializer_old.rs

use bytes::Bytes;
use quyc::jsonpath::{
    JsonPathParser, buffer::StreamBuffer, deserializer::JsonPathDeserializer,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    id: String,
    value: i32,
}

#[cfg(test)]
mod deserializer_tests {
    use super::*;

    #[test]
    fn test_simple_array_deserialization() {
        let json_data = r#"[{"id":"test1","value":42},{"id":"test2","value":24}]"#;
        let path_expr = JsonPathParser::compile("$[*]").expect("Valid JSONPath expression");
        let mut buffer = StreamBuffer::with_capacity(1024);

        buffer.append_chunk(Bytes::from(json_data));

        let mut deserializer = JsonPathDeserializer::<TestModel>::new(&path_expr, &mut buffer);
        let results: Vec<_> = deserializer.process_available().collect();

        // Debug output
        println!("Results found: {}", results.len());
        for (i, result) in results.iter().enumerate() {
            match result {
                Ok(obj) => println!("Result {}: id={}, value={}", i, obj.id, obj.value),
                Err(e) => println!("Result {}: Error={:?}", i, e),
            }
        }

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        let first = results[0].as_ref().unwrap();
        assert_eq!(first.id, "test1");
        assert_eq!(first.value, 42);
    }

    #[test]
    fn test_nested_object_deserialization() {
        let json_data = r#"{"data":[{"id":"nested1","value":100}],"meta":"info"}"#;
        let path_expr = JsonPathParser::compile("$.data[*]").expect("Valid JSONPath expression");
        let mut buffer = StreamBuffer::with_capacity(1024);

        buffer.append_chunk(Bytes::from(json_data));

        let mut deserializer = JsonPathDeserializer::<TestModel>::new(&path_expr, &mut buffer);
        let results: Vec<_> = deserializer.process_available().collect();

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());

        let item = results[0].as_ref().unwrap();
        assert_eq!(item.id, "nested1");
        assert_eq!(item.value, 100);
    }

    #[test]
    fn test_streaming_chunks() {
        let path_expr = JsonPathParser::compile("$.items[*]").expect("Valid JSONPath expression");
        let mut buffer = StreamBuffer::with_capacity(1024);

        // Add data in chunks to simulate streaming
        buffer.append_chunk(Bytes::from(r#"{"items":["#));
        buffer.append_chunk(Bytes::from(r#"{"id":"chunk1","value":1},"#));
        buffer.append_chunk(Bytes::from(r#"{"id":"chunk2","value":2}"#));
        buffer.append_chunk(Bytes::from(r#"]}"#));

        let mut deserializer = JsonPathDeserializer::<TestModel>::new(&path_expr, &mut buffer);
        let results: Vec<_> = deserializer.process_available().collect();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_malformed_json_error_handling() {
        let json_data = r#"{"data":[{"id":"test1","invalid":}]}"#; // Missing value
        let path_expr = JsonPathParser::compile("$.data[*]").expect("Valid JSONPath expression");
        let mut buffer = StreamBuffer::with_capacity(1024);

        buffer.append_chunk(Bytes::from(json_data));

        let mut deserializer = JsonPathDeserializer::<TestModel>::new(&path_expr, &mut buffer);
        let results: Vec<_> = deserializer.process_available().collect();

        assert!(!results.is_empty());
        assert!(results[0].is_err());
    }
}
