use quyc_client::proxy::builder::constructors::*;
use quyc_client::proxy::builder::types::{Proxy, ProxyIntercept};

#[test]
fn test_proxy_http_creation() {
    let proxy = Proxy::http("http://proxy.example.com:8080")
        .expect("Failed to create HTTP proxy");
    match proxy.intercept() {
        ProxyIntercept::Http(url) => {
            assert_eq!(url.as_str(), "http://proxy.example.com:8080/");
        }
        _ => panic!("Expected Http intercept"),
    }
}

#[test]
fn test_proxy_https_creation() {
    let proxy = Proxy::https("https://proxy.example.com:8080")
        .expect("Failed to create HTTPS proxy");
    match proxy.intercept() {
        ProxyIntercept::Https(url) => {
            assert_eq!(url.as_str(), "https://proxy.example.com:8080/");
        }
        _ => panic!("Expected Https intercept"),
    }
}

#[test]
fn test_proxy_all_creation() {
    let proxy = Proxy::all("http://proxy.example.com:8080")
        .expect("Failed to create All proxy");
    match proxy.intercept() {
        ProxyIntercept::All(url) => {
            assert_eq!(url.as_str(), "http://proxy.example.com:8080/");
        }
        _ => panic!("Expected All intercept"),
    }
}

#[test]
fn test_proxy_custom_creation() {
    let proxy = Proxy::custom(|_url| Some("http://custom.proxy"));
    match proxy.intercept() {
        ProxyIntercept::Custom(_) => {
            // Custom proxy created successfully
        }
        _ => panic!("Expected Custom intercept"),
    }
}