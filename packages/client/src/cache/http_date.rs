//! HTTP date parsing and formatting utilities
//!
//! Provides production-ready HTTP date parsing following RFC 7231
//! with comprehensive error handling and multiple format support.

#[allow(unused_imports)]
use std::time::SystemTime;

/// HTTP date parsing error types
#[derive(Debug, Clone)]
pub enum HttpDateParseError {
    /// Date format was not recognized by any of the supported parsers
    UnrecognizedFormat(String),
    /// Date was parsed but represents a time before Unix epoch
    InvalidTimestamp(String),
}

impl std::fmt::Display for HttpDateParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpDateParseError::UnrecognizedFormat(date) => {
                write!(f, "Unrecognized HTTP date format: {}", date)
            }
            HttpDateParseError::InvalidTimestamp(date) => {
                write!(f, "Invalid timestamp in HTTP date: {}", date)
            }
        }
    }
}

impl std::error::Error for HttpDateParseError {}

/// HTTP date parsing utilities
pub mod httpdate {
    use std::time::{Duration, SystemTime};

    use super::HttpDateParseError;

    /// Parse HTTP date string into SystemTime following RFC 7231 formats
    pub fn parse_http_date(date_str: &str) -> Result<SystemTime, HttpDateParseError> {
        use chrono::{DateTime, NaiveDateTime};

        // RFC 7231 Section 7.1.1.1: HTTP-date format preferences
        // 1. IMF-fixdate (preferred): "Sun, 06 Nov 1994 08:49:37 GMT"
        // 2. RFC 850 format: "Sunday, 06-Nov-94 08:49:37 GMT"
        // 3. ANSI C asctime() format: "Sun Nov  6 08:49:37 1994"

        // Try IMF-fixdate format first (RFC 7231 preferred format)
        if let Ok(dt) = DateTime::parse_from_str(date_str, "%a, %d %b %Y %H:%M:%S GMT") {
            let timestamp = dt.timestamp();
            if timestamp >= 0 {
                return Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp as u64));
            }
        }

        // Try RFC 850 format with 2-digit year
        if let Ok(dt) = DateTime::parse_from_str(date_str, "%A, %d-%b-%y %H:%M:%S GMT") {
            let timestamp = dt.timestamp();
            if timestamp >= 0 {
                return Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp as u64));
            }
        }

        // Try ANSI C asctime() format (no timezone, assume GMT)
        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%a %b %e %H:%M:%S %Y") {
            let timestamp = dt.and_utc().timestamp();
            if timestamp >= 0 {
                return Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp as u64));
            }
        }

        // Try RFC 2822 format as fallback
        if let Ok(dt) = DateTime::parse_from_rfc2822(date_str) {
            let timestamp = dt.timestamp();
            if timestamp >= 0 {
                return Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp as u64));
            }
        }

        Err(HttpDateParseError::UnrecognizedFormat(date_str.to_string()))
    }

    /// Format SystemTime as HTTP date string in RFC 7231 IMF-fixdate format
    pub fn fmt_http_date(time: SystemTime) -> String {
        use chrono::{DateTime, Utc};

        let duration = time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();

        let dt = DateTime::<Utc>::from_timestamp(duration.as_secs() as i64, 0).unwrap_or_default();

        dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    }
}
