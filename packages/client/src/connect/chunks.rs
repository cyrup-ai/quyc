//! TCP connection chunk types for fluent-ai streaming architecture
//!
//! Provides MessageChunk implementations for TCP connection events,
//! following the streams-first architecture with no Result wrapping.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ystream::prelude::MessageChunk;

static TCP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// TCP connection chunk for connection events
#[derive(Debug)]
pub enum TcpConnectionChunk {
    Connected {
        connection_id: u64,
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
        stream: Option<Box<dyn crate::connect::types::ConnectionTrait + Send>>,
    },
    Disconnected {
        connection_id: u64,
        reason: String,
    },
    DataReceived {
        connection_id: u64,
        data: Vec<u8>,
    },
    DataSent {
        connection_id: u64,
        bytes_sent: usize,
    },
    ConnectionClosed {
        connection_id: u64,
        reason: String,
    },
    Error {
        connection_id: u64,
        message: Arc<str>,
    },
}

impl Clone for TcpConnectionChunk {
    fn clone(&self) -> Self {
        match self {
            Self::Connected { connection_id, local_addr, remote_addr, stream: _ } => {
                // Cannot clone the stream, so create a new Connected without stream
                Self::Connected {
                    connection_id: *connection_id,
                    local_addr: *local_addr,
                    remote_addr: *remote_addr,
                    stream: None, // Stream cannot be cloned
                }
            },
            Self::Disconnected { connection_id, reason } => {
                Self::Disconnected { connection_id: *connection_id, reason: reason.clone() }
            },
            Self::DataReceived { connection_id, data } => {
                Self::DataReceived { connection_id: *connection_id, data: data.clone() }
            },
            Self::DataSent { connection_id, bytes_sent } => {
                Self::DataSent { connection_id: *connection_id, bytes_sent: *bytes_sent }
            },
            Self::ConnectionClosed { connection_id, reason } => {
                Self::ConnectionClosed { connection_id: *connection_id, reason: reason.clone() }
            },
            Self::Error { connection_id, message } => {
                Self::Error { connection_id: *connection_id, message: message.clone() }
            },
        }
    }
}

impl MessageChunk for TcpConnectionChunk {
    #[inline]
    fn bad_chunk(error_message: String) -> Self {
        let connection_id = TCP_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::Error {
            connection_id,
            message: Arc::from(error_message),
        }
    }

    #[inline]
    fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            Self::Error { message, .. } => Some(message),
            _ => None,
        }
    }
}

impl Default for TcpConnectionChunk {
    fn default() -> Self {
        Self::bad_chunk("Default TcpConnectionChunk".to_string())
    }
}

impl TcpConnectionChunk {
    /// Get the connection ID for this chunk
    pub fn connection_id(&self) -> u64 {
        match self {
            Self::Connected { connection_id, .. }
            | Self::Disconnected { connection_id, .. }
            | Self::DataReceived { connection_id, .. }
            | Self::DataSent { connection_id, .. }
            | Self::ConnectionClosed { connection_id, .. }
            | Self::Error { connection_id, .. } => *connection_id,
        }
    }

    /// Create a new connected chunk
    pub fn connected(local_addr: SocketAddr, remote_addr: SocketAddr, stream: Option<Box<dyn crate::connect::types::ConnectionTrait + Send>>) -> Self {
        let connection_id = TCP_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::Connected {
            connection_id,
            local_addr,
            remote_addr,
            stream,
        }
    }

    /// Create a disconnected chunk
    pub fn disconnected(connection_id: u64, reason: String) -> Self {
        Self::Disconnected {
            connection_id,
            reason,
        }
    }

    /// Create a data received chunk
    pub fn data_received(connection_id: u64, data: Vec<u8>) -> Self {
        Self::DataReceived {
            connection_id,
            data,
        }
    }

    /// Create a data sent chunk
    pub fn data_sent(connection_id: u64, bytes_sent: usize) -> Self {
        Self::DataSent {
            connection_id,
            bytes_sent,
        }
    }

    /// Create a connection closed chunk
    pub fn connection_closed(connection_id: u64, reason: String) -> Self {
        Self::ConnectionClosed {
            connection_id,
            reason,
        }
    }
}