//! Tests for proxy internal functionality
//!
//! This module contains comprehensive tests for the proxy matcher,
//! intercepted connections, and proxy scheme functionality.

use quyc_client::proxy::internal::proxy_scheme::ProxyScheme;
use quyc_client::proxy::internal::intercepted::Intercepted;
use quyc_client::proxy::types::Extra;

#[test]
fn test_proxy_scheme_uri() {
    let scheme = ProxyScheme::Http {
        auth: None,
        host: "proxy.example.com".to_string(),
        port: 8080,
    };
    
    let uri = scheme.uri();
    assert_eq!(uri.scheme(), "http");
    assert_eq!(uri.host_str(), Some("proxy.example.com"));
    assert_eq!(uri.port(), Some(8080));
}

#[test]
fn test_intercepted_creation() {
    let scheme = ProxyScheme::Https {
        auth: None,
        host: "secure.proxy.com".to_string(),
        port: 3128,
    };
    
    let intercepted = Intercepted::new(scheme, Extra::default());
    assert_eq!(intercepted.scheme().host(), "secure.proxy.com");
    assert_eq!(intercepted.scheme().port(), 3128);
}

#[test]
fn test_socks5_scheme() {
    let scheme = ProxyScheme::Socks5 {
        auth: Some(("user".to_string(), "pass".to_string())),
        host: "socks.proxy.com".to_string(),
        port: 1080,
    };
    
    let uri = scheme.uri();
    assert_eq!(uri.scheme(), "socks5");
    assert_eq!(scheme.basic_auth(), Some(("user", "pass")));
}