//! Quiche QUIC protocol implementation
//!
//! This module provides Quiche-specific QUIC functionality including connection management,
//! packet handling, and Quiche library integration.

pub mod chunks;
pub mod h3_adapter;
pub mod h3_quiche;
pub mod streaming;

pub use chunks::{QuichePacketChunk, QuicheReadableChunk, QuicheStreamChunk, QuicheWriteResult};
pub use streaming::QuicheConnectionChunk;
pub use streaming::*;
