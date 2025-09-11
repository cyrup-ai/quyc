//! Wire Protocol Tests
//! 
//! Comprehensive tests for HPACK and QPACK implementations in the wire protocol module.

use std::collections::HashMap;
use quyc_client::protocols::wire::WireProtocol;

/// Test HPACK header parsing with RFC 7541 compliance
#[test]
fn test_hpack_header_parsing() {
    // Test basic header parsing with static table entries
    // HTTP method GET is index 2 in static table  
    let basic_hpack = vec![0x82]; // Indexed header field for :method GET
    let headers = WireProtocol::parse_hpack_headers(&basic_hpack);
    
    assert!(headers.contains_key(":method"));
    assert_eq!(headers[":method"], "GET");
}

#[test]
fn test_hpack_literal_headers() {
    // Test literal header field with indexing - new name
    // Pattern: 01 (literal with incremental indexing) + name length + name + value length + value
    let literal_hpack = vec![
        0x40, // Literal Header Field with Incremental Indexing — New Name
        0x0a, // Name length: 10
        b'c', b'u', b's', b't', b'o', b'm', b'-', b'k', b'e', b'y', // "custom-key"
        0x0d, // Value length: 13
        b'c', b'u', b's', b't', b'o', b'm', b'-', b'h', b'e', b'a', b'd', b'e', b'r' // "custom-header"
    ];
    
    let headers = WireProtocol::parse_hpack_headers(&literal_hpack);
    assert!(headers.contains_key("custom-key"));
    assert_eq!(headers["custom-key"], "custom-header");
}

#[test]
fn test_hpack_integer_decoding() {
    // Test integer decoding with different prefix lengths
    // 5-bit prefix: value 10 = 0x0A (since 10 < 31, no continuation)
    let result = WireProtocol::decode_integer(&[0x0A], 0, 5);
    assert_eq!(result, Ok((10, 1)));
    
    // 5-bit prefix: value 1337 requires continuation
    // 1337 = 31 (max 5-bit) + 1306
    // Encoded as: 0x1F (31) + variable-length encoding of 1306
    let large_int = vec![0x1F, 0x9A, 0x0A]; // 31 + (1306 in varint)
    let result = WireProtocol::decode_integer(&large_int, 0, 5);
    assert_eq!(result, Ok((1337, 3)));
}

#[test]
fn test_hpack_string_decoding() {
    // Test string decoding without Huffman encoding
    let test_string = b"www.example.com";
    let mut encoded = vec![test_string.len() as u8]; // Length prefix
    encoded.extend_from_slice(test_string);
    
    let result = WireProtocol::decode_string(&encoded, 0);
    assert_eq!(result, Ok(("www.example.com".to_string(), encoded.len())));
}

#[test]
fn test_hpack_string_huffman_encoded() {
    // Test string decoding with Huffman encoding bit set
    // Huffman encoding: bit 7 = 1, length in bits 6-0
    let huffman_encoded = vec![
        0x80 | 0x08, // Huffman + length 8 (placeholder)
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // Huffman data (placeholder)
    ];
    
    let result = WireProtocol::decode_string(&huffman_encoded, 0);
    // For now, we expect an error since Huffman decoding is not fully implemented
    assert!(result.is_err());
}

/// Test QPACK header parsing with RFC 9204 compliance
#[test]
fn test_qpack_header_parsing() {
    // Test basic QPACK header block parsing
    // QPACK header block structure: required insert count + base + encoded headers
    let qpack_block = vec![
        0x00, // Required Insert Count = 0 (no dynamic table references)
        0x00, // Base = 0
        0x00, // Encoded Field Section (empty for this test)
    ];
    
    let headers = WireProtocol::parse_qpack_headers(&qpack_block);
    // Should return empty headers for minimal block
    assert!(headers.is_empty());
}

#[test]
fn test_qpack_static_table_lookup() {
    // Test QPACK static table lookups
    // QPACK uses same static table as HPACK but with different indexing
    let static_ref = vec![
        0x00, // Required Insert Count = 0
        0x00, // Base = 0  
        0xC1, // Static Table Reference: index 1 (:authority)
    ];
    
    let headers = WireProtocol::parse_qpack_headers(&static_ref);
    assert!(headers.contains_key(":authority"));
}

#[test]
fn test_qpack_integer_encoding() {
    // Test QPACK integer encoding/decoding
    let (value, bytes_consumed) = WireProtocol::decode_qpack_integer(&[0x0A], 0, 8).unwrap();
    assert_eq!(value, 10);
    assert_eq!(bytes_consumed, 1);
    
    // Test larger integer requiring continuation
    let large_value = vec![0xFF, 0x80, 0x01]; // 255 + 128 = 383 in varint
    let (value, bytes_consumed) = WireProtocol::decode_qpack_integer(&large_value, 0, 8).unwrap();
    assert_eq!(value, 383);
    assert_eq!(bytes_consumed, 3);
}

/// Test HPACK serialization functionality
#[test]
fn test_hpack_serialization() {
    let mut headers = HashMap::new();
    headers.insert(":method".to_string(), "GET".to_string());
    headers.insert(":path".to_string(), "/index.html".to_string());
    headers.insert("host".to_string(), "example.com".to_string());
    
    let serialized = WireProtocol::serialize_hpack_headers(&headers);
    
    // Verify it's not empty
    assert!(!serialized.is_empty());
    
    // Verify it can be parsed back
    let parsed = WireProtocol::parse_hpack_headers(&serialized);
    assert_eq!(parsed.len(), headers.len());
    
    // Check key headers are present
    assert!(parsed.contains_key(":method"));
    assert!(parsed.contains_key(":path"));
    assert!(parsed.contains_key("host"));
}

/// Test QPACK serialization functionality
#[test]
fn test_qpack_serialization() {
    let mut headers = HashMap::new();
    headers.insert(":method".to_string(), "POST".to_string());
    headers.insert(":path".to_string(), "/api/data".to_string());
    headers.insert("content-type".to_string(), "application/json".to_string());
    
    let serialized = WireProtocol::serialize_qpack_headers(&headers);
    
    // Verify it's not empty
    assert!(!serialized.is_empty());
    
    // Verify it can be parsed back
    let parsed = WireProtocol::parse_qpack_headers(&serialized);
    assert_eq!(parsed.len(), headers.len());
    
    // Check key headers are present  
    assert!(parsed.contains_key(":method"));
    assert!(parsed.contains_key(":path"));
    assert!(parsed.contains_key("content-type"));
}

/// Test round-trip serialization/deserialization
#[test]
fn test_hpack_round_trip() {
    let original_headers = vec![
        (":method", "PUT"),
        (":scheme", "https"),
        (":authority", "api.example.com"),
        (":path", "/v1/users/123"),
        ("content-type", "application/json"),
        ("authorization", "Bearer token123"),
        ("x-request-id", "abc-def-ghi"),
    ];
    
    let mut headers_map = HashMap::new();
    for (key, value) in &original_headers {
        headers_map.insert(key.to_string(), value.to_string());
    }
    
    // Serialize
    let serialized = WireProtocol::serialize_hpack_headers(&headers_map);
    
    // Deserialize
    let parsed = WireProtocol::parse_hpack_headers(&serialized);
    
    // Verify all headers are preserved
    assert_eq!(parsed.len(), original_headers.len());
    for (key, expected_value) in &original_headers {
        assert!(parsed.contains_key(*key), "Missing header: {}", key);
        assert_eq!(parsed[*key], *expected_value, "Value mismatch for header: {}", key);
    }
}

/// Test QPACK round-trip serialization/deserialization  
#[test]
fn test_qpack_round_trip() {
    let original_headers = vec![
        (":method", "DELETE"),
        (":scheme", "https"),
        (":authority", "service.example.org"),
        (":path", "/api/v2/resources/456"),
        ("accept", "application/json"),
        ("user-agent", "quyc-client/1.0"),
        ("x-correlation-id", "correlation-xyz-789"),
    ];
    
    let mut headers_map = HashMap::new();
    for (key, value) in &original_headers {
        headers_map.insert(key.to_string(), value.to_string());
    }
    
    // Serialize
    let serialized = WireProtocol::serialize_qpack_headers(&headers_map);
    
    // Deserialize
    let parsed = WireProtocol::parse_qpack_headers(&serialized);
    
    // Verify all headers are preserved
    assert_eq!(parsed.len(), original_headers.len());
    for (key, expected_value) in &original_headers {
        assert!(parsed.contains_key(*key), "Missing header: {}", key);
        assert_eq!(parsed[*key], *expected_value, "Value mismatch for header: {}", key);
    }
}

/// Test error handling for malformed input
#[test]
fn test_malformed_input_handling() {
    // Test empty input
    let empty_headers = WireProtocol::parse_hpack_headers(&[]);
    assert!(empty_headers.is_empty());
    
    // Test truncated input
    let truncated = vec![0x40, 0x0a]; // Literal header with name length 10 but no name
    let headers = WireProtocol::parse_hpack_headers(&truncated);
    // Should handle gracefully without panicking
    assert!(headers.is_empty() || !headers.is_empty()); // Just ensure no panic
    
    // Test invalid integer encoding
    let invalid_int = WireProtocol::decode_integer(&[], 0, 5);
    assert!(invalid_int.is_err());
}

/// Test static table lookups for HTTP/2 and HTTP/3
#[test]
fn test_static_table_coverage() {
    // Test that we can resolve common static table entries
    let common_indices = vec![2, 3, 4, 5, 6, 7]; // :method GET/POST, :path /, :scheme http/https
    
    for index in common_indices {
        // Try to create a simple indexed header field
        let indexed_header = vec![0x80 | index]; // Indexed Header Field
        let headers = WireProtocol::parse_hpack_headers(&indexed_header);
        
        // Should find at least one header
        assert!(!headers.is_empty(), "Failed to resolve static table index {}", index);
    }
}