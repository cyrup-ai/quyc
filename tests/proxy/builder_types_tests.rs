use quyc_client::proxy::builder::types::*;
use quyc_client::Url;

#[test]
fn test_proxy_creation() {
    let proxy = Proxy::new(ProxyIntercept::Http(
        Url::parse("http://example.com").expect("Failed to parse proxy URL")
    ));
    
    assert!(matches!(proxy.intercept(), ProxyIntercept::Http(_)));
    assert!(proxy.no_proxy().is_none());
}

#[test]
fn test_proxy_debug() {
    let proxy = Proxy::new(ProxyIntercept::Http(
        Url::parse("http://example.com").expect("Failed to parse proxy URL for debug test")
    ));
    
    let debug_str = format!("{:?}", proxy);
    assert!(debug_str.contains("Proxy"));
}