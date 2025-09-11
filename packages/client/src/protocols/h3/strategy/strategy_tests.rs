//! H3Strategy Integration Tests

#[cfg(test)]
mod tests {
    use http::{Method, HeaderMap};
    use crate::http::request::{RequestBody, MultipartField, MultipartValue};
    use crate::protocols::h3::strategy::core::{
        serialize_multipart_form_data_public, 
        serialize_http_request_for_h3_public
    };

    /// Test multipart form data serialization
    #[test]
    fn test_multipart_serialization() {
        let fields = vec![
            MultipartField {
                name: "username".to_string(),
                filename: None,
                content_type: None,
                value: MultipartValue::Text("test_user".to_string()),
            },
            MultipartField {
                name: "file".to_string(),
                filename: Some("test.txt".to_string()),
                content_type: Some("text/plain".to_string()),
                value: MultipartValue::Bytes(bytes::Bytes::from(b"file content".to_vec())),
            },
        ];
        
        let mut headers = HeaderMap::new();
        let result = serialize_multipart_form_data_public(&fields, &mut headers);
        
        assert!(result.is_ok());
        let body_data = result.expect("Multipart serialization should succeed");
        
        // Verify body is not empty
        assert!(!body_data.is_empty());
        
        // Verify Content-Type header was set
        assert!(headers.contains_key("content-type"));
        let content_type = headers["content-type"].to_str()
            .expect("Content-Type header should contain valid UTF-8");
        assert!(content_type.starts_with("multipart/form-data; boundary="));
        
        // Verify body contains expected field data
        let body_str = String::from_utf8_lossy(&body_data);
        assert!(body_str.contains("username"));
        assert!(body_str.contains("test_user"));
        assert!(body_str.contains("file"));
        assert!(body_str.contains("test.txt"));
        assert!(body_str.contains("text/plain"));
    }

    /// Test HTTP/3 request serialization
    #[test]
    fn test_h3_request_serialization() {
        let method = Method::POST;
        let uri = "https://api.example.com/test";
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/json".parse()
            .expect("Static content-type should parse successfully"));
        
        let json_body = br#"{"test": "data"}"#;
        let body = RequestBody::Bytes(json_body.to_vec());
        
        let result = serialize_http_request_for_h3_public(&method, uri, &mut headers, Some(&body));
        
        assert!(result.is_ok());
        let serialized = result.expect("HTTP/3 request serialization should succeed");
        assert!(!serialized.is_empty());
        
        // Body should contain JSON data
        assert!(serialized.len() > 50);
    }

    /// Test various HTTP methods
    #[test]
    fn test_http_methods() {
        let test_methods = vec![
            Method::GET,
            Method::POST, 
            Method::PUT,
            Method::DELETE,
        ];
        
        for method in test_methods {
            let uri = "https://example.com/resource";
            let mut headers = HeaderMap::new();
            headers.insert("host", "example.com".parse()
                .expect("Static host header should parse successfully"));
            
            let result = serialize_http_request_for_h3_public(&method, uri, &mut headers, None);
            assert!(result.is_ok(), "Failed to serialize {} request", method);
            
            let serialized = result.expect("HTTP method serialization should succeed");
            assert!(!serialized.is_empty(), "Empty serialization for {} request", method);
        }
    }

    /// Test boundary generation uniqueness
    #[test]
    fn test_boundary_uniqueness() {
        let fields = vec![
            MultipartField {
                name: "test".to_string(),
                filename: None,
                content_type: None,
                value: MultipartValue::Text("data".to_string()),
            },
        ];
        
        let mut headers1 = HeaderMap::new();
        let mut headers2 = HeaderMap::new();
        
        let _result1 = serialize_multipart_form_data_public(&fields, &mut headers1);
        let _result2 = serialize_multipart_form_data_public(&fields, &mut headers2);
        
        let boundary1 = headers1["content-type"].to_str()
            .expect("First content-type header should contain valid UTF-8");
        let boundary2 = headers2["content-type"].to_str()
            .expect("Second content-type header should contain valid UTF-8");
        
        // Boundaries should be different for each call
        assert_ne!(boundary1, boundary2);
    }
}