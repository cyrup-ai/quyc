//! HTTP/2 protocol implementation
//!
//! This module provides HTTP/2 specific functionality including connection management,
//! stream handling, and protocol-specific optimizations.

pub mod adapter;
pub mod chunks;
pub mod connection;
pub mod implementation;
pub mod streaming;
pub mod strategy;

pub use adapter::execute_h2_request;
pub use chunks::*;
pub use connection::{H2Connection, H2Stream};
pub use implementation::H2Chunk;
pub use streaming::H2ConnectionManager;
