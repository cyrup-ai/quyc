//! DNS resolution and connection management
//!
//! This module provides production-quality DNS resolution with caching,
//! retry logic, and integrated client resource protection.

pub mod cache;
pub mod config;
pub mod core;
pub mod engine;
pub mod error;
pub mod rate_limiter;
pub mod stats;
pub mod validation;

// Re-export public API for backwards compatibility
pub use core::Resolver;
// Keep the same public interface as the original resolver.rs
// This ensures zero breaking changes for existing code
pub use core::Resolver as DnsResolver;

pub use cache::{DnsCache, DnsCacheEntry};
pub use config::{CacheConfig, RetryConfig};
pub use error::ResolverError;
pub use stats::ResolverStats;
pub use validation::validate_hostname; // Alias for clarity
