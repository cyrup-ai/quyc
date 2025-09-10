//! Compression integration tests for HTTP pipeline
//!
//! Tests the complete compression functionality including Accept-Encoding headers,
//! response decompression detection, and integration with H3/H2 protocols.

use quyc_client::config::HttpConfig;
use quyc_client::http::headers::{
    build_accept_encoding_header, add_compression_headers, detect_compression_algorithm,
    needs_decompression, CompressionAlgorithm,
};
use quyc_client::client::HttpClient;
use quyc_client::builder::core::Http3Builder;
use http::{HeaderMap, HeaderValue, Method, Uri};

/// Test Accept-Encoding header generation based on compression config
#[test]
fn test_accept_encoding_header_generation() {
    // Test with all compression types enabled
    let mut config = HttpConfig::default();
    config.gzip_enabled = true;
    config.brotli_enabled = true;
    config.deflate = true;

    let header = build_accept_encoding_header(&config).unwrap();
    let header_str = header.to_str().unwrap();
    
    assert!(header_str.contains("gzip"));
    assert!(header_str.contains("br"));
    assert!(header_str.contains("deflate"));
    assert!(header_str.contains("identity"));

    // Test with only gzip enabled
    config.gzip_enabled = true;
    config.brotli_enabled = false;
    config.deflate = false;

    let header = build_accept_encoding_header(&config).unwrap();
    let header_str = header.to_str().unwrap();
    
    assert!(header_str.contains("gzip"));
    assert!(!header_str.contains("br"));
    assert!(!header_str.contains("deflate"));
    assert!(header_str.contains("identity"));

    // Test with no compression enabled (should still include identity)
    config.gzip_enabled = false;
    config.brotli_enabled = false;  
    config.deflate = false;

    let header = build_accept_encoding_header(&config).unwrap();
    let header_str = header.to_str().unwrap();
    
    assert!(!header_str.contains("gzip"));
    assert!(!header_str.contains("br"));
    assert!(!header_str.contains("deflate"));
    assert!(header_str.contains("identity"));
}

/// Test compression header addition to requests
#[test]
fn test_compression_headers_addition() {
    let mut headers = HeaderMap::new();
    let mut config = HttpConfig::default();
    config.response_compression = true;
    config.gzip_enabled = true;
    config.brotli_enabled = true;
    config.deflate = true;

    add_compression_headers(&mut headers, &config);

    let accept_encoding = headers.get("accept-encoding").unwrap();
    let header_str = accept_encoding.to_str().unwrap();
    
    assert!(header_str.contains("gzip"));
    assert!(header_str.contains("br"));
    assert!(header_str.contains("deflate"));
    assert!(header_str.contains("identity"));

    // Test that headers are not added when response compression is disabled
    let mut headers_disabled = HeaderMap::new();
    config.response_compression = false;
    
    add_compression_headers(&mut headers_disabled, &config);
    
    assert!(headers_disabled.get("accept-encoding").is_none());
}

/// Test compression algorithm detection from response headers
#[test]
fn test_compression_algorithm_detection() {
    let mut headers = HeaderMap::new();

    // Test gzip detection
    headers.insert("content-encoding", HeaderValue::from_static("gzip"));
    assert_eq!(
        detect_compression_algorithm(&headers),
        Some(CompressionAlgorithm::Gzip)
    );

    // Test brotli detection
    headers.insert("content-encoding", HeaderValue::from_static("br"));
    assert_eq!(
        detect_compression_algorithm(&headers),
        Some(CompressionAlgorithm::Brotli)
    );

    // Test deflate detection
    headers.insert("content-encoding", HeaderValue::from_static("deflate"));
    assert_eq!(
        detect_compression_algorithm(&headers),
        Some(CompressionAlgorithm::Deflate)
    );

    // Test identity detection
    headers.insert("content-encoding", HeaderValue::from_static("identity"));
    assert_eq!(
        detect_compression_algorithm(&headers),
        Some(CompressionAlgorithm::Identity)
    );

    // Test unknown algorithm
    headers.insert("content-encoding", HeaderValue::from_static("unknown"));
    assert_eq!(detect_compression_algorithm(&headers), None);

    // Test no content-encoding header
    headers.remove("content-encoding");
    assert_eq!(detect_compression_algorithm(&headers), None);
}

/// Test decompression need detection based on config and headers
#[test]
fn test_decompression_need_detection() {
    let mut config = HttpConfig::default();
    config.response_compression = true;
    config.gzip_enabled = true;
    config.brotli_enabled = true;
    config.deflate = true;

    let mut headers = HeaderMap::new();

    // Test gzip needs decompression
    headers.insert("content-encoding", HeaderValue::from_static("gzip"));
    assert_eq!(
        needs_decompression(&headers, &config),
        Some(CompressionAlgorithm::Gzip)
    );

    // Test identity doesn't need decompression
    headers.insert("content-encoding", HeaderValue::from_static("identity"));
    assert_eq!(needs_decompression(&headers, &config), None);

    // Test disabled response compression
    config.response_compression = false;
    headers.insert("content-encoding", HeaderValue::from_static("gzip"));
    assert_eq!(needs_decompression(&headers, &config), None);

    // Test disabled gzip support
    config.response_compression = true;
    config.gzip_enabled = false;
    headers.insert("content-encoding", HeaderValue::from_static("gzip"));
    assert_eq!(needs_decompression(&headers, &config), None);
}

/// Test compression algorithm support detection
#[test]
fn test_compression_algorithm_support() {
    let mut config = HttpConfig::default();
    config.gzip_enabled = true;
    config.brotli_enabled = false;
    config.deflate = true;

    assert!(CompressionAlgorithm::Gzip.is_supported(&config));
    assert!(!CompressionAlgorithm::Brotli.is_supported(&config));
    assert!(CompressionAlgorithm::Deflate.is_supported(&config));
    assert!(CompressionAlgorithm::Identity.is_supported(&config)); // Always supported
}

/// Test encoding name retrieval
#[test]
fn test_compression_algorithm_encoding_names() {
    assert_eq!(CompressionAlgorithm::Gzip.encoding_name(), "gzip");
    assert_eq!(CompressionAlgorithm::Brotli.encoding_name(), "br");
    assert_eq!(CompressionAlgorithm::Deflate.encoding_name(), "deflate");
    assert_eq!(CompressionAlgorithm::Identity.encoding_name(), "identity");
}

/// Test compression algorithm parsing from encoding strings
#[test]
fn test_compression_algorithm_parsing() {
    assert_eq!(
        CompressionAlgorithm::from_encoding("gzip"),
        Some(CompressionAlgorithm::Gzip)
    );
    assert_eq!(
        CompressionAlgorithm::from_encoding("x-gzip"),
        Some(CompressionAlgorithm::Gzip)
    );
    assert_eq!(
        CompressionAlgorithm::from_encoding("br"),
        Some(CompressionAlgorithm::Brotli)
    );
    assert_eq!(
        CompressionAlgorithm::from_encoding("deflate"),
        Some(CompressionAlgorithm::Deflate)
    );
    assert_eq!(
        CompressionAlgorithm::from_encoding("identity"),
        Some(CompressionAlgorithm::Identity)
    );
    assert_eq!(
        CompressionAlgorithm::from_encoding(""),
        Some(CompressionAlgorithm::Identity)
    );
    assert_eq!(CompressionAlgorithm::from_encoding("unknown"), None);
}

/// Integration test: HttpClient applies compression headers correctly
#[test]
fn test_http_client_compression_integration() {
    let mut config = HttpConfig::default();
    config.response_compression = true;
    config.gzip_enabled = true;
    config.brotli_enabled = true;
    config.deflate = true;

    let client = HttpClient::with_config(config);
    
    // Create a test request
    let uri: Uri = "https://example.com/test".parse().unwrap();
    let mut request = crate::http::HttpRequest::new(Method::GET, uri.try_into().unwrap(), None, None, None);
    
    // Manually apply compression headers (simulating what HttpClient.execute does)
    quyc_client::http::headers::add_compression_headers(request.headers_mut(), client.config());
    
    // Verify Accept-Encoding header was added
    let accept_encoding = request.headers().get("accept-encoding");
    assert!(accept_encoding.is_some());
    
    let header_str = accept_encoding.unwrap().to_str().unwrap();
    assert!(header_str.contains("gzip"));
    assert!(header_str.contains("br"));
    assert!(header_str.contains("deflate"));
    assert!(header_str.contains("identity"));
}

/// Integration test: HttpConfig presets have compression enabled
#[test]  
fn test_config_presets_compression_settings() {
    // Test AI-optimized config
    let ai_config = HttpConfig::ai_optimized();
    assert!(ai_config.response_compression);
    assert!(ai_config.request_compression);
    assert!(ai_config.gzip_enabled);
    assert!(ai_config.brotli_enabled);
    assert!(ai_config.deflate);

    // Test streaming-optimized config
    let streaming_config = HttpConfig::streaming_optimized();
    assert!(streaming_config.response_compression);
    assert!(streaming_config.request_compression);
    assert!(streaming_config.gzip_enabled);
    assert!(streaming_config.brotli_enabled);
    assert!(streaming_config.deflate);

    // Test batch-optimized config
    let batch_config = HttpConfig::batch_optimized();
    assert!(batch_config.response_compression);
    assert!(batch_config.request_compression);
    assert!(batch_config.gzip_enabled);
    assert!(batch_config.brotli_enabled);
    assert!(batch_config.deflate);

    // Test low-latency config
    let low_latency_config = HttpConfig::low_latency();
    assert!(low_latency_config.response_compression);
    assert!(low_latency_config.request_compression);
    assert!(low_latency_config.gzip_enabled);
    assert!(low_latency_config.brotli_enabled);
    assert!(low_latency_config.deflate);

    // Test default config
    let default_config = HttpConfig::default();
    assert!(default_config.response_compression);
    assert!(default_config.request_compression);
    assert!(default_config.gzip_enabled);
    assert!(default_config.brotli_enabled);
    assert!(default_config.deflate);
}

/// Test HttpClientBuilder compression methods
#[test]
fn test_http_client_builder_compression_methods() {
    use quyc_client::client::configuration::HttpClientBuilder;

    let client_result = HttpClientBuilder::new()
        .gzip(true)
        .brotli(true)
        .deflate(false)
        .build();
    
    assert!(client_result.is_ok());
    let client = client_result.unwrap();
    
    assert!(client.config().gzip_enabled);
    assert!(client.config().brotli_enabled);
    assert!(!client.config().deflate);

    // Test disabling compression
    let client_disabled = HttpClientBuilder::new()
        .gzip(false)
        .brotli(false)
        .deflate(false)
        .build()
        .unwrap();
    
    assert!(!client_disabled.config().gzip_enabled);
    assert!(!client_disabled.config().brotli_enabled);
    assert!(!client_disabled.config().deflate);
}