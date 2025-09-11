//! Headers Management Module - Using standard http crate types

use base64::{Engine as _, engine::general_purpose};
use http::{HeaderMap, HeaderName, HeaderValue, header};
use thiserror::Error;

/// A wrapper around `http::HeaderMap` to provide fluent, application-specific helpers.
#[derive(Debug, Clone, Default)]
pub struct HeaderManager {
    headers: HeaderMap,
    /// Internal error state for deferred error handling
    error: Option<String>,
}

impl HeaderManager {
    /// Creates a new, empty `HeaderManager`.
    #[must_use] 
    pub fn new() -> Self {
        HeaderManager {
            headers: HeaderMap::new(),
            error: None,
        }
    }

    /// Sets a header, consuming the manager and returning a new one.
    pub fn set(mut self, key: HeaderName, value: HeaderValue) -> Self {
        // If there's already an error, preserve it
        if self.error.is_some() {
            return self;
        }
        self.headers.insert(key, value);
        self
    }

    /// Sets the Content-Type header.
    pub fn content_type(self, content_type: &str) -> Self {
        match HeaderValue::from_str(content_type) {
            Ok(value) => self.set(header::CONTENT_TYPE, value),
            Err(e) => Self {
                headers: self.headers,
                error: Some(format!("Invalid Content-Type header: {e}")),
            },
        }
    }

    /// Sets the Authorization header with a bearer token.
    pub fn bearer_token(self, token: &str) -> Self {
        let auth_header = format!("Bearer {token}");
        match HeaderValue::from_str(&auth_header) {
            Ok(value) => self.set(header::AUTHORIZATION, value),
            Err(e) => Self {
                headers: self.headers,
                error: Some(format!("Invalid bearer token: {e}")),
            },
        }
    }

    /// Sets basic authentication.
    pub fn basic_auth(self, user: &str, pass: Option<&str>) -> Self {
        let credentials = format!("{}:{}", user, pass.unwrap_or_default());
        let encoded = general_purpose::STANDARD.encode(credentials);
        let auth_header = format!("Basic {encoded}");
        match HeaderValue::from_str(&auth_header) {
            Ok(value) => self.set(header::AUTHORIZATION, value),
            Err(e) => Self {
                headers: self.headers,
                error: Some(format!("Invalid basic auth credentials: {e}")),
            },
        }
    }

    /// Returns the underlying `HeaderMap`.
    #[must_use] 
    pub fn build(self) -> HeaderMap {
        self.headers
    }

    /// Check if there were any header validation errors
    #[must_use] 
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Get the error message if any
    #[must_use] 
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// Header-related errors.
#[derive(Debug, Clone, Error)]
pub enum HeaderError {
    /// Represents an error when a header value is invalid.
    #[error("Invalid header value: {message}")]
    InvalidHeaderValue {
        /// Error message describing the invalid header value
        message: String,
    },
}

impl From<http::header::InvalidHeaderValue> for HeaderError {
    fn from(err: http::header::InvalidHeaderValue) -> Self {
        HeaderError::InvalidHeaderValue {
            message: err.to_string(),
        }
    }
}

// Additional header utilities merged from util/header_utils.rs

/// Parse headers from string format
#[inline]
pub fn parse_headers(header_str: &str) -> Result<HeaderMap, crate::error::HttpError> {
    let mut headers = HeaderMap::new();

    for line in header_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim();
            let value = value.trim();

            let header_name = create_header_name(name)?;
            let header_value = create_header_value(value)?;

            headers.insert(header_name, header_value);
        }
    }

    Ok(headers)
}

/// Format headers as string representation
#[inline]
#[must_use] 
pub fn format_headers(headers: &HeaderMap) -> String {
    let mut result = String::new();

    for (name, value) in headers {
        result.push_str(name.as_str());
        result.push_str(": ");
        if let Ok(value_str) = value.to_str() {
            result.push_str(value_str);
        }
        result.push('\n');
    }

    result
}

/// Validate header name and value combination
#[inline]
pub fn validate_header(name: &str, value: &str) -> Result<(), crate::error::HttpError> {
    create_header_name(name)?;
    create_header_value(value)?;
    Ok(())
}

/// Create header value from string
#[inline]
pub fn create_header_value(value: &str) -> Result<HeaderValue, crate::error::HttpError> {
    HeaderValue::from_str(value)
        .map_err(|e| crate::error::invalid_header(format!("Invalid header value: {e}")))
}

/// Create header name from string
#[inline]
pub fn create_header_name(name: &str) -> Result<HeaderName, crate::error::HttpError> {
    HeaderName::from_bytes(name.as_bytes())
        .map_err(|e| crate::error::invalid_header(format!("Invalid header name: {e}")))
}

/// Merge header maps with conflict resolution
#[inline]
pub fn merge_headers(base: &mut HeaderMap, additional: HeaderMap) {
    for (name, value) in additional {
        if let Some(name) = name {
            base.insert(name, value);
        }
    }
}

/// Extract content type from headers
#[inline]
#[must_use] 
pub fn extract_content_type(headers: &HeaderMap) -> Option<&str> {
    headers.get("content-type").and_then(|v| v.to_str().ok())
}

/// Check if headers indicate compressed content
#[inline]
#[must_use] 
pub fn is_compressed(headers: &HeaderMap) -> bool {
    headers
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|encoding| !matches!(encoding, "identity" | ""))
}

/// Compression algorithms supported by the client
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    Gzip,
    Brotli,
    Deflate,
    Identity,
}

impl CompressionAlgorithm {
    /// Get the HTTP encoding name for this algorithm
    #[inline]
    #[must_use] 
    pub fn encoding_name(&self) -> &'static str {
        match self {
            CompressionAlgorithm::Gzip => "gzip",
            CompressionAlgorithm::Brotli => "br",
            CompressionAlgorithm::Deflate => "deflate", 
            CompressionAlgorithm::Identity => "identity",
        }
    }

    /// Parse compression algorithm from encoding string
    #[inline]
    #[must_use] 
    pub fn from_encoding(encoding: &str) -> Option<Self> {
        match encoding.to_lowercase().as_str() {
            "gzip" | "x-gzip" => Some(CompressionAlgorithm::Gzip),
            "br" => Some(CompressionAlgorithm::Brotli),
            "deflate" => Some(CompressionAlgorithm::Deflate),
            "identity" | "" => Some(CompressionAlgorithm::Identity),
            _ => None,
        }
    }

    /// Check if client supports decompression for this algorithm
    #[inline]
    #[must_use] 
    pub fn is_supported(&self, config: &crate::config::HttpConfig) -> bool {
        match self {
            CompressionAlgorithm::Gzip => config.gzip_enabled,
            CompressionAlgorithm::Brotli => config.brotli_enabled,
            CompressionAlgorithm::Deflate => config.deflate,
            CompressionAlgorithm::Identity => true,
        }
    }
}

/// Detect compression algorithm from response headers
#[inline]
pub fn detect_compression_algorithm(headers: &HeaderMap) -> Option<CompressionAlgorithm> {
    headers
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .and_then(CompressionAlgorithm::from_encoding)
}

/// Check if response needs decompression based on headers and config
#[inline]
#[must_use] 
pub fn needs_decompression(headers: &HeaderMap, config: &crate::config::HttpConfig) -> Option<CompressionAlgorithm> {
    if !config.response_compression {
        return None;
    }

    let algorithm = detect_compression_algorithm(headers)?;
    
    // Only decompress if we support it and it's not identity
    if algorithm != CompressionAlgorithm::Identity && algorithm.is_supported(config) {
        Some(algorithm)
    } else {
        None
    }
}

/// Get content length from headers
#[inline]
#[must_use] 
pub fn get_content_length(headers: &HeaderMap) -> Option<u64> {
    headers
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
}

/// Replace headers function for compatibility
#[inline]
pub fn replace_headers(headers: &mut HeaderMap, new_headers: HeaderMap) {
    headers.clear();
    headers.extend(new_headers);
}

/// Build Accept-Encoding header value based on compression configuration
#[inline] 
#[must_use] 
pub fn build_accept_encoding_header(config: &crate::config::HttpConfig) -> Option<HeaderValue> {
    let mut encodings = Vec::new();
    
    if config.gzip_enabled {
        encodings.push("gzip");
    }
    
    if config.brotli_enabled {
        encodings.push("br");
    }
    
    if config.deflate {
        encodings.push("deflate");
    }
    
    // Always include identity as fallback
    encodings.push("identity");
    
    if encodings.is_empty() {
        None
    } else {
        let encoding_str = encodings.join(", ");
        HeaderValue::from_str(&encoding_str).ok()
    }
}

/// Add compression headers to request based on configuration
#[inline]
pub fn add_compression_headers(headers: &mut HeaderMap, config: &crate::config::HttpConfig) {
    // Only add Accept-Encoding if response compression is enabled
    if config.response_compression
        && let Some(accept_encoding) = build_accept_encoding_header(config) {
            headers.insert(http::header::ACCEPT_ENCODING, accept_encoding);
        }
}
