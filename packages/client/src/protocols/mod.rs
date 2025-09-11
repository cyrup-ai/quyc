//! Direct protocol implementations using ystream `AsyncStream`
//!
//! NO middleware, NO Futures, NO abstractions - pure streaming protocols

#![allow(dead_code)]

pub mod connection;
pub mod core;
pub mod frames;
pub mod h2;
pub mod h3;
pub mod intelligence;
pub mod quiche;
pub mod response_converter;
pub mod strategy;
pub mod strategy_trait;
pub mod auto_strategy;
pub mod transport;
pub mod wire;

// Re-export protocol types
pub use core::{HttpVersion, TimeoutConfig};

// Re-export intelligence cache
pub use intelligence::{ProtocolIntelligence, DomainCapabilities, IntelligenceConfig};

// Re-export connection types
pub use connection::{Connection, ConnectionManager};
pub use h2::{H2Connection, H2Stream};
pub use h3::{H3Connection, H3Stream};
pub use quiche::{QuicheConnectionChunk, QuichePacketChunk, QuicheStreamChunk};

// Re-export configuration types
pub use strategy::{H2Config, H3Config, QuicheConfig, HttpProtocolStrategy};

// Re-export transport types
pub use transport::{TransportConnection, TransportManager, TransportType};
pub use wire::{H2FrameParser, H3FrameParser};

// Re-export response conversion utilities
pub use response_converter::convert_http_chunks_to_response;

// Include tests
#[cfg(test)]
mod wire_tests;
