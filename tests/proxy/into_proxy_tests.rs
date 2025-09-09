use quyc_client::proxy::into_proxy::{IntoProxy, validate_proxy_url, parse_proxy_url};

#[test]
fn test_str_into_proxy() {
    let proxy_url = "http://proxy.example.com:8080".into_proxy().unwrap();
    assert_eq!(proxy_url.scheme(), "http");
    assert_eq!(proxy_url.host_str(), Some("proxy.example.com"));
    assert_eq!(proxy_url.port(), Some(8080));
}

#[test]
fn test_string_into_proxy() {
    let proxy_str = "https://secure.proxy.com:3128".to_string();
    let proxy_url = proxy_str.into_proxy().unwrap();
    assert_eq!(proxy_url.scheme(), "https");
    assert_eq!(proxy_url.host_str(), Some("secure.proxy.com"));
    assert_eq!(proxy_url.port(), Some(3128));
}

#[test]
fn test_url_into_proxy() {
    let original_url = quyc_client::Url::parse("http://proxy.test").unwrap();
    let proxy_url = original_url.clone().into_proxy().unwrap();
    assert_eq!(proxy_url, original_url);
}

#[test]
fn test_validate_proxy_url_valid() {
    let url = quyc_client::Url::parse("http://proxy.example.com:8080").unwrap();
    assert!(validate_proxy_url(&url).is_ok());
}

#[test]
fn test_validate_proxy_url_invalid_scheme() {
    let url = quyc_client::Url::parse("ftp://proxy.example.com").unwrap();
    let result = validate_proxy_url(&url);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unsupported proxy scheme"));
}#[test]
fn test_validate_proxy_url_no_host() {
    let url = quyc_client::Url::parse("http:///path").unwrap();
    let result = validate_proxy_url(&url);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must have a host"));
}

#[test]
fn test_parse_proxy_url_valid() {
    let result = parse_proxy_url("http://proxy.example.com:8080");
    assert!(result.is_ok());
    let url = result.unwrap();
    assert_eq!(url.host_str(), Some("proxy.example.com"));
    assert_eq!(url.port(), Some(8080));
}

#[test]
fn test_parse_proxy_url_invalid() {
    let result = parse_proxy_url("not-a-url");
    assert!(result.is_err());
}

#[test]
fn test_socks5_proxy() {
    let proxy_url = "socks5://127.0.0.1:1080".into_proxy().unwrap();
    assert_eq!(proxy_url.scheme(), "socks5");
    assert!(validate_proxy_url(&proxy_url).is_ok());
}