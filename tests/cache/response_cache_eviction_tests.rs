use http::{Response, StatusCode};
use quyc_client::cache::response_cache::eviction::*;
use quyc_client::cache::{cache_config::CacheConfig, cache_entry::CacheEntry};
use quyc_client::prelude::*;
use std::sync::atomic::Ordering;

fn create_test_response() -> HttpResponse {
    Response::builder().status(StatusCode::OK).body(()).expect("Response creation should succeed in test")
}

#[test]
fn test_cleanup_expired_no_entries() {
    let cache = ResponseCache::default();
    cache.cleanup_expired();

    let (entries, _, _) = cache.size_info();
    assert_eq!(entries, 0);
}

#[test]
fn test_evict_lru_entries_empty_cache() {
    let cache = ResponseCache::default();
    let evicted = cache.evict_lru_entries();

    assert_eq!(evicted, 0);
}

#[test]
fn test_cleanup_running_flag() {
    let cache = ResponseCache::default();

    // Set cleanup running flag manually for test
    cache.cleanup_running.store(true, Ordering::Relaxed);

    // This should return early due to flag
    cache.cleanup_expired();

    // Reset flag
    cache.cleanup_running.store(false, Ordering::Relaxed);
}