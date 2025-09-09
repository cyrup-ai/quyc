//! HTTP/3 protocol implementation
//!
//! This module provides HTTP/3 specific functionality including QUIC connection management,
//! stream multiplexing, and HTTP/3 protocol optimizations.

pub mod adapter;
pub mod chunks;
pub mod connection;
pub mod strategy;

pub use adapter::execute_h3_request;
pub use chunks::*;
pub use connection::{H3Connection, H3Stream};
