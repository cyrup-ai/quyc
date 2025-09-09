//! DNS resolution traits and type aliases
//!
//! Contains the core Resolve trait and related type definitions
//! for the DNS resolution system.

use ystream::AsyncStream;

use super::types::{DnsResult, HyperName};

/// Trait for DNS resolution in streams-first architecture.
/// Provides asynchronous DNS resolution using AsyncStream instead of Futures.
pub trait Resolve: Send + Sync + 'static {
    /// Resolve a hostname to socket addresses using streams-first architecture.
    /// Returns AsyncStream of DnsResult with error-as-data pattern.
    fn resolve(&self, name: HyperName) -> AsyncStream<DnsResult, 1024>;
}

/// An iterator of resolved socket addresses.
pub type Addrs = DnsResult;

/// Type alias for DNS resolution result streams.
pub type Resolving = AsyncStream<Addrs, 1024>;
