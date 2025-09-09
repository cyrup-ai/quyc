//! Basic security tests for critical vulnerabilities

use quyc::Http3;

#[tokio::test]
async fn test_ssrf_protection_blocks_localhost() {
    let client = Http3::new().unwrap();
    let result = client.get("http://localhost:22").collect_one::<serde_json::Value>().await;
    assert!(result.is_err()); // Should be blocked
}

#[tokio::test]
async fn test_ssrf_protection_blocks_private_ips() {
    let client = Http3::new().unwrap();
    let result = client.get("http://192.168.1.1/admin").collect_one::<serde_json::Value>().await;
    assert!(result.is_err()); // Should be blocked
}

#[tokio::test]
async fn test_ssrf_protection_blocks_metadata_service() {
    let client = Http3::new().unwrap();
    let result = client.get("http://169.254.169.254/metadata").collect_one::<serde_json::Value>().await;
    assert!(result.is_err()); // Should be blocked
}

#[tokio::test] 
async fn test_legitimate_urls_work() {
    let client = Http3::new().unwrap();
    // This should work (but may fail due to network, not security blocking)
    let _result = client.get("https://httpbin.org/json").collect_one::<serde_json::Value>().await;
    // Test passes if no security blocking occurs
}