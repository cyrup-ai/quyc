//! DNS resolution error types
//!
//! This module provides comprehensive error handling for DNS resolution operations.

/// DNS resolution errors with detailed granularity
#[derive(Debug, thiserror::Error)]
pub enum ResolverError {
    #[error("DNS resolution timeout")]
    Timeout,
    #[error("No addresses found for hostname")]
    NoAddresses,
    #[error("DNS query returned empty result set")]
    EmptyResult,
    #[error("DNS lookup failed - invalid hostname or DNS server error")]
    LookupFailed,
    #[error("Invalid hostname: {0}")]
    InvalidHostname(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("DNS server error: {0}")]
    DnsServerError(String),
    #[error("Network error during DNS resolution: {0}")]
    NetworkError(String),
}

impl ResolverError {
    /// Create a new InvalidHostname error
    pub fn invalid_hostname(msg: impl Into<String>) -> Self {
        Self::InvalidHostname(msg.into())
    }

    /// Create a new DnsServerError
    pub fn dns_server_error(msg: impl Into<String>) -> Self {
        Self::DnsServerError(msg.into())
    }

    /// Create a new NetworkError
    pub fn network_error(msg: impl Into<String>) -> Self {
        Self::NetworkError(msg.into())
    }
}
