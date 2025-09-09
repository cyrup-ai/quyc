//! JSON Path Buffer Tests
//!
//! Tests for the streaming buffer functionality, moved from src/json_path/buffer.rs

use std::io::Read;

use bytes::Bytes;
use quyc::jsonpath::buffer::StreamBuffer;

#[cfg(test)]
mod buffer_tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = StreamBuffer::with_capacity(1024);
        assert_eq!(buffer.current_size(), 0);
        assert!(buffer.capacity() >= 1024);
    }

    #[test]
    fn test_chunk_appending() {
        let mut buffer = StreamBuffer::new();
        let chunk1 = Bytes::from("hello");
        let chunk2 = Bytes::from(" world");

        buffer.append_chunk(chunk1);
        buffer.append_chunk(chunk2);

        assert_eq!(buffer.current_size(), 11);
        assert_eq!(buffer.total_bytes_processed(), 11);
    }

    #[test]
    fn test_buffer_consumption() {
        let mut buffer = StreamBuffer::new();
        buffer.append_chunk(Bytes::from("hello world"));

        buffer.consume(5);
        assert_eq!(buffer.current_size(), 6);

        // Verify remaining data
        let reader_data: Vec<u8> = {
            let mut reader = buffer.reader();
            let mut data = Vec::new();
            reader.read_to_end(&mut data).expect("Read should succeed");
            data
        };
        assert_eq!(
            String::from_utf8(reader_data).expect("Valid UTF-8"),
            " world"
        );
    }

    #[test]
    fn test_json_boundary_detection() {
        let mut buffer = StreamBuffer::new();
        buffer.append_chunk(Bytes::from(r#"{"a":1}{"b":2}{"c":3}"#));

        let boundaries = buffer.find_object_boundaries();
        assert_eq!(boundaries, vec![7, 14, 21]);
    }

    #[test]
    fn test_json_boundary_with_strings() {
        let mut buffer = StreamBuffer::new();
        buffer.append_chunk(Bytes::from(r#"{"str":"with }braces"}{"next":true}"#));

        let boundaries = buffer.find_object_boundaries();
        assert_eq!(boundaries.len(), 2);
        assert!(boundaries[0] > 18); // After first complete object
    }

    #[test]
    fn test_buffer_reader() {
        let mut buffer = StreamBuffer::new();
        buffer.append_chunk(Bytes::from("test data"));

        let mut reader = buffer.reader();
        let mut read_buffer = [0u8; 4];

        let bytes_read = reader.read(&mut read_buffer).expect("Read should succeed");
        assert_eq!(bytes_read, 4);
        assert_eq!(&read_buffer, b"test");

        let bytes_read = reader.read(&mut read_buffer).expect("Read should succeed");
        assert_eq!(bytes_read, 4);
        assert_eq!(&read_buffer[..bytes_read], b" dat");
    }

    #[test]
    fn test_buffer_stats() {
        let mut buffer = StreamBuffer::with_capacity(1024);
        buffer.append_chunk(Bytes::from("test"));

        let stats = buffer.stats();
        assert_eq!(stats.current_size, 4);
        assert_eq!(stats.total_processed, 4);
        assert!(stats.capacity >= 1024);
        assert!(stats.utilization_ratio < 0.1); // Very low utilization
    }
}
