use quyc_client::http::resolver::validation::*;

#[test]
fn test_valid_hostnames() {
    assert!(validate_hostname("example.com").is_ok());
    assert!(validate_hostname("sub.example.com").is_ok());
    assert!(validate_hostname("test-host.example.org").is_ok());
    assert!(validate_hostname("a.b.c.d").is_ok());
}

#[test]
fn test_invalid_hostnames() {
    assert!(validate_hostname("").is_err());
    assert!(validate_hostname("-invalid.com").is_err());
    assert!(validate_hostname("invalid-.com").is_err());
    assert!(validate_hostname("invalid..com").is_err());
    assert!(validate_hostname("invalid_hostname.com").is_err());
}

#[test]
fn test_hostname_length_limits() {
    let long_hostname = "a".repeat(254);
    assert!(validate_hostname(&long_hostname).is_err());

    let long_label = format!("{}.com", "a".repeat(64));
    assert!(validate_hostname(&long_label).is_err());
}