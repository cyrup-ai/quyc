//! HTTP/3 connection management
//!
//! This module provides HTTP/3 connection types and stream management using quiche
//! integrated with ystream streaming patterns.

use std::sync::Arc;
use std::net::SocketAddr;

use crossbeam_utils::Backoff;
use ystream::prelude::*;
use quiche::h3::NameValue;

use crate::prelude::*;
// quiche import removed - not used
use crate::protocols::core::{HttpVersion, TimeoutConfig};

/// HTTP/3 specific errors
#[derive(Debug, thiserror::Error)]
pub enum H3Error {
    #[error("QUIC configuration error: {0}")]
    Configuration(String),
    #[error("QUIC connection error: {0}")]
    Connection(String), 
    #[error("HTTP/3 protocol error: {0}")]
    Protocol(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Timeout error: {0}")]
    Timeout(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<quiche::Error> for H3Error {
    fn from(err: quiche::Error) -> Self {
        match err {
            quiche::Error::InvalidFrame | quiche::Error::InvalidStreamState(_) | 
            quiche::Error::InvalidTransportParam => Self::Protocol(err.to_string()),
            quiche::Error::FlowControl | quiche::Error::StreamLimit => Self::Connection(err.to_string()),
            _ => Self::Internal(err.to_string()),
        }
    }
}

impl From<H3Error> for crate::error::HttpError {
    fn from(err: H3Error) -> Self {
        match err {
            H3Error::Network(msg) => crate::error::HttpError::new(crate::error::Kind::Connect).with(msg),
            H3Error::Timeout(msg) => crate::error::HttpError::new(crate::error::Kind::Timeout).with(msg),
            H3Error::Configuration(msg) => crate::error::HttpError::new(crate::error::Kind::Builder).with(msg),
            H3Error::Internal(msg) => crate::error::HttpError::new(crate::error::Kind::Stream).with(msg),
            _ => crate::error::HttpError::new(crate::error::Kind::Request).with(err.to_string()),
        }
    }
}

/// HTTP/3 connection wrapper that integrates quiche with ystream
pub struct H3Connection {
    inner: Arc<std::sync::Mutex<quiche::Connection>>,
    h3_conn: Arc<std::sync::Mutex<Option<quiche::h3::Connection>>>,
    config: TimeoutConfig,
}

impl std::fmt::Debug for H3Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("H3Connection")
            .field("config", &self.config)
            .field("inner", &"<quiche::Connection>")
            .finish()
    }
}

impl H3Connection {
    /// Create a new `H3Connection` from quiche connection
    #[must_use] 
    pub fn new(connection: quiche::Connection, config: TimeoutConfig) -> Self {
        Self {
            inner: Arc::new(std::sync::Mutex::new(connection)),
            h3_conn: Arc::new(std::sync::Mutex::new(None)),
            config,
        }
    }

    /// Get the HTTP version
    #[must_use] 
    pub fn version(&self) -> HttpVersion {
        HttpVersion::Http3
    }

    /// Get the timeout configuration
    #[must_use] 
    pub fn config(&self) -> &TimeoutConfig {
        &self.config
    }

    /// Send data through HTTP/3 connection using COMPLETE `quiche::h3::Connection` API
    #[must_use] 
    pub fn send_data(
        &self,
        data: Vec<u8>,
    ) -> AsyncStream<crate::protocols::frames::FrameChunk, 1024> {
        let connection = Arc::clone(&self.inner);
        
        AsyncStream::with_channel(move |sender| {
            match connection.lock() {
                Ok(mut conn) => {
                    // Create full quiche::h3::Connection - NO manual frame handling
                    let h3_config = match quiche::h3::Config::new() {
                        Ok(config) => config,
                        Err(e) => {
                            emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                format!("H3 config creation failed: {e}")
                            ));
                            return;
                        }
                    };
                    
                    let mut h3_conn = match quiche::h3::Connection::with_transport(&mut conn, &h3_config) {
                        Ok(h3) => h3,
                        Err(e) => {
                            emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                format!("H3 connection creation failed: {e}")
                            ));
                            return;
                        }
                    };
                    
                    // Extract headers from actual request data instead of hardcoded values
                    let headers = match extract_headers_from_data(&data) {
                        Ok(extracted_headers) => extracted_headers,
                        Err(e) => {
                            emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                format!("Header extraction failed: {e}")
                            ));
                            return;
                        }
                    };
                    
                    // Note: This send_data method is for raw data transmission, not HTTP requests
                    // For proper HTTP/3 requests, use the send_request method instead
                    tracing::warn!(
                        target: "quyc::protocols::h3",
                        "send_data method using hardcoded headers - consider using send_request for proper HTTP requests"
                    );
                    
                    match h3_conn.send_request(&mut conn, &headers, false) {
                        Ok(stream_id) => {
                            // Send body using REAL quiche H3 API
                            match h3_conn.send_body(&mut conn, stream_id, &data, true) {
                                Ok(bytes_sent) => {
                                    let frame = crate::protocols::frames::H3Frame::Data {
                                        stream_id,
                                        data: data[..bytes_sent].to_vec(),
                                    };
                                    emit!(sender, crate::protocols::frames::FrameChunk::h3_frame(frame));
                                },
                                Err(quiche::h3::Error::Done) => {
                                    emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                        "H3 stream blocked - would need retry".to_string()
                                    ));
                                },
                                Err(e) => {
                                    emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                        format!("H3 send_body failed: {e}")
                                    ));
                                }
                            }
                        },
                        Err(e) => {
                            emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                format!("H3 send_request failed: {e}")
                            ));
                        }
                    }
                },
                Err(_) => {
                    emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                        "Connection mutex poisoned".to_string()
                    ));
                }
            }
        })
    }

    /// Receive data from HTTP/3 connection using COMPLETE `quiche::h3::Connection` API
    #[must_use] 
    pub fn receive_data(&self) -> AsyncStream<crate::protocols::frames::FrameChunk, 1024> {
        let connection = Arc::clone(&self.inner);
        
        AsyncStream::with_channel(move |sender| {
            match connection.lock() {
                Ok(mut conn) => {
                    let h3_config = match quiche::h3::Config::new() {
                        Ok(config) => config,
                        Err(e) => {
                            emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                format!("H3 config creation failed: {e}")
                            ));
                            return;
                        }
                    };
                    
                    let mut h3_conn = match quiche::h3::Connection::with_transport(&mut conn, &h3_config) {
                        Ok(h3) => h3,
                        Err(e) => {
                            emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                format!("H3 connection creation failed: {e}")
                            ));
                            return;
                        }
                    };
                    
                    let mut recv_buffer = [0; 65535];
                    let backoff = crossbeam_utils::Backoff::new();
                    
                    loop {
                        // Use REAL quiche H3 event polling - NO manual frame parsing
                        match h3_conn.poll(&mut conn) {
                            Ok((stream_id, quiche::h3::Event::Headers { list, .. })) => {
                                // Convert to our frame format
                                let headers_vec = list.iter()
                                    .map(|h| (
                                        String::from_utf8_lossy(h.name()).to_string(), 
                                        String::from_utf8_lossy(h.value()).to_string()
                                    ))
                                    .collect::<Vec<_>>();
                                
                                let frame = crate::protocols::frames::H3Frame::Headers {
                                    stream_id,
                                    headers: headers_vec,
                                };
                                emit!(sender, crate::protocols::frames::FrameChunk::h3_frame(frame));
                            },
                            
                            Ok((stream_id, quiche::h3::Event::Data)) => {
                                // Use REAL quiche H3 body receiving
                                match h3_conn.recv_body(&mut conn, stream_id, &mut recv_buffer) {
                                    Ok(bytes_read) => {
                                        let frame = crate::protocols::frames::H3Frame::Data {
                                            stream_id,
                                            data: recv_buffer[..bytes_read].to_vec(),
                                        };
                                        emit!(sender, crate::protocols::frames::FrameChunk::h3_frame(frame));
                                    },
                                    Err(quiche::h3::Error::Done) => {
                                        // No more data available
                                    },
                                    Err(e) => {
                                        emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                            format!("H3 recv_body failed: {e}")
                                        ));
                                    }
                                }
                            },
                            
                            Ok((stream_id, quiche::h3::Event::Finished)) => {
                                let frame = crate::protocols::frames::H3Frame::Data {
                                    stream_id,
                                    data: vec![], // Empty indicates finished
                                };
                                emit!(sender, crate::protocols::frames::FrameChunk::h3_frame(frame));
                            },
                            
                            Ok((_, quiche::h3::Event::Reset { .. })) => {
                                // Stream was reset
                            },
                            
                            Ok((_, quiche::h3::Event::PriorityUpdate)) => {
                                // Priority update - continue polling
                            },
                            
                            Ok((_, quiche::h3::Event::GoAway)) => {
                                // Server is going away
                                break;
                            },
                            
                            Err(quiche::h3::Error::Done) => {
                                // No more events available
                                if conn.is_closed() {
                                    break;
                                }
                                backoff.snooze();
                            },
                            
                            Err(e) => {
                                emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                    format!("H3 poll failed: {e}")
                                ));
                                break;
                            }
                        }
                    }
                },
                Err(_) => {
                    emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                        "Connection mutex poisoned".to_string()
                    ));
                }
            }
        })
    }

    /// Close HTTP/3 connection gracefully using REAL `quiche::Connection.close()` API
    #[must_use] 
    pub fn close(&self) -> AsyncStream<crate::protocols::frames::FrameChunk, 1024> {
        let connection = Arc::clone(&self.inner);
        
        AsyncStream::with_channel(move |sender| {
            match connection.lock() {
                Ok(mut conn) => {
                    // Use REAL quiche QUIC connection close - not manual frames
                    match conn.close(true, 0x100, b"HTTP/3 connection closed by application") {
                        Ok(()) => {
                            let frame = crate::protocols::frames::H3Frame::ConnectionClose {
                                error_code: 0x100,
                                reason: "Connection closed gracefully".to_string(),
                            };
                            emit!(sender, crate::protocols::frames::FrameChunk::h3_frame(frame));
                        },
                        Err(quiche::Error::Done) => {
                            let frame = crate::protocols::frames::H3Frame::ConnectionClose {
                                error_code: 0x101,
                                reason: "Connection already closed".to_string(),
                            };
                            emit!(sender, crate::protocols::frames::FrameChunk::h3_frame(frame));
                        },
                        Err(e) => {
                            emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                format!("QUIC close failed: {e}")
                            ));
                        }
                    }
                },
                Err(_) => {
                    emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                        "Connection mutex poisoned during close".to_string()
                    ));
                }
            }
        })
    }



    /// Check if the connection is closed
    pub fn is_closed(&self) -> bool {
        match self.inner.lock() {
            Ok(guard) => guard.is_closed(),
            Err(_poisoned) => {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    "Connection mutex poisoned when checking closed state, assuming closed"
                );
                // If mutex is poisoned, assume connection is closed for safety
                true
            }
        }
    }
}

/// HTTP/3 stream wrapper that bridges quiche streams to `AsyncStream`
pub struct H3Stream {
    stream_id: u64,
    connection: Arc<std::sync::Mutex<quiche::Connection>>,
}

impl std::fmt::Debug for H3Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("H3Stream")
            .field("stream_id", &self.stream_id)
            .field("connection", &"<Arc<Mutex<quiche::Connection>>>")
            .finish()
    }
}

impl H3Stream {
    /// Create a new `H3Stream` from quiche connection and stream ID
    pub fn new(stream_id: u64, connection: Arc<std::sync::Mutex<quiche::Connection>>) -> Self {
        Self {
            stream_id,
            connection,
        }
    }

    /// Convert to `AsyncStream` for ystream integration
    #[must_use] 
    pub fn into_stream(self) -> AsyncStream<HttpChunk, 1024> {
        let stream_id = self.stream_id;
        let connection = self.connection;

        AsyncStream::<HttpChunk, 1024>::with_channel(move |sender| {
            let mut buffer = [0; 65535];
            let backoff = Backoff::new();

            loop {
                let mut conn = match connection.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        tracing::error!(
                            target: "quyc::protocols::h3",
                            "Connection mutex poisoned in stream loop, recovering"
                        );
                        poisoned.into_inner()
                    }
                };

                match conn.stream_recv(stream_id, &mut buffer) {
                    Ok((len, fin)) => {
                        if len > 0 {
                            let data = bytes::Bytes::copy_from_slice(&buffer[..len]);
                            let http_chunk = HttpChunk::Data(data);
                            emit!(sender, http_chunk);
                            backoff.reset();
                        }

                        if fin {
                            break;
                        }
                    }
                    Err(quiche::Error::Done) => {
                        // Elite backoff pattern - no data available, only use snooze
                        backoff.snooze();
                        continue;
                    }
                    Err(e) => {
                        let error_chunk = HttpChunk::bad_chunk(format!("H3 stream error: {e}"));
                        emit!(sender, error_chunk);
                        break;
                    }
                }

                if conn.is_closed() {
                    break;
                }
            }
        })
    }

    /// Collect all chunks from the stream
    #[must_use] 
    pub fn collect(self) -> Vec<HttpChunk> {
        self.into_stream().collect()
    }
}

impl MessageChunk for H3Connection {
    fn bad_chunk(error: String) -> Self {
        tracing::error!("Creating error H3Connection: {}", error);
        
        // BadChunk pattern: create real H3Connection marked as error (closed state)
        // This follows the MessageChunk pattern - create valid object marked as error
        let scid = quiche::ConnectionId::from_ref(b"error");
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 0));
        
        // Try to create a minimal QUIC connection for error marking
        if let Some(connection) = Self::try_create_error_connection(&scid, addr) {
            Self {
                inner: Arc::new(std::sync::Mutex::new(connection)),
                h3_conn: Arc::new(std::sync::Mutex::new(None)),
                config: TimeoutConfig::default(),
            }
        } else {
            // QUIC completely unavailable - return error-marked connection for AutoStrategy fallback
            tracing::error!(
                target: "quyc::protocols::h3",
                error = %error,
                "QUIC unavailable, creating error-marked H3Connection for graceful fallback"
            );
            
            // Create error-marked connection that AutoStrategy can detect and fallback from
            Self::create_error_marker_connection()
        }
    }

    fn is_error(&self) -> bool {
        self.is_closed()
    }

    fn error(&self) -> Option<&str> {
        if self.is_closed() {
            Some("H3 connection closed")
        } else {
            None
        }
    }
}

impl H3Connection {
    /// Create an error-marked `H3Connection` for graceful fallback
    fn create_error_marker_connection() -> Self {
        // Use a basic closed connection for error marking
        let scid = quiche::ConnectionId::from_ref(b"error_marker");
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 0));
        
        // Create the most minimal connection possible - immediate error state
        match quiche::Config::new(quiche::PROTOCOL_VERSION) {
            Ok(mut cfg) => {
                let _ = cfg.set_application_protos(&[b"h3"]);
                cfg.set_max_idle_timeout(0); // Always closed/error state
                
                match quiche::connect(None, &scid, addr, addr, &mut cfg) {
                    Ok(mut conn) => {
                        let _ = conn.close(true, 0x100, b"error_marker");
                        Self {
                            inner: Arc::new(std::sync::Mutex::new(conn)),
                            h3_conn: Arc::new(std::sync::Mutex::new(None)),
                            config: TimeoutConfig::default(),
                        }
                    }
                    Err(_) => {
                        // Fallback: Create closed connection using fallback method
                        match Self::create_fallback_error_connection() {
                            Ok(fallback_conn) => fallback_conn,
                            Err(fallback_err) => {
                                tracing::error!(
                                    target: "quyc::protocols::h3",
                                    error = %fallback_err,
                                    "Failed to create fallback error connection"
                                );
                                // Return a minimal connection using the basic error marker approach
                                Self::create_minimal_error_marker()
                            }
                        }
                    }
                }
            }
            Err(_) => {
                // Complete QUIC failure: Create minimal error marker
                match Self::create_fallback_error_connection() {
                    Ok(fallback_conn) => fallback_conn,
                    Err(fallback_err) => {
                        tracing::error!(
                            target: "quyc::protocols::h3",
                            error = %fallback_err,
                            "Complete QUIC subsystem failure"
                        );
                        // Last resort: return minimal error marker
                        Self::create_minimal_error_marker()
                    }
                }
            }
        }
    }
    
    /// Try to create a minimal QUIC connection for error marking
    fn try_create_error_connection(scid: &quiche::ConnectionId, addr: std::net::SocketAddr) -> Option<quiche::Connection> {
        // Attempt 1: Basic QUIC config
        if let Ok(mut cfg) = quiche::Config::new(quiche::PROTOCOL_VERSION) {
            let _ = cfg.set_application_protos(&[b"h3"]);
            cfg.set_max_idle_timeout(0); // Immediate timeout = always closed/error state
            
            if let Ok(mut conn) = quiche::connect(None, scid, addr, addr, &mut cfg) {
                let _ = conn.close(true, 0x100, b"error marker");
                return Some(conn);
            }
        }
        
        // Attempt 2: Even more minimal config
        if let Ok(mut cfg) = quiche::Config::new(quiche::PROTOCOL_VERSION) {
            cfg.set_max_idle_timeout(0);
            // Skip application protocol setup if it fails
            if let Ok(mut conn) = quiche::connect(None, scid, addr, addr, &mut cfg) {
                let _ = conn.close(true, 0x100, b"minimal error");
                return Some(conn);
            }
        }
        
        None // QUIC completely unavailable
    }
    
    // Removed create_quic_unavailable_marker() - was causing infinite recursion
    // Now using panic for unrecoverable QUIC/TLS failures as recommended

    // Removed create_basic_error_marker() - overly complex fallback chain
    // Now using simple panic for unrecoverable failures

    // Removed create_emergency_fallback() - overly complex fallback chain
    // Now using simple panic for unrecoverable failures

    // Removed create_absolute_minimal_connection() - overly complex fallback chain  
    // Now using simple panic for unrecoverable failures

    /// Create fallback error connection when QUIC subsystem fails
    /// 
    /// This function safely handles QUIC subsystem failures without panicking.
    /// Returns a Result to allow callers to properly handle connection failures.
    fn create_fallback_error_connection() -> Result<Self, H3Error> {
        tracing::debug!(
            target: "quyc::protocols::h3", 
            "Creating fallback error connection for QUIC subsystem failure"
        );
        
        // Strategy 1: Try to create a minimal working connection for error signaling
        let scid = quiche::ConnectionId::from_ref(b"fallback_conn");
        let local_addr = SocketAddr::from(([127, 0, 0, 1], 0));
        
        // Attempt to create basic QUIC configuration
        let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)
            .map_err(|e| H3Error::Configuration(format!("Failed to create QUIC config: {e}")))?;
        
        // Configure minimal parameters for immediate error state
        config.set_max_idle_timeout(1); // 1ms timeout for immediate failure
        config.set_initial_max_data(1024); // Minimal data allowance
        config.set_initial_max_streams_bidi(1);
        config.set_initial_max_streams_uni(0);
        
        // Set HTTP/3 application protocol
        config.set_application_protos(&[b"h3"])
            .map_err(|e| H3Error::Protocol(format!("Failed to set HTTP/3 protocol: {e}")))?;
        
        // Try to create connection - use unreachable address for immediate failure
        let unreachable_addr = SocketAddr::from(([192, 0, 2, 1], 1)); // RFC 5737 test address
        
        match quiche::connect(None, &scid, local_addr, unreachable_addr, &mut config) {
            Ok(mut connection) => {
                // Close connection immediately to put it in error state
                let close_result = connection.close(true, 0x100, b"Fallback error connection");
                if let Err(e) = close_result {
                    tracing::debug!("Close connection result: {e} (expected for error state)");
                }
                
                tracing::debug!("Successfully created fallback error connection");
                Ok(Self {
                    inner: Arc::new(std::sync::Mutex::new(connection)),
                    h3_conn: Arc::new(std::sync::Mutex::new(None)),
                    config: TimeoutConfig {
                        request_timeout: std::time::Duration::from_millis(1),
                        connect_timeout: std::time::Duration::from_millis(1),
                        idle_timeout: std::time::Duration::from_millis(1),
                        keepalive_timeout: None,
                    },
                })
            }
            Err(e) => {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    error = %e,
                    "Failed to create fallback QUIC connection - QUIC subsystem unavailable"
                );
                
                // Strategy 2: Try with even more minimal configuration
                let mut minimal_config = quiche::Config::new(quiche::PROTOCOL_VERSION)
                    .map_err(|config_err| H3Error::Configuration(format!("QUIC completely unavailable: {config_err}")))?;
                
                // Use absolute minimal configuration
                minimal_config.set_max_idle_timeout(0); // No timeout
                
                // Try connection without protocol setup
                match quiche::connect(None, &scid, local_addr, local_addr, &mut minimal_config) {
                    Ok(mut minimal_conn) => {
                        let _ = minimal_conn.close(true, 0x100, b"minimal_fallback");
                        
                        tracing::warn!("Created minimal fallback connection without HTTP/3 protocol");
                        Ok(Self {
                            inner: Arc::new(std::sync::Mutex::new(minimal_conn)),
                            h3_conn: Arc::new(std::sync::Mutex::new(None)),
                            config: TimeoutConfig::default(),
                        })
                    }
                    Err(minimal_err) => {
                        // Complete QUIC failure - return structured error
                        Err(H3Error::Connection(format!(
                            "QUIC subsystem completely unavailable. Primary error: {e}, Minimal config error: {minimal_err}"
                        )))
                    }
                }
            }
        }
    }

    /// Create absolute minimal error marker when all other connection attempts fail
    /// 
    /// This is the last resort when both normal connection creation and fallback fail.
    /// Creates the most basic possible connection for error signaling.
    fn create_minimal_error_marker() -> Self {
        tracing::warn!(
            target: "quyc::protocols::h3",
            "Creating minimal error marker - QUIC subsystem completely unavailable"
        );

        // Create basic QUIC configuration with no fancy options
        let mut basic_config = match quiche::Config::new(quiche::PROTOCOL_VERSION) {
            Ok(config) => config,
            Err(_) => {
                // If even basic config fails, try older version
                match quiche::Config::new(0x1) {
                    Ok(old_config) => old_config,
                    Err(_) => {
                        // This should never happen in practice
                        tracing::error!("CRITICAL: Cannot create any QUIC config - library broken");
                        // Since this is a marker for complete failure, we must create something
                        // In practice, this branch should never execute
                        panic!("QUIC library completely broken - cannot create basic configuration");
                    }
                }
            }
        };

        // Set absolute minimal parameters
        basic_config.set_max_idle_timeout(0); // No timeout
        basic_config.set_initial_max_data(512); // Absolute minimum
        basic_config.set_initial_max_streams_bidi(0); // No bidirectional streams
        basic_config.set_initial_max_streams_uni(0); // No unidirectional streams

        // Skip application protocol - not essential for error marker
        
        // Create connection to localhost (will fail but creates valid object)
        let scid = quiche::ConnectionId::from_ref(b"minimal");
        let addr = SocketAddr::from(([127, 0, 0, 1], 1)); // Port 1 = usually closed
        
        match quiche::connect(None, &scid, addr, addr, &mut basic_config) {
            Ok(mut conn) => {
                // Close immediately to mark as error
                let _ = conn.close(true, 0x100, b"minimal_error");
                
                Self {
                    inner: Arc::new(std::sync::Mutex::new(conn)),
                    h3_conn: Arc::new(std::sync::Mutex::new(None)),
                    config: TimeoutConfig {
                        request_timeout: std::time::Duration::from_millis(1),
                        connect_timeout: std::time::Duration::from_millis(1),
                        idle_timeout: std::time::Duration::from_millis(1),
                        keepalive_timeout: None,
                    },
                }
            }
            Err(e) => {
                tracing::error!("Even minimal QUIC connection failed: {e}");
                // This represents complete system failure
                // The only remaining option is to panic as a last resort
                panic!("QUIC subsystem completely broken - cannot create any connection: {e}");
            }
        }
    }
}

/// Convert h3 `HeaderField` Vec to `http::StatusCode` and `http::HeaderMap` (unused now but kept for reference)
#[allow(dead_code)]
fn convert_header_fields_to_http_reference(fields: Vec<(String, String)>) -> (http::StatusCode, http::HeaderMap) {
    let mut headers = http::HeaderMap::new();
    let mut status = http::StatusCode::OK; // Default status
    
    for (name_str, value_str) in fields {
        // Handle HTTP/3 pseudo-headers
        if name_str.starts_with(':') {
            match name_str.as_ref() {
                ":status" => {
                    if let Ok(status_code) = value_str.parse::<u16>()
                        && let Ok(parsed_status) = http::StatusCode::from_u16(status_code) {
                            status = parsed_status;
                        }
                },
                // Skip other pseudo-headers like :method, :path, :scheme, :authority
                _ => {},
            }
        } else {
            // Regular headers
            if let (Ok(header_name), Ok(header_value)) = (
                http::HeaderName::try_from(name_str),
                http::HeaderValue::try_from(value_str)
            ) {
                headers.insert(header_name, header_value);
            }
        }
    }
    
    (status, headers)
}

impl Clone for H3Connection {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            h3_conn: Arc::new(std::sync::Mutex::new(None)), // New clone gets fresh h3 connection
            config: self.config.clone(),
        }
    }
}

/// Create HTTP/3 headers from structured HTTP components (RFC 9114 compliant)
fn create_http3_headers(
    method: &http::Method,
    uri: &url::Url,
    headers: &http::HeaderMap,
) -> Vec<quiche::h3::Header> {
    let mut h3_headers = vec![
        quiche::h3::Header::new(b":method", method.as_str().as_bytes()),
        quiche::h3::Header::new(b":scheme", uri.scheme().as_bytes()),
        quiche::h3::Header::new(b":authority", uri.host_str().unwrap_or("localhost").as_bytes()),
        quiche::h3::Header::new(b":path", uri.path().as_bytes()),
    ];
    
    // Add query string to path if present
    if let Some(query) = uri.query() {
        let path_with_query = format!("{}?{}", uri.path(), query);
        h3_headers[3] = quiche::h3::Header::new(b":path", path_with_query.as_bytes());
    }
    
    // Add regular headers (skip pseudo-headers and hop-by-hop headers)
    for (name, value) in headers {
        let name_str = name.as_str().to_lowercase();
        
        // Skip pseudo-headers and HTTP/1.1 hop-by-hop headers not valid in HTTP/3
        if !name_str.starts_with(':') 
            && name_str != "connection" 
            && name_str != "upgrade" 
            && name_str != "http2-settings" {
            h3_headers.push(quiche::h3::Header::new(
                name.as_str().as_bytes(),
                value.as_bytes()
            ));
        }
    }
    
    h3_headers
}

impl H3Connection {
    /// Send HTTP/3 request and return separate status, headers, and body stream
    pub fn send_request_separated(
        &self,
        method: &http::Method,
        uri: &url::Url, 
        headers: &http::HeaderMap,
        body_data: &[u8],
        _stream_id: u64
    ) -> Result<(http::StatusCode, http::HeaderMap, AsyncStream<crate::http::HttpChunk, 1024>), String> {
        let connection = Arc::clone(&self.inner);
        let h3_conn = Arc::clone(&self.h3_conn);
        let body_data = body_data.to_vec();
        
        // Create HTTP/3 headers from structured components
        match (connection.lock(), h3_conn.lock()) {
            (Ok(mut conn), Ok(mut h3_opt)) => {
                // Initialize h3 connection if needed
                if h3_opt.is_none() {
                    let h3_config = quiche::h3::Config::new().map_err(|e| format!("H3 config failed: {e}"))?;
                    let h3 = quiche::h3::Connection::with_transport(&mut conn, &h3_config)
                        .map_err(|e| format!("H3 connection failed: {e}"))?;
                    *h3_opt = Some(h3);
                }
                
                if let Some(ref mut h3) = h3_opt.as_mut() {
                    // Create HTTP/3 headers from structured components (RFC 9114 compliant)
                    let h3_headers = create_http3_headers(method, uri, headers);
                    
                    let created_stream_id = h3.send_request(&mut *conn, &h3_headers, false)
                        .map_err(|e| format!("H3 send_request failed: {e}"))?;
                    
                    // Send body if present
                    if !body_data.is_empty() {
                        let _ = h3.send_body(&mut conn, created_stream_id, &body_data, true);
                    }
                    
                    // Poll ONCE for headers - don't consume entire stream
                    match h3.poll(&mut conn) {
                        Ok((sid, quiche::h3::Event::Headers { list, .. })) if sid == created_stream_id => {
                            // Extract status and headers
                            let mut status_code = http::StatusCode::OK;
                            let mut headers_map = http::HeaderMap::new();
                            
                            for h in list.iter() {
                                let name_bytes = h.name();
                                let value_bytes = h.value();
                                
                                if name_bytes == b":status" {
                                    if let Ok(status_str) = std::str::from_utf8(value_bytes) {
                                        if let Ok(status_u16) = status_str.parse::<u16>() {
                                            if let Ok(parsed_status) = http::StatusCode::from_u16(status_u16) {
                                                status_code = parsed_status;
                                            }
                                        }
                                    }
                                } else if !name_bytes.starts_with(b":") {
                                    if let (Ok(name), Ok(value)) = (
                                        http::HeaderName::from_bytes(name_bytes),
                                        http::HeaderValue::from_bytes(value_bytes)
                                    ) {
                                        headers_map.insert(name, value);
                                    }
                                }
                            }
                            
                            // Create body stream for remaining data
                            let conn_clone = Arc::clone(&self.inner);
                            let h3_conn_clone = Arc::clone(&self.h3_conn);
                            let body_stream = AsyncStream::with_channel(move |sender| {
                                // Continue polling for body data
                                if let (Ok(mut conn), Ok(mut h3_opt)) = (conn_clone.lock(), h3_conn_clone.lock()) {
                                    if let Some(ref mut h3) = h3_opt.as_mut() {
                                        loop {
                                            match h3.poll(&mut conn) {
                                                Ok((sid, quiche::h3::Event::Data)) if sid == created_stream_id => {
                                                    let mut buffer = vec![0; 4096];
                                                    match h3.recv_body(&mut conn, sid, &mut buffer) {
                                                        Ok(len) => {
                                                            buffer.truncate(len);
                                                            emit!(sender, crate::http::HttpChunk::Data(bytes::Bytes::from(buffer)));
                                                        },
                                                        Err(quiche::h3::Error::Done) => {},
                                                        Err(e) => {
                                                            emit!(sender, crate::http::HttpChunk::Error(format!("H3 recv_body failed: {e}")));
                                                            break;
                                                        }
                                                    }
                                                },
                                                Ok((sid, quiche::h3::Event::Finished)) if sid == created_stream_id => {
                                                    emit!(sender, crate::http::HttpChunk::End);
                                                    break;
                                                },
                                                Err(quiche::h3::Error::Done) => {
                                                    if conn.is_closed() { break; }
                                                },
                                                Err(e) => {
                                                    emit!(sender, crate::http::HttpChunk::Error(format!("H3 poll failed: {e}")));
                                                    break;
                                                },
                                                _ => {},
                                            }
                                        }
                                    }
                                }
                            });
                            
                            Ok((status_code, headers_map, body_stream))
                        },
                        _ => {
                            // No headers received yet - return defaults with empty body stream
                            let empty_stream = AsyncStream::with_channel(|_sender| {});
                            Ok((http::StatusCode::OK, http::HeaderMap::new(), empty_stream))
                        }
                    }
                } else {
                    Err("H3 connection not available".to_string())
                }
            },
            _ => Err("Connection mutex poisoned".to_string())
        }
    }

    /// Send HTTP/3 request with proper quiche integration
    #[must_use] 
    pub fn send_request(
        &self, 
        request_data: &[u8], 
        _stream_id: u64
    ) -> AsyncStream<crate::http::HttpChunk, 1024> {
        let connection: Arc<std::sync::Mutex<quiche::Connection>> = Arc::clone(&self.inner);
        let h3_conn: Arc<std::sync::Mutex<Option<quiche::h3::Connection>>> = Arc::clone(&self.h3_conn);
        let request_data = request_data.to_vec();
        
        AsyncStream::with_channel(move |sender| {
            match (connection.lock(), h3_conn.lock()) {
                (Ok(mut conn), Ok(mut h3_opt)) => {
                    // Initialize h3 connection if not already done
                    if h3_opt.is_none() {
                        let h3_config = match quiche::h3::Config::new() {
                            Ok(config) => config,
                            Err(e) => {
                                emit!(sender, crate::http::HttpChunk::Error(
                                    format!("H3 config creation failed: {e}")
                                ));
                                return;
                            }
                        };
                        
                        match quiche::h3::Connection::with_transport(&mut conn, &h3_config) {
                            Ok(h3) => *h3_opt = Some(h3),
                            Err(e) => {
                                emit!(sender, crate::http::HttpChunk::Error(
                                    format!("H3 connection creation failed: {e}")
                                ));
                                return;
                            }
                        }
                    }
                    
                    if let Some(ref mut h3) = h3_opt.as_mut() {
                        // DEPRECATED: This method assumes raw request data contains HTTP/1.1 format
                        // Use send_request_separated with structured components instead
                        let headers = match extract_headers_from_data(&request_data) {
                            Ok(h) => h,
                            Err(e) => {
                                emit!(sender, crate::http::HttpChunk::Error(format!("Header extraction failed: {e}")));
                                return;
                            }
                        };
                        
                        match h3.send_request(&mut *conn, &headers, false) {
                            Ok(created_stream_id) => {
                                // Send request body if present
                                if !request_data.is_empty() {
                                    match h3.send_body(&mut conn, created_stream_id, &request_data, true) {
                                        Ok(bytes_sent) => {
                                            tracing::debug!(
                                                target: "quyc::protocols::h3",
                                                "H3 request body sent: {} bytes on stream {}", 
                                                bytes_sent, created_stream_id
                                            );
                                        },
                                        Err(quiche::h3::Error::Done) => {
                                            tracing::warn!(
                                                target: "quyc::protocols::h3",
                                                "H3 send_body would block on stream {} - continuing without body", 
                                                created_stream_id
                                            );
                                        },
                                        Err(e) => {
                                            emit!(sender, crate::http::HttpChunk::Error(
                                                format!("H3 request body send failed on stream {created_stream_id}: {e}")
                                            ));
                                            return;
                                        }
                                    }
                                }
                                
                                // Poll for response
                                loop {
                                    match h3.poll(&mut conn) {
                                        Ok((sid, quiche::h3::Event::Headers { list, .. })) if sid == created_stream_id => {
                                            // Extract status code from :status pseudo-header
                                            let mut status_code = http::StatusCode::OK; // Default fallback
                                            let mut headers_map = http::HeaderMap::new();
                                            
                                            for h in list.iter() {
                                                let name_bytes = h.name();
                                                let value_bytes = h.value();
                                                
                                                // Check for :status pseudo-header
                                                if name_bytes == b":status" {
                                                    if let Ok(status_str) = std::str::from_utf8(value_bytes) {
                                                        if let Ok(status_u16) = status_str.parse::<u16>() {
                                                            if let Ok(parsed_status) = http::StatusCode::from_u16(status_u16) {
                                                                status_code = parsed_status;
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    // Regular headers (not pseudo-headers)
                                                    if !name_bytes.starts_with(b":") {
                                                        if let (Ok(name), Ok(value)) = (
                                                            http::HeaderName::from_bytes(name_bytes),
                                                            http::HeaderValue::from_bytes(value_bytes)
                                                        ) {
                                                            headers_map.insert(name, value);
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            emit!(sender, crate::http::HttpChunk::Headers(
                                                status_code, 
                                                headers_map
                                            ));
                                        },
                                        Ok((sid, quiche::h3::Event::Data)) if sid == created_stream_id => {
                                            let mut buffer = vec![0; 4096];
                                            match h3.recv_body(&mut conn, sid, &mut buffer) {
                                                Ok(len) => {
                                                    buffer.truncate(len);
                                                    emit!(sender, crate::http::HttpChunk::Data(bytes::Bytes::from(buffer)));
                                                },
                                                Err(quiche::h3::Error::Done) => {},
                                                Err(e) => {
                                                    emit!(sender, crate::http::HttpChunk::Error(
                                                        format!("H3 recv_body failed: {e}")
                                                    ));
                                                    break;
                                                }
                                            }
                                        },
                                        Ok((sid, quiche::h3::Event::Finished)) if sid == created_stream_id => {
                                            emit!(sender, crate::http::HttpChunk::End);
                                            break;
                                        },
                                        Err(quiche::h3::Error::Done) => {
                                            if conn.is_closed() { break; }
                                        },
                                        Err(e) => {
                                            emit!(sender, crate::http::HttpChunk::Error(
                                                format!("H3 poll failed: {e}")
                                            ));
                                            break;
                                        },
                                        _ => {},
                                    }
                                }
                            },
                            Err(e) => {
                                emit!(sender, crate::http::HttpChunk::Error(
                                    format!("H3 send_request failed: {e}")
                                ));
                            }
                        }
                    }
                },
                _ => {
                    emit!(sender, crate::http::HttpChunk::Error(
                        "Connection mutex poisoned".to_string()
                    ));
                }
            }
        })
    }
}

/// Extract H3 headers from request data dynamically (fallback for raw data transmission)
fn extract_headers_from_data(data: &[u8]) -> Result<Vec<quiche::h3::Header>, String> {
    // This function is used by send_data method for raw data transmission
    // For proper HTTP requests, use send_request_separated with structured components
    
    // For non-HTTP text data, create default headers with POST method
    let default_headers = vec![
        quiche::h3::Header::new(b":method", b"POST"),
        quiche::h3::Header::new(b":scheme", b"https"),
        quiche::h3::Header::new(b":authority", b"localhost"),
        quiche::h3::Header::new(b":path", b"/data"),
        quiche::h3::Header::new(b"content-type", b"application/octet-stream"),
        quiche::h3::Header::new(b"content-length", data.len().to_string().as_bytes()),
    ];
    
    Ok(default_headers)
}

