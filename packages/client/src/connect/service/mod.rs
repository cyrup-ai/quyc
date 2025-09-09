//! HTTP/3 connector service with zero-allocation connection establishment
//!
//! Decomposed connector service providing TCP, TLS, and proxy connections
//! with elite polling and streaming architecture.

pub mod core;
pub mod direct;
pub mod interface;
pub mod proxy;
// TLS module removed - will be integrated through proper TlsManager

// Re-export main service
pub use core::ConnectorService;
