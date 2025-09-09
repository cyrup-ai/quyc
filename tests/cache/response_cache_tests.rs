use http::{Response, StatusCode};
use quyc_client::cache::cache_config::CacheConfig;
use quyc_client::cache::cache_key::CacheKey;
use quyc_client::cache::response_cache::ResponseCache;
use quyc_client::prelude::*;

fn create_test_response() -> HttpResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("etag", "\"test-etag\"")
        .body(())
        .unwrap()
}

#[test]
fn test_module_integration() {
    // Test that all modules are properly integrated
    let cache = ResponseCache::default();
    let (entries, memory, _) = cache.size_info();
    assert_eq!(entries, 0);
    assert_eq!(memory, 0);
}

#[test]
fn test_cache_operations_integration() {
    let cache = ResponseCache::default();
    let response = create_test_response();
    let key = CacheKey::new("GET", "http://example.com", &[]);

    // Test should_cache
    assert!(cache.should_cache(&response));

    // Test put and get
    cache.put(key.clone(), response.clone());
    let cached = cache.get(&key);
    assert!(cached.is_some());
}

#[test]
fn test_eviction_integration() {
    let config = CacheConfig {
        max_entries: 1,
        max_memory_bytes: 1024,
        default_ttl_seconds: 3600,
    };
    let cache = ResponseCache::new(config);

    // Test cleanup operations
    cache.cleanup_expired();
    cache.clear();

    let (entries, memory, _) = cache.size_info();
    assert_eq!(entries, 0);
    assert_eq!(memory, 0);
}