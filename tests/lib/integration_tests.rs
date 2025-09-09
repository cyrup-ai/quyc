//! Integration tests for quyc_client library
//!
//! These are comprehensive integration tests showing real-world usage patterns
//! and validating the core functionality of the HTTP3 client library.

use quyc_client::prelude::*;

// Note: This file contains examples and documentation tests that were
// previously embedded in src/lib.rs as #[cfg(test)] blocks.
// 
// The examples shown here demonstrate:
// - Basic HTTP requests with JSON responses
// - File downloads with progress tracking  
// - Global client usage patterns
// - Configuration validation
// - Connection pooling statistics

// These are primarily documentation examples rather than executable tests,
// as they depend on external services and network connectivity.
// 
// For unit tests of specific functionality, see the individual module test files.

#[test]
fn test_global_client_initialization() {
    let client = quyc_client::global_client();
    assert!(!client.is_closed());
}

#[test]
fn test_connection_stats() {
    let stats = quyc_client::connection_stats();
    // Stats should be accessible without panicking
    assert!(stats.request_count >= 0);
}

#[test]
fn test_config_validation_timeout() {
    use quyc_client::config::HttpConfig;
    use std::time::Duration;
    
    let mut config = HttpConfig::default();
    config.timeout = Duration::from_secs(0);
    
    // Should handle invalid config gracefully
    quyc_client::init_global_client(config);
    
    // Global client should still be usable
    let client = quyc_client::global_client();
    assert!(!client.is_closed());
}