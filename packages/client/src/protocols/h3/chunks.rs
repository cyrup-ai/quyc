use std::sync::atomic::{AtomicU64, Ordering};

use bytes::Bytes;
use ystream::prelude::MessageChunk;

static CHUNK_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub enum H3ConnectionChunk {
    ConnectionReady {
        connection_id: u64,
    },
    RecvStreamAccepted {
        stream_id: u64,
        connection_id: u64,
    },
    BiStreamAccepted {
        send_stream_id: u64,
        recv_stream_id: u64,
        connection_id: u64,
    },
    ConnectionClosed {
        connection_id: u64,
        reason: String,
    },
    ConnectionError {
        connection_id: u64,
        error_code: u32,
        message: String,
    },
    ProtocolError {
        connection_id: u64,
        protocol_error: String,
    },
}

impl H3ConnectionChunk {
    #[inline]
    #[must_use] 
    pub fn new_connection_ready(connection_id: u64) -> Self {
        Self::ConnectionReady { connection_id }
    }

    #[inline]
    pub fn new_recv_stream(stream_id: u64) -> Self {
        let connection_id = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::RecvStreamAccepted {
            stream_id,
            connection_id,
        }
    }

    #[inline]
    pub fn new_bidi_stream(send_stream_id: u64, recv_stream_id: u64) -> Self {
        let connection_id = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::BiStreamAccepted {
            send_stream_id,
            recv_stream_id,
            connection_id,
        }
    }

    #[inline]
    #[must_use] 
    pub fn connection_closed(connection_id: u64, reason: String) -> Self {
        Self::ConnectionClosed {
            connection_id,
            reason,
        }
    }
}

impl MessageChunk for H3ConnectionChunk {
    #[inline]
    fn bad_chunk(error_message: String) -> Self {
        let connection_id = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::ConnectionError {
            connection_id,
            error_code: 0,
            message: error_message,
        }
    }

    #[inline]
    fn is_error(&self) -> bool {
        matches!(
            self,
            Self::ConnectionError { .. } | Self::ProtocolError { .. }
        )
    }

    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            Self::ConnectionError { message, .. } => Some(message),
            Self::ProtocolError { protocol_error, .. } => Some(protocol_error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum H3BiStreamChunk {
    BiStreamReady {
        send_stream_id: u64,
        recv_stream_id: u64,
    },
    SendStreamClosed {
        stream_id: u64,
    },
    RecvStreamClosed {
        stream_id: u64,
    },
    StreamError {
        stream_id: u64,
        error_code: u32,
        message: String,
    },
    ProtocolError {
        stream_id: u64,
        protocol_error: String,
    },
}

impl H3BiStreamChunk {
    #[inline]
    #[must_use] 
    pub fn new_bidi_stream(send_stream_id: u64, recv_stream_id: u64) -> Self {
        Self::BiStreamReady {
            send_stream_id,
            recv_stream_id,
        }
    }

    #[inline]
    #[must_use] 
    pub fn send_closed(stream_id: u64) -> Self {
        Self::SendStreamClosed { stream_id }
    }

    #[inline]
    #[must_use] 
    pub fn recv_closed(stream_id: u64) -> Self {
        Self::RecvStreamClosed { stream_id }
    }
}

impl MessageChunk for H3BiStreamChunk {
    #[inline]
    fn bad_chunk(error_message: String) -> Self {
        let stream_id = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::StreamError {
            stream_id,
            error_code: 0,
            message: error_message,
        }
    }

    #[inline]
    fn is_error(&self) -> bool {
        matches!(self, Self::StreamError { .. } | Self::ProtocolError { .. })
    }

    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            Self::StreamError { message, .. } => Some(message),
            Self::ProtocolError { protocol_error, .. } => Some(protocol_error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum H3DataChunk {
    DataReceived {
        data: Bytes,
        stream_id: u64,
        chunk_index: u64,
    },
    StreamComplete {
        stream_id: u64,
        total_bytes: u64,
    },
    StreamReset {
        stream_id: u64,
        error_code: u32,
    },
    DataError {
        stream_id: u64,
        error_code: u32,
        message: String,
    },
    ProtocolError {
        stream_id: u64,
        protocol_error: String,
    },
}

impl H3DataChunk {
    #[inline]
    pub fn from_bytes(data: Bytes) -> Self {
        let stream_id = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        let chunk_index = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::DataReceived {
            data,
            stream_id,
            chunk_index,
        }
    }

    #[inline]
    pub fn stream_complete() -> Self {
        let stream_id = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::StreamComplete {
            stream_id,
            total_bytes: 0,
        }
    }

    #[inline]
    #[must_use] 
    pub fn stream_reset(stream_id: u64, error_code: u32) -> Self {
        Self::StreamReset {
            stream_id,
            error_code,
        }
    }
}

impl MessageChunk for H3DataChunk {
    #[inline]
    fn bad_chunk(error_message: String) -> Self {
        let stream_id = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::DataError {
            stream_id,
            error_code: 0,
            message: error_message,
        }
    }

    #[inline]
    fn is_error(&self) -> bool {
        matches!(
            self,
            Self::StreamReset { .. } | Self::DataError { .. } | Self::ProtocolError { .. }
        )
    }

    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            Self::DataError { message, .. } => Some(message),
            Self::ProtocolError { protocol_error, .. } => Some(protocol_error),
            _ => None,
        }
    }
}

impl Default for H3DataChunk {
    fn default() -> Self {
        Self::DataError {
            stream_id: 0,
            error_code: 0,
            message: "Default error".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum H3SendResult {
    DataSent {
        bytes_sent: u64,
        stream_id: u64,
    },
    SendComplete {
        stream_id: u64,
        total_bytes: u64,
    },
    SendReady {
        stream_id: u64,
    },
    SendError {
        stream_id: u64,
        error_code: u32,
        message: String,
    },
    ProtocolError {
        stream_id: u64,
        protocol_error: String,
    },
}

impl H3SendResult {
    #[inline]
    pub fn data_sent() -> Self {
        let stream_id = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::DataSent {
            bytes_sent: 0,
            stream_id,
        }
    }

    #[inline]
    pub fn send_complete() -> Self {
        let stream_id = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::SendComplete {
            stream_id,
            total_bytes: 0,
        }
    }

    #[inline]
    #[must_use] 
    pub fn send_ready(stream_id: u64) -> Self {
        Self::SendReady { stream_id }
    }
}

impl MessageChunk for H3SendResult {
    #[inline]
    fn bad_chunk(error_message: String) -> Self {
        let stream_id = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::SendError {
            stream_id,
            error_code: 0,
            message: error_message,
        }
    }

    #[inline]
    fn is_error(&self) -> bool {
        matches!(self, Self::SendError { .. } | Self::ProtocolError { .. })
    }

    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            Self::SendError { message, .. } => Some(message),
            Self::ProtocolError { protocol_error, .. } => Some(protocol_error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum H3RequestChunk {
    RequestSent {
        stream_id: u64,
        request_id: u64,
    },
    RequestReady {
        request_id: u64,
    },
    RequestError {
        request_id: u64,
        error_code: u32,
        message: String,
    },
    ProtocolError {
        request_id: u64,
        protocol_error: String,
    },
}

impl H3RequestChunk {
    #[inline]
    pub fn request_sent(stream_id: u64) -> Self {
        let request_id = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::RequestSent {
            stream_id,
            request_id,
        }
    }

    #[inline]
    #[must_use] 
    pub fn request_ready(request_id: u64) -> Self {
        Self::RequestReady { request_id }
    }
}

impl MessageChunk for H3RequestChunk {
    #[inline]
    fn bad_chunk(error_message: String) -> Self {
        let request_id = CHUNK_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::RequestError {
            request_id,
            error_code: 0,
            message: error_message,
        }
    }

    #[inline]
    fn is_error(&self) -> bool {
        matches!(self, Self::RequestError { .. } | Self::ProtocolError { .. })
    }

    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            Self::RequestError { message, .. } => Some(message),
            Self::ProtocolError { protocol_error, .. } => Some(protocol_error),
            _ => None,
        }
    }
}
