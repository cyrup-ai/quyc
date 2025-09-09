use quyc_client::proxy::builder::{Proxy, ProxyIntercept};
use http::{HeaderMap, header::HeaderValue};

#[test]
fn test_module_integration() {
    // Test that all modules are properly integrated
    let proxy = Proxy::http("http://proxy.example.com:8080")
        .expect("Failed to create HTTP proxy for integration test");
    
    // Test type access
    assert!(matches!(proxy.intercept(), ProxyIntercept::Http(_)));
    
    // Test configuration chaining
    let configured_proxy = proxy
        .basic_auth("user", "pass")
        .no_proxy("localhost");
    
    assert!(configured_proxy.extra().auth().is_some());
    assert!(configured_proxy.no_proxy().is_some());
}

#[test]
fn test_all_constructor_types() {
    // Test all constructor methods work
    let http_proxy = Proxy::http("http://proxy.example.com")
        .expect("Failed to create HTTP proxy");
    let https_proxy = Proxy::https("https://proxy.example.com")
        .expect("Failed to create HTTPS proxy");
    let all_proxy = Proxy::all("http://proxy.example.com")
        .expect("Failed to create All proxy");
    let custom_proxy = Proxy::custom(|_| Some("http://custom.proxy"));

    assert!(matches!(http_proxy.intercept(), ProxyIntercept::Http(_)));
    assert!(matches!(https_proxy.intercept(), ProxyIntercept::Https(_)));
    assert!(matches!(all_proxy.intercept(), ProxyIntercept::All(_)));
    assert!(matches!(custom_proxy.intercept(), ProxyIntercept::Custom(_)));
}

#[test]
fn test_configuration_chaining() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Test", HeaderValue::from_static("value"));

    let proxy = Proxy::http("http://proxy.example.com")
        .expect("Failed to create HTTP proxy for configuration test")
        .basic_auth("user", "pass")
        .custom_headers(headers)
        .no_proxy("localhost,*.internal");

    assert!(proxy.extra().auth().is_some());
    assert!(proxy.extra().headers().is_some());
    assert!(proxy.no_proxy().is_some());
}