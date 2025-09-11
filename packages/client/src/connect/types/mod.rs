//! HTTP/3 connection types and trait definitions
//!
//! Core connection abstractions with zero-allocation `MessageChunk` implementations.
//! The functionality is organized into logical modules:
//!
//! - `connector`: HTTP/3 connector types and service abstractions
//! - `connection`: HTTP connection wrappers and trait definitions
//! - `tcp_impl`: TCP connection implementations with `MessageChunk` support
//!
//! All modules maintain production-quality code standards with comprehensive error handling.

pub mod connection;
pub mod connector;
pub mod tcp_impl;
pub mod tls_connection;

// Re-export all main types for backward compatibility
pub use connection::{BrokenConnectionImpl, Conn, ConnectionTrait, TlsInfo};
pub use connector::{
    BoxedConnectorLayer, BoxedConnectorService, Connector, ConnectorKind, Unnameable,
};
pub use tcp_impl::{BrokenStream, TcpConnection, TcpStreamWrapper};
pub use tls_connection::TlsConnection;


