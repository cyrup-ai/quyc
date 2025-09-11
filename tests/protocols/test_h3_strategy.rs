//! H3Strategy Tests
//!
//! Tests for HTTP/3 strategy implementations including multipart form data
//! and streaming request body serialization.

use std::collections::HashMap;
use http::{Method, HeaderMap};
use quyc_client::http::request::{RequestBody, MultipartField};
use quyc_client::protocols::h3::strategy::core::{serialize_http_request_for_h3, serialize_multipart_form_data};

/// Test multipart form data serialization
#[test]
fn test_multipart_form_data_serialization() {
    let fields = vec![
        MultipartField {
            name: "username".to_string(),
            filename: None,
            content_type: None,
            data: b"john_doe".to_vec(),
        },
        MultipartField {
            name: "email".to_string(), 
            filename: None,
            content_type: None,
            data: b"john@example.com".to_vec(),
        },
        MultipartField {
            name: "avatar".to_string(),
            filename: Some("profile.jpg".to_string()),
            content_type: Some("image/jpeg".to_string()),
            data: b"fake_jpeg_data".to_vec(),
        },
    ];
    
    let mut headers = HeaderMap::new();
    let result = serialize_multipart_form_data(&fields, &mut headers);
    
    assert!(result.is_ok());
    let body_data = result.unwrap();
    
    // Verify body is not empty
    assert!(!body_data.is_empty());
    
    // Verify Content-Type header was set
    assert!(headers.contains_key("content-type"));
    let content_type = headers["content-type"].to_str().unwrap();
    assert!(content_type.starts_with("multipart/form-data; boundary="));
    
    // Verify body contains expected field data
    let body_str = String::from_utf8_lossy(&body_data);
    assert!(body_str.contains("username"));
    assert!(body_str.contains("john_doe"));
    assert!(body_str.contains("email"));
    assert!(body_str.contains("john@example.com"));
    assert!(body_str.contains("avatar"));
    assert!(body_str.contains("profile.jpg"));
    assert!(body_str.contains("image/jpeg"));
}

/// Test multipart boundary generation uniqueness  
#[test]
fn test_multipart_boundary_uniqueness() {
    let fields = vec![
        MultipartField {
            name: "test".to_string(),
            filename: None,
            content_type: None,
            data: b"test_data".to_vec(),
        },
    ];
    
    let mut headers1 = HeaderMap::new();
    let mut headers2 = HeaderMap::new();
    
    let _result1 = serialize_multipart_form_data(&fields, &mut headers1);
    let _result2 = serialize_multipart_form_data(&fields, &mut headers2);
    
    let boundary1 = headers1["content-type"].to_str().unwrap();
    let boundary2 = headers2["content-type"].to_str().unwrap();
    
    // Boundaries should be different for each call
    assert_ne!(boundary1, boundary2);
}

/// Test HTTP/3 request serialization with various request types
#[test] 
fn test_h3_request_serialization_get() {
    let method = Method::GET;
    let uri = "https://api.example.com/users";
    let mut headers = HeaderMap::new();
    headers.insert("accept", "application/json".parse().unwrap());
    headers.insert("user-agent", "quyc-client/1.0".parse().unwrap());
    
    let result = serialize_http_request_for_h3(&method, uri, &mut headers, None);
    
    assert!(result.is_ok());
    let serialized = result.unwrap();
    assert!(!serialized.is_empty());
    
    // Verify it contains HTTP/3 components
    let serialized_str = String::from_utf8_lossy(&serialized);
    // Basic validation that it looks like HTTP request data
    assert!(serialized_str.len() > 10);
}

#[test]
fn test_h3_request_serialization_post_json() {
    let method = Method::POST;
    let uri = "https://api.example.com/users";
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());
    headers.insert("accept", "application/json".parse().unwrap());
    
    let json_body = br#"{"name": "John Doe", "email": "john@example.com"}"#;
    let body = RequestBody::Bytes(json_body.to_vec());
    
    let result = serialize_http_request_for_h3(&method, uri, &mut headers, Some(&body));
    
    assert!(result.is_ok());
    let serialized = result.unwrap();
    assert!(!serialized.is_empty());
    
    // Body should be longer since it includes JSON data
    assert!(serialized.len() > 100);
}

#[test]
fn test_h3_request_serialization_multipart() {
    let method = Method::POST;
    let uri = "https://api.example.com/upload";
    let mut headers = HeaderMap::new();
    
    let fields = vec![
        MultipartField {
            name: "file".to_string(),
            filename: Some("document.pdf".to_string()),
            content_type: Some("application/pdf".to_string()),
            data: b"fake_pdf_content".to_vec(),
        },
        MultipartField {
            name: "description".to_string(),
            filename: None,
            content_type: None,
            data: b"Important document".to_vec(),
        },
    ];
    
    let body = RequestBody::Multipart(fields);
    
    let result = serialize_http_request_for_h3(&method, uri, &mut headers, Some(&body));
    
    assert!(result.is_ok());
    let serialized = result.unwrap();
    assert!(!serialized.is_empty());
    
    // Verify multipart content-type was set
    assert!(headers.contains_key("content-type"));
    let content_type = headers["content-type"].to_str().unwrap();
    assert!(content_type.starts_with("multipart/form-data"));
    
    // Body should contain multipart data
    let body_str = String::from_utf8_lossy(&serialized);
    assert!(body_str.contains("document.pdf"));
    assert!(body_str.contains("application/pdf"));
    assert!(body_str.contains("Important document"));
}

/// Test edge cases and error handling
#[test]
fn test_empty_multipart_fields() {
    let fields = vec![];
    let mut headers = HeaderMap::new();
    
    let result = serialize_multipart_form_data(&fields, &mut headers);
    assert!(result.is_ok());
    
    // Should still set content-type even for empty fields
    assert!(headers.contains_key("content-type"));
    
    let body = result.unwrap();
    // Should have minimal multipart structure
    assert!(!body.is_empty());
}

#[test]
fn test_special_characters_in_multipart() {
    let fields = vec![
        MultipartField {
            name: "field with spaces".to_string(),
            filename: Some("file-with-dashes_and_underscores.txt".to_string()),
            content_type: Some("text/plain; charset=utf-8".to_string()),
            data: "Content with\nnewlines and\ttabs".as_bytes().to_vec(),
        },
    ];
    
    let mut headers = HeaderMap::new();
    let result = serialize_multipart_form_data(&fields, &mut headers);
    
    assert!(result.is_ok());
    let body_data = result.unwrap();
    
    let body_str = String::from_utf8_lossy(&body_data);
    assert!(body_str.contains("field with spaces"));
    assert!(body_str.contains("file-with-dashes_and_underscores.txt"));
    assert!(body_str.contains("text/plain; charset=utf-8"));
    assert!(body_str.contains("Content with\nnewlines and\ttabs"));
}

/// Test request serialization with various HTTP methods
#[test]
fn test_h3_request_methods() {
    let test_cases = vec![
        Method::GET,
        Method::POST, 
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
        Method::HEAD,
        Method::OPTIONS,
    ];
    
    for method in test_cases {
        let uri = "https://api.example.com/resource";
        let mut headers = HeaderMap::new();
        headers.insert("host", "api.example.com".parse().unwrap());
        
        let result = serialize_http_request_for_h3(&method, uri, &mut headers, None);
        assert!(result.is_ok(), "Failed to serialize {} request", method);
        
        let serialized = result.unwrap();
        assert!(!serialized.is_empty(), "Empty serialization for {} request", method);
    }
}

/// Test header handling in request serialization  
#[test]
fn test_h3_request_header_handling() {
    let method = Method::GET;
    let uri = "https://api.example.com/data";
    let mut headers = HeaderMap::new();
    
    // Add various types of headers
    headers.insert("accept", "application/json, text/plain".parse().unwrap());
    headers.insert("accept-encoding", "gzip, deflate, br".parse().unwrap());
    headers.insert("accept-language", "en-US,en;q=0.9".parse().unwrap());
    headers.insert("cache-control", "no-cache".parse().unwrap());
    headers.insert("connection", "keep-alive".parse().unwrap());
    headers.insert("user-agent", "quyc-client/1.0 (HTTP/3)".parse().unwrap());
    headers.insert("x-custom-header", "custom-value-123".parse().unwrap());
    
    let result = serialize_http_request_for_h3(&method, uri, &mut headers, None);
    
    assert!(result.is_ok());
    let serialized = result.unwrap();
    assert!(!serialized.is_empty());
    
    // Verify headers are preserved (implementation-dependent how they're encoded)
    assert!(serialized.len() > 200); // Should be substantial with all these headers
}

/// Test URL encoding and special characters
#[test]
fn test_h3_request_uri_handling() {
    let test_uris = vec![
        "https://example.com/simple",
        "https://example.com/path/with/slashes",
        "https://example.com/query?param1=value1&param2=value2",
        "https://example.com/encoded?query=hello%20world&special=%21%40%23",
        "https://subdomain.example.com:8080/port/test",
    ];
    
    for uri in test_uris {
        let method = Method::GET;
        let mut headers = HeaderMap::new();
        headers.insert("host", "example.com".parse().unwrap());
        
        let result = serialize_http_request_for_h3(&method, uri, &mut headers, None);
        assert!(result.is_ok(), "Failed to serialize request for URI: {}", uri);
        
        let serialized = result.unwrap();
        assert!(!serialized.is_empty(), "Empty serialization for URI: {}", uri);
    }
}