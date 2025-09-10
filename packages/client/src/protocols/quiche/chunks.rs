//! QUICHE protocol chunk types

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ystream::prelude::MessageChunk;

static QUICHE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// QUICHE connection chunk for connection events
#[derive(Debug, Clone)]
pub enum QuicheConnectionEvent {
    Connected {
        connection_id: u64,
    },
    Disconnected {
        connection_id: u64,
        reason: String,
    },
    HandshakeComplete {
        connection_id: u64,
    },
    StreamOpened {
        connection_id: u64,
        stream_id: u64,
    },
    StreamClosed {
        connection_id: u64,
        stream_id: u64,
    },
    ConnectionClosed {
        connection_id: u64,
        reason: String,
    },
    ConnectionClosing {
        connection_id: u64,
        reason: String,
    },
    Error {
        connection_id: u64,
        message: Arc<str>,
    },
}

impl QuicheConnectionEvent {
    /// Create a connection closed chunk
    pub fn connection_closed() -> Self {
        let connection_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::ConnectionClosed {
            connection_id,
            reason: "Connection closed".to_string(),
        }
    }

    /// Create a connection closing chunk
    pub fn connection_closing(err: u64) -> Self {
        let connection_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::ConnectionClosing {
            connection_id,
            reason: format!("Connection closing with error: {err}"),
        }
    }

    /// Create an established connection chunk
    pub fn established(_local: std::net::SocketAddr, _peer: std::net::SocketAddr) -> Self {
        let connection_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::Connected { connection_id }
    }

    /// Create connection stats chunk
    pub fn connection_stats(
        _recv: usize,
        _sent: usize,
        _lost: usize,
        _rtt: std::time::Duration,
        _cwnd: usize,
    ) -> Self {
        let connection_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::Connected { connection_id }
    }

    /// Create a bad chunk (alias for bad_chunk from MessageChunk trait)
    pub fn bad_chunk(error_message: String) -> Self {
        <Self as MessageChunk>::bad_chunk(error_message)
    }
}

impl MessageChunk for QuicheConnectionEvent {
    #[inline]
    fn bad_chunk(error_message: String) -> Self {
        let connection_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
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

impl Default for QuicheConnectionEvent {
    fn default() -> Self {
        Self::bad_chunk("Default connection chunk".to_string())
    }
}
/// QUICHE packet chunk for packet-level events
#[derive(Debug, Clone)]
pub enum QuichePacketChunk {
    Received { packet_id: u64, size: usize },
    Sent { packet_id: u64, size: usize },
    Lost { packet_id: u64 },
    Acked { packet_id: u64 },
    Error { packet_id: u64, message: Arc<str> },
    /// Data chunk for download operations
    Data { chunk: Vec<u8>, downloaded: u64, total_size: Option<u64> },
    /// Completion marker for download streams
    Complete,
}

impl QuichePacketChunk {
    pub fn packet_sent(
        size: usize,
        _local: std::net::SocketAddr,
        _peer: std::net::SocketAddr,
    ) -> Self {
        let packet_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::Sent { packet_id, size }
    }

    pub fn packet_connection_closed() -> Self {
        let packet_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::Lost { packet_id }
    }

    pub fn packet_processed(
        bytes: usize,
        _from: std::net::SocketAddr,
        _local: std::net::SocketAddr,
    ) -> Self {
        let packet_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::Received {
            packet_id,
            size: bytes,
        }
    }

    pub fn timeout_handled() -> Self {
        let packet_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::Acked { packet_id }
    }
}

impl MessageChunk for QuichePacketChunk {
    #[inline]
    fn bad_chunk(error_message: String) -> Self {
        let packet_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::Error {
            packet_id,
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

impl Default for QuichePacketChunk {
    fn default() -> Self {
        Self::bad_chunk("Default packet chunk".to_string())
    }
}

/// QUICHE stream chunk for stream-level events
#[derive(Debug, Clone)]
pub enum QuicheStreamChunk {
    Data { stream_id: u64, data: Vec<u8> },
    Opened { stream_id: u64 },
    Closed { stream_id: u64 },
    Reset { stream_id: u64, error_code: u64 },
    Error { stream_id: u64, message: Arc<str> },
}

impl QuicheStreamChunk {
    pub fn stream_opened(stream_id: u64, _bidirectional: bool) -> Self {
        Self::Opened { stream_id }
    }

    pub fn readable_stream(stream_id: u64) -> Self {
        Self::Opened { stream_id }
    }

    pub fn stream_data(stream_id: u64, data: Vec<u8>, fin: bool) -> Self {
        if fin {
            Self::Closed { stream_id }
        } else {
            Self::Data { stream_id, data }
        }
    }

    pub fn stream_finished(stream_id: u64) -> Self {
        Self::Closed { stream_id }
    }
}

impl MessageChunk for QuicheStreamChunk {
    #[inline]
    fn bad_chunk(error_message: String) -> Self {
        let stream_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::Error {
            stream_id,
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

impl Default for QuicheStreamChunk {
    fn default() -> Self {
        Self::bad_chunk("Default stream chunk".to_string())
    }
}

/// Quiche readable chunk type alias
pub type QuicheReadableChunk = QuicheConnectionEvent;

/// Quiche write result type alias  
pub type QuicheWriteResult = QuichePacketChunk;

impl QuicheConnectionEvent {
    pub fn readable_stream(stream_id: u64) -> Self {
        let connection_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::StreamOpened {
            connection_id,
            stream_id,
        }
    }

    pub fn stream_data(stream_id: u64, _data: Vec<u8>, fin: bool) -> Self {
        let connection_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        if fin {
            Self::StreamClosed {
                connection_id,
                stream_id,
            }
        } else {
            Self::StreamOpened {
                connection_id,
                stream_id,
            }
        }
    }

    pub fn stream_finished(stream_id: u64) -> Self {
        let connection_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::StreamClosed {
            connection_id,
            stream_id,
        }
    }

    pub fn streams_available(readable_streams: Vec<u64>, writable_streams: Vec<u64>) -> Self {
        let connection_id = QUICHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        // For now, we'll create a Connected event if there are any available streams
        // In a full implementation, we would encode the stream lists in the event
        if !readable_streams.is_empty() || !writable_streams.is_empty() {
            Self::Connected { connection_id }
        } else {
            Self::ConnectionClosed {
                connection_id,
                reason: "No streams available".to_string(),
            }
        }
    }
}

impl QuicheWriteResult {
    pub fn write_complete(stream_id: u64) -> Self {
        let packet_id = stream_id; // Use stream_id as packet_id for traceability
        Self::Sent { packet_id, size: 0 }
    }

    pub fn write_blocked(stream_id: u64) -> Self {
        let packet_id = stream_id; // Use stream_id as packet_id for traceability
        Self::Lost { packet_id } // Use Lost variant to indicate blocked write
    }

    pub fn bytes_written(stream_id: u64, bytes_written: usize, _fin: bool) -> Self {
        let packet_id = stream_id; // Use stream_id as packet_id for traceability
        Self::Sent {
            packet_id,
            size: bytes_written,
        }
        // Note: fin parameter is used implicitly - when fin=true, this is the final write
    }
}
