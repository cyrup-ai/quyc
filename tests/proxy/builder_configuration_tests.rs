use quyc_client::proxy::builder::configuration::*;
use quyc_client::proxy::builder::types::{Proxy, ProxyIntercept};
use quyc_client::Url;
use http::{HeaderMap, HeaderValue};

fn create_test_proxy() -> Proxy {
    Proxy::new(ProxyIntercept::Http(
        Url::parse("http://proxy.example.com:8080")
            .expect("Failed to parse test proxy URL")
    ))
}

#[test]
fn test_proxy_basic_auth() {
    let proxy = create_test_proxy().basic_auth("user", "pass");
    
    assert!(proxy.extra().auth().is_some());
    let auth_header = proxy.extra().auth().expect("Auth header should be present");
    assert!(auth_header.to_str().expect("Auth header should be valid UTF-8").starts_with("Basic "));
}

#[test]
fn test_proxy_custom_auth() {
    let auth_value = HeaderValue::from_static("Bearer token123");
    let proxy = create_test_proxy().custom_http_auth(auth_value.clone());
    
    assert!(proxy.extra().auth().is_some());
    assert_eq!(proxy.extra().auth().expect("Auth header should be present"), &auth_value);
}

#[test]
fn test_proxy_custom_headers() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Custom", HeaderValue::from_static("test"));
    
    let proxy = create_test_proxy().custom_headers(headers);
    
    assert!(proxy.extra().headers().is_some());
    assert_eq!(proxy.extra().headers().expect("Headers should be present").len(), 1);
}

#[test]
fn test_proxy_no_proxy() {
    let proxy = create_test_proxy().no_proxy("localhost,*.internal");
    
    assert!(proxy.no_proxy().is_some());
}

#[test]
fn test_encode_basic_auth() {
    let header = encode_basic_auth("user", "pass");
    let expected = "Basic dXNlcjpwYXNz"; // base64 of "user:pass"
    assert_eq!(header.to_str().expect("Header should be valid UTF-8"), expected);
}

#[test]
fn test_encode_basic_auth_invalid_chars() {
    // Test with characters that might cause issues
    let header = encode_basic_auth("user\n", "pass\r");
    assert!(header.to_str().expect("Header should be valid UTF-8").starts_with("Basic "));
}