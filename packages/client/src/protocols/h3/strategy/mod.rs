//! H3 Strategy Module
//!
//! Modular HTTP/3 protocol strategy implementation with logical separation of concerns.
//!
//! ## Architecture
//!
//! - `core`: Main `H3Strategy` struct and `ProtocolStrategy` implementation
//! - `connection`: QUIC connection establishment and UDP socket management
//! - `processing`: HTTP/3 request sending and response processing
//! - `security`: Address validation and security measures
//!
//! ## Re-exports
//!
//! The main `H3Strategy` struct is re-exported for compatibility with existing code.

pub mod core;
pub mod connection;
pub mod processing;
pub mod security;

// Re-export the main strategy for backwards compatibility
pub use core::H3Strategy;