//! Unit tests for compression functionality
//!
//! These tests verify the core compression features without requiring 
//! the full HTTP client pipeline to compile.

use quyc_client::config::HttpConfig;
use quyc_client::http::headers::{
    build_accept_encoding_header, CompressionAlgorithm
};
use http::{HeaderMap, HeaderValue};

#[test]
fn test_accept_encoding_generation() {
    let mut config = HttpConfig::default();
    config.gzip_enabled = true;
    config.brotli_enabled = true;
    config.deflate = true;

    let header = build_accept_encoding_header(&config);
    assert!(header.is_some());
    
    let header_str = header.unwrap().to_str().unwrap();
    assert!(header_str.contains("gzip"));
    assert!(header_str.contains("br"));
    assert!(header_str.contains("deflate"));
    assert!(header_str.contains("identity"));
}

#[test]
fn test_compression_algorithm_enum() {
    assert_eq!(CompressionAlgorithm::Gzip.encoding_name(), "gzip");
    assert_eq!(CompressionAlgorithm::Brotli.encoding_name(), "br");
    assert_eq!(CompressionAlgorithm::Deflate.encoding_name(), "deflate");
    assert_eq!(CompressionAlgorithm::Identity.encoding_name(), "identity");
}

#[test]
fn test_config_defaults() {
    let config = HttpConfig::default();
    assert!(config.gzip_enabled);
    assert!(config.brotli_enabled);
    assert!(config.deflate);
    assert!(config.response_compression);
    assert!(config.request_compression);
}