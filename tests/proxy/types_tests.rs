use http::{HeaderMap, HeaderValue};
use quyc_client::proxy::types::{ProxyUrl, Intercept, Via, Extra};

#[test]
fn test_proxy_url_creation() {
    let url = quyc_client::Url::parse("http://proxy.example.com:8080")
        .expect("Test URL should parse correctly");
    let proxy_url = ProxyUrl::new(url.clone());
    assert_eq!(proxy_url.url, url);
    assert!(!proxy_url.is_error());
}

#[test]
fn test_proxy_url_bad_chunk() {
    let bad_proxy = ProxyUrl::bad_chunk("Test error".to_string());
    assert!(bad_proxy.is_error());
    assert_eq!(bad_proxy.error_message(), Some("Test error"));
}

#[test]
fn test_intercept_basic_auth() {
    let url = quyc_client::Url::parse("http://user:pass@proxy.example.com:8080")
        .expect("Test URL with auth should parse correctly");
    let intercept = Intercept::new(url, Via::Http);
    
    let auth = intercept.basic_auth();
    assert_eq!(auth, Some(("user", "pass")));
}

#[test]
fn test_extra_configuration() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Custom", HeaderValue::from_static("value"));
    
    let extra = Extra::new()
        .with_headers(headers);
        
    assert!(extra.headers().is_some());
    assert_eq!(extra.headers().expect("Headers should be present").len(), 1);
}