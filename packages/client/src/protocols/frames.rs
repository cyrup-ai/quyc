//! HTTP/2 and HTTP/3 frame definitions and processing

use std::sync::atomic::AtomicU64;

use ystream::prelude::MessageChunk;

static FRAME_COUNTER: AtomicU64 = AtomicU64::new(0);

/// HTTP/2 frame types
#[derive(Debug, Clone, PartialEq)]
pub enum H2Frame {
    Data {
        stream_id: u64,
        data: Vec<u8>,
        end_stream: bool,
    },
    Headers {
        stream_id: u64,
        headers: Vec<(String, String)>,
        end_stream: bool,
        end_headers: bool,
    },
    Priority {
        stream_id: u64,
        dependency: u64,
        weight: u8,
        exclusive: bool,
    },
    RstStream {
        stream_id: u64,
        error_code: u32,
    },
    Settings {
        settings: Vec<(u16, u32)>,
    },
    PushPromise {
        stream_id: u64,
        promised_stream_id: u64,
        headers: Vec<(String, String)>,
    },
    Ping {
        data: [u8; 8],
    },
    GoAway {
        last_stream_id: u64,
        error_code: u32,
        debug_data: Vec<u8>,
    },
    WindowUpdate {
        stream_id: u64,
        increment: u32,
    },
    Continuation {
        stream_id: u64,
        headers: Vec<(String, String)>,
        end_headers: bool,
    },
    /// Error frame for bad chunks
    Error {
        message: String,
    },
}

impl MessageChunk for H2Frame {
    fn bad_chunk(error: String) -> Self {
        H2Frame::Error { message: error }
    }

    fn error(&self) -> Option<&str> {
        match self {
            H2Frame::Error { message } => Some(message),
            _ => None,
        }
    }
}

impl Default for H2Frame {
    fn default() -> Self {
        H2Frame::Error {
            message: "Default H2Frame".to_string(),
        }
    }
}

/// HTTP/3 frame types
#[derive(Debug, Clone, PartialEq)]
pub enum H3Frame {
    Data {
        stream_id: u64,
        data: Vec<u8>,
    },
    Headers {
        stream_id: u64,
        headers: Vec<(String, String)>,
    },
    CancelPush {
        push_id: u64,
    },
    Settings {
        settings: Vec<(u64, u64)>,
    },
    PushPromise {
        push_id: u64,
        headers: Vec<(String, String)>,
    },
    GoAway {
        stream_id: u64,
    },
    MaxPushId {
        push_id: u64,
    },
    ConnectionClose {
        error_code: u64,
        reason: String,
    },
    /// Error frame for bad chunks
    Error {
        message: String,
    },
}

impl MessageChunk for H3Frame {
    fn bad_chunk(error: String) -> Self {
        H3Frame::Error { message: error }
    }

    fn error(&self) -> Option<&str> {
        match self {
            H3Frame::Error { message } => Some(message),
            _ => None,
        }
    }
}

impl Default for H3Frame {
    fn default() -> Self {
        H3Frame::Error {
            message: "Default H3Frame".to_string(),
        }
    }
}

/// Generic frame chunk for protocol abstraction
#[derive(Debug, Clone, PartialEq)]
pub enum FrameChunk {
    H2(H2Frame),
    H3(H3Frame),
    ConnectionClosed,
    Error { message: String },
}

impl FrameChunk {
    /// Create H2 frame chunk
    #[inline]
    pub fn h2_frame(frame: H2Frame) -> Self {
        Self::H2(frame)
    }

    /// Create H3 frame chunk
    #[inline]
    pub fn h3_frame(frame: H3Frame) -> Self {
        Self::H3(frame)
    }
}

impl MessageChunk for FrameChunk {
    #[inline]
    fn bad_chunk(error_message: String) -> Self {
        Self::Error {
            message: error_message,
        }
    }

    #[inline]
    fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            Self::Error { message } => Some(message),
            _ => None,
        }
    }
}

impl Default for FrameChunk {
    fn default() -> Self {
        Self::Error {
            message: "Default frame chunk".to_string(),
        }
    }
}
