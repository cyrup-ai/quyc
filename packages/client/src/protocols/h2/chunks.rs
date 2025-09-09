use std::sync::Arc;

use ystream::prelude::MessageChunk;
// prelude import removed - not used

// H2 Connection Chunk - TOKIO-FREE IMPLEMENTATION
#[derive(Debug, Clone)]
pub enum H2ConnectionChunk {
    Ready,
    ConnectionError { message: Arc<str> },
}

impl Default for H2ConnectionChunk {
    fn default() -> Self {
        Self::Ready
    }
}

impl MessageChunk for H2ConnectionChunk {
    #[inline]
    fn bad_chunk(error: String) -> Self {
        H2ConnectionChunk::ConnectionError {
            message: Arc::from(error.as_str()),
        }
    }

    #[inline]
    fn is_error(&self) -> bool {
        matches!(self, H2ConnectionChunk::ConnectionError { .. })
    }

    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            H2ConnectionChunk::ConnectionError { message } => Some(message),
            _ => None,
        }
    }
}

impl H2ConnectionChunk {
    #[inline]
    pub fn ready() -> Self {
        H2ConnectionChunk::Ready
    }
}

// H2 Request Chunk
#[derive(Debug, Clone)]
pub enum H2RequestChunk {
    Sent {
        stream_id: u32,
        connection_id: Arc<str>,
    },
    SendError {
        message: Arc<str>,
    },
}

impl MessageChunk for H2RequestChunk {
    #[inline]
    fn bad_chunk(error: String) -> Self {
        H2RequestChunk::SendError {
            message: Arc::from(error.as_str()),
        }
    }

    #[inline]
    fn is_error(&self) -> bool {
        matches!(self, H2RequestChunk::SendError { .. })
    }

    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            H2RequestChunk::SendError { message } => Some(message),
            _ => None,
        }
    }
}

impl H2RequestChunk {
    #[inline]
    pub fn sent(stream_id: u32, connection_id: Arc<str>) -> Self {
        H2RequestChunk::Sent {
            stream_id,
            connection_id,
        }
    }
}

// H2 Data Chunk
#[derive(Debug, Clone)]
pub enum H2DataChunk {
    Data { bytes: bytes::Bytes },
    StreamComplete,
    DataError { message: Arc<str> },
}

impl Default for H2DataChunk {
    fn default() -> Self {
        Self::StreamComplete
    }
}

impl MessageChunk for H2DataChunk {
    #[inline]
    fn bad_chunk(error: String) -> Self {
        H2DataChunk::DataError {
            message: Arc::from(error.as_str()),
        }
    }

    #[inline]
    fn is_error(&self) -> bool {
        matches!(self, H2DataChunk::DataError { .. })
    }

    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            H2DataChunk::DataError { message } => Some(message),
            _ => None,
        }
    }
}

impl H2DataChunk {
    #[inline]
    pub fn from_bytes(bytes: bytes::Bytes) -> Self {
        H2DataChunk::Data { bytes }
    }

    #[inline]
    pub fn stream_complete() -> Self {
        H2DataChunk::StreamComplete
    }
}

// H2 Send Result
#[derive(Debug, Clone)]
pub enum H2SendResult {
    DataSent,
    SendComplete,
    SendError { message: String },
}

impl Default for H2SendResult {
    fn default() -> Self {
        Self::SendComplete
    }
}

impl MessageChunk for H2SendResult {
    #[inline]
    fn bad_chunk(error: String) -> Self {
        H2SendResult::SendError { message: error }
    }

    #[inline]
    fn is_error(&self) -> bool {
        matches!(self, H2SendResult::SendError { .. })
    }

    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            H2SendResult::SendError { message } => Some(message),
            _ => None,
        }
    }
}

impl H2SendResult {
    #[inline]
    pub fn data_sent() -> Self {
        H2SendResult::DataSent
    }

    #[inline]
    pub fn send_complete() -> Self {
        H2SendResult::SendComplete
    }
}
