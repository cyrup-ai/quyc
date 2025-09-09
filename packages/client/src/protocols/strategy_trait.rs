//! Protocol Strategy Trait - Common interface for all protocol implementations
//!
//! This trait defines the contract that all protocol strategies must implement,
//! providing a clean abstraction over HTTP/2, HTTP/3, and other protocols.

use crate::http::{HttpRequest, HttpResponse};

/// Common interface for protocol execution strategies
///
/// Each protocol implementation (H2, H3, QUIC) implements this trait
/// to provide a unified way to execute HTTP requests regardless of
/// the underlying protocol complexity.
pub trait ProtocolStrategy: Send + Sync {
    /// Execute an HTTP request using this protocol strategy
    ///
    /// This method encapsulates ALL protocol-specific complexity:
    /// - Connection establishment (TCP/UDP)
    /// - TLS negotiation
    /// - Protocol handshakes
    /// - Request serialization
    /// - Response streaming
    /// - Error handling
    /// - Connection pooling
    ///
    /// # Arguments
    /// * `request` - The HTTP request to execute
    ///
    /// # Returns
    /// * `HttpResponse` - A high-level response object with streaming internals
    fn execute(&self, request: HttpRequest) -> HttpResponse;
    
    /// Get the protocol name for debugging/logging
    fn protocol_name(&self) -> &'static str;
    
    /// Check if this strategy supports server push
    fn supports_push(&self) -> bool {
        false
    }
    
    /// Get maximum concurrent streams supported
    fn max_concurrent_streams(&self) -> usize {
        100
    }
}