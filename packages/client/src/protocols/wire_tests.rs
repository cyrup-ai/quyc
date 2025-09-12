//! Wire Protocol Integration Tests
//! 
//! Tests for HPACK and QPACK implementations in the wire protocol module.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::protocols::wire::WireProtocol;

    /// Test basic HPACK functionality
    #[test]
    fn test_hpack_basic_functionality() {
        // Test static table lookup
        let static_indexed = vec![0x82]; // :method GET (index 2)
        let headers = WireProtocol::parse_hpack_headers(&static_indexed);
        
        assert!(headers.contains_key(":method"));
        assert_eq!(headers[":method"], "GET");
    }

    /// Test HPACK round-trip serialization
    #[test]
    fn test_hpack_round_trip() {
        let mut original_headers = HashMap::new();
        original_headers.insert(":method".to_string(), "POST".to_string());
        original_headers.insert(":path".to_string(), "/api/test".to_string());
        original_headers.insert("host".to_string(), "example.com".to_string());
        
        // Serialize
        let serialized = WireProtocol::serialize_hpack_headers(&original_headers);
        assert!(!serialized.is_empty());
        
        // Deserialize
        let parsed = WireProtocol::parse_hpack_headers(&serialized);
        
        // Verify key headers are preserved
        assert_eq!(parsed.len(), original_headers.len());
        for (key, expected_value) in &original_headers {
            assert!(parsed.contains_key(key), "Missing header: {key}");
            assert_eq!(parsed[key], *expected_value, "Value mismatch for header: {key}");
        }
    }

    /// Test QPACK basic functionality
    #[test]
    fn test_qpack_basic_functionality() {
        // Test empty QPACK block
        let empty_block = vec![0x00, 0x00]; // Required Insert Count = 0, Base = 0
        let headers = WireProtocol::parse_qpack_headers(&empty_block);
        assert!(headers.is_empty());
    }

    /// Test QPACK round-trip serialization
    #[test]
    fn test_qpack_round_trip() {
        let mut original_headers = HashMap::new();
        original_headers.insert(":method".to_string(), "PUT".to_string());
        original_headers.insert(":path".to_string(), "/api/v1/resource".to_string());
        original_headers.insert("content-type".to_string(), "application/json".to_string());
        
        // Serialize
        let serialized = WireProtocol::serialize_qpack_headers(&original_headers);
        assert!(!serialized.is_empty());
        
        // Deserialize
        let parsed = WireProtocol::parse_qpack_headers(&serialized);
        
        // Verify headers are preserved
        assert_eq!(parsed.len(), original_headers.len());
        for (key, expected_value) in &original_headers {
            assert!(parsed.contains_key(key), "Missing header: {key}");
            assert_eq!(parsed[key], *expected_value, "Value mismatch for header: {key}");
        }
    }

    /// Test integer encoding/decoding
    #[test]
    fn test_integer_encoding_decoding() {
        // Test small integer
        let result = WireProtocol::decode_integer(&[0x0A], 0, 5);
        assert_eq!(result, Ok((10, 1)));
        
        // Test QPACK integer
        let result = WireProtocol::decode_qpack_integer(&[0x15], 0, 8);
        assert_eq!(result, Ok((21, 1)));
    }

    /// Test string decoding
    #[test] 
    fn test_string_decoding() {
        // Test basic string without Huffman encoding
        let test_string = b"example.com";
        let mut encoded = vec![test_string.len() as u8];
        encoded.extend_from_slice(test_string);
        
        let result = WireProtocol::decode_string(&encoded, 0);
        assert_eq!(result, Ok(("example.com".to_string(), encoded.len())));
    }

    /// Test error handling for malformed input
    #[test]
    fn test_error_handling() {
        // Test empty input
        let empty_headers = WireProtocol::parse_hpack_headers(&[]);
        assert!(empty_headers.is_empty());
        
        // Test invalid integer decoding
        let invalid_int = WireProtocol::decode_integer(&[], 0, 5);
        assert!(invalid_int.is_err());
    }
}