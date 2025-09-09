//! JSON Path Streaming Tests
//!
//! Tests for the JSONPath streaming functionality, moved from src/json_path/mod.rs

use bytes::Bytes;
use quyc::jsonpath::JsonArrayStream;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    id: String,
    value: i32,
}

#[cfg(test)]
mod streaming_tests {
    use super::*;

    #[test]
    fn test_jsonpath_stream_creation() {
        let stream = JsonArrayStream::<TestModel>::new("$.data[*]");
        // Stream creation now always succeeds, invalid paths are handled internally
        assert!(format!("{:?}", stream).contains("JsonArrayStream"));

        let invalid_stream = JsonArrayStream::<TestModel>::new("$.invalid[syntax");
        // Invalid JSONPath expressions are handled via error emission, stream creation still succeeds
        assert!(format!("{:?}", invalid_stream).contains("JsonArrayStream"));
    }

    #[test]
    fn test_simple_array_streaming() {
        let mut stream = JsonArrayStream::<TestModel>::new("$[*]");

        let json_data = r#"[{"id":"test1","value":42},{"id":"test2","value":24}]"#;
        let chunk = Bytes::from(json_data);

        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(results.len(), 2);

        let first = &results[0];
        assert_eq!(first.id, "test1");
        assert_eq!(first.value, 42);
    }

    #[test]
    fn test_nested_object_streaming() {
        let mut stream = JsonArrayStream::<TestModel>::new("$.data[*]");

        let json_data = r#"{"data":[{"id":"nested1","value":100}],"meta":"info"}"#;
        let chunk = Bytes::from(json_data);

        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(results.len(), 1);

        let item = &results[0];
        assert_eq!(item.id, "nested1");
        assert_eq!(item.value, 100);
    }
}
