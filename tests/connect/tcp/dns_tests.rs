use quyc_client::connect::tcp::dns::*;

#[test]
fn test_resolve_ip_address() {
    let result = resolve_host_sync("127.0.0.1", 8080);
    assert!(result.is_ok());
    let addrs = result.expect("Failed to resolve IP address 127.0.0.1");
    assert_eq!(addrs.len(), 1);
    assert_eq!(addrs[0].port(), 8080);
}

#[test]
fn test_resolve_ipv6_address() {
    let result = resolve_host_sync("::1", 8080);
    assert!(result.is_ok());
    let addrs = result.expect("Failed to resolve IPv6 address ::1");
    assert_eq!(addrs.len(), 1);
    assert_eq!(addrs[0].port(), 8080);
}

#[test]
fn test_resolve_localhost() {
    let result = resolve_host_sync("localhost", 80);
    assert!(result.is_ok());
    let addrs = result.expect("Failed to resolve localhost");
    assert!(!addrs.is_empty());
}

#[test]
fn test_resolve_invalid_host() {
    let result = resolve_host_sync("invalid.nonexistent.domain.test", 80);
    assert!(result.is_err());
    assert!(result.expect_err("Expected DNS resolution to fail for invalid host").contains("DNS resolution failed"));
}

#[test]
fn test_resolve_empty_host() {
    let result = resolve_host_sync("", 80);
    assert!(result.is_err());
}