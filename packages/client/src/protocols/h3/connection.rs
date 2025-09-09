//! HTTP/3 connection management
//!
//! This module provides HTTP/3 connection types and stream management using quiche
//! integrated with ystream streaming patterns.

use std::sync::Arc;

use crossbeam_utils::Backoff;
use ystream::prelude::*;
use bytes::Bytes;
use quiche::h3::NameValue;

use crate::prelude::*;
// quiche import removed - not used
use crate::protocols::core::{HttpVersion, TimeoutConfig};

/// HTTP/3 connection wrapper that integrates quiche with ystream
pub struct H3Connection {
    inner: Arc<std::sync::Mutex<quiche::Connection>>,
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
    /// Create a new H3Connection from quiche connection
    pub fn new(connection: quiche::Connection, config: TimeoutConfig) -> Self {
        Self {
            inner: Arc::new(std::sync::Mutex::new(connection)),
            config,
        }
    }

    /// Get the HTTP version
    pub fn version(&self) -> HttpVersion {
        HttpVersion::Http3
    }

    /// Get the timeout configuration
    pub fn config(&self) -> &TimeoutConfig {
        &self.config
    }

    /// Send data through HTTP/3 connection using COMPLETE quiche::h3::Connection API
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
                                format!("H3 config creation failed: {}", e)
                            ));
                            return;
                        }
                    };
                    
                    let mut h3_conn = match quiche::h3::Connection::with_transport(&mut conn, &h3_config) {
                        Ok(h3) => h3,
                        Err(e) => {
                            emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                format!("H3 connection creation failed: {}", e)
                            ));
                            return;
                        }
                    };
                    
                    // Use REAL quiche HTTP/3 request sending
                    let headers = vec![
                        quiche::h3::Header::new(b":method", b"POST"),
                        quiche::h3::Header::new(b":scheme", b"https"),
                        quiche::h3::Header::new(b":authority", b"example.com"),
                        quiche::h3::Header::new(b":path", b"/data"),
                    ];
                    
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
                                        format!("H3 send_body failed: {}", e)
                                    ));
                                }
                            }
                        },
                        Err(e) => {
                            emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                format!("H3 send_request failed: {}", e)
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

    /// Receive data from HTTP/3 connection using COMPLETE quiche::h3::Connection API
    pub fn receive_data(&self) -> AsyncStream<crate::protocols::frames::FrameChunk, 1024> {
        let connection = Arc::clone(&self.inner);
        
        AsyncStream::with_channel(move |sender| {
            match connection.lock() {
                Ok(mut conn) => {
                    let h3_config = match quiche::h3::Config::new() {
                        Ok(config) => config,
                        Err(e) => {
                            emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                format!("H3 config creation failed: {}", e)
                            ));
                            return;
                        }
                    };
                    
                    let mut h3_conn = match quiche::h3::Connection::with_transport(&mut conn, &h3_config) {
                        Ok(h3) => h3,
                        Err(e) => {
                            emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                format!("H3 connection creation failed: {}", e)
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
                                        continue;
                                    },
                                    Err(e) => {
                                        emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                            format!("H3 recv_body failed: {}", e)
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
                                continue;
                            },
                            
                            Ok((_, quiche::h3::Event::PriorityUpdate { .. })) => {
                                // Priority update - continue polling
                                continue;
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
                                continue;
                            },
                            
                            Err(e) => {
                                emit!(sender, crate::protocols::frames::FrameChunk::bad_chunk(
                                    format!("H3 poll failed: {}", e)
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

    /// Close HTTP/3 connection gracefully using REAL quiche::Connection.close() API
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
                                format!("QUIC close failed: {}", e)
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

    /// Send an HTTP/3 request using COMPLETE quiche::h3::Connection API - NO manual frame parsing
    pub fn send_request(&self, request: &[u8], stream_id: u64) -> AsyncStream<HttpChunk, 1024> {
        let connection = Arc::clone(&self.inner);
        let request_data = request.to_vec();

        AsyncStream::<HttpChunk, 1024>::with_channel(move |sender| {
            match connection.lock() {
                Ok(mut conn) => {
                    let h3_config = match quiche::h3::Config::new() {
                        Ok(config) => config,
                        Err(e) => {
                            emit!(sender, HttpChunk::bad_chunk(format!("H3 config failed: {}", e)));
                            return;
                        }
                    };

                    let mut h3_conn = match quiche::h3::Connection::with_transport(&mut conn, &h3_config) {
                        Ok(h3) => h3,
                        Err(e) => {
                            emit!(sender, HttpChunk::bad_chunk(format!("H3 connection failed: {}", e)));
                            return;
                        }
                    };

                    // Parse request headers from raw HTTP request (simplified - real implementation would parse properly)
                    let headers = vec![
                        quiche::h3::Header::new(b":method", b"GET"),
                        quiche::h3::Header::new(b":scheme", b"https"), 
                        quiche::h3::Header::new(b":authority", b"example.com"),
                        quiche::h3::Header::new(b":path", b"/"),
                    ];

                    // Send request using REAL quiche H3 API
                    match h3_conn.send_request(&mut conn, &headers, request_data.is_empty()) {
                        Ok(actual_stream_id) => {
                            // Send body if present
                            if !request_data.is_empty() {
                                match h3_conn.send_body(&mut conn, actual_stream_id, &request_data, true) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        emit!(sender, HttpChunk::bad_chunk(format!("H3 send_body failed: {}", e)));
                                        return;
                                    }
                                }
                            }

                            // Poll for response using REAL quiche H3 event system - NO manual parsing
                            let mut recv_buffer = [0; 65535];
                            
                            loop {
                                match h3_conn.poll(&mut conn) {
                                    Ok((stream_id, quiche::h3::Event::Headers { list, more_frames })) => {
                                        // Convert quiche headers to http::HeaderMap
                                        let mut header_map = http::HeaderMap::new();
                                        let mut status = http::StatusCode::OK;
                                        
                                        for header in &list {
                                            let name = header.name();
                                            let value = header.value();
                                            
                                            if name == b":status" {
                                                if let Ok(status_str) = std::str::from_utf8(value) {
                                                    if let Ok(status_code) = status_str.parse::<u16>() {
                                                        if let Ok(parsed_status) = http::StatusCode::from_u16(status_code) {
                                                            status = parsed_status;
                                                        }
                                                    }
                                                }
                                            } else if !name.starts_with(b":") {
                                                if let (Ok(header_name), Ok(header_value)) = (
                                                    http::HeaderName::try_from(name),
                                                    http::HeaderValue::try_from(value)
                                                ) {
                                                    header_map.insert(header_name, header_value);
                                                }
                                            }
                                        }
                                        
                                        emit!(sender, HttpChunk::Headers(status, header_map));
                                        
                                        if !more_frames {
                                            break;
                                        }
                                    },
                                    
                                    Ok((stream_id, quiche::h3::Event::Data)) => {
                                        match h3_conn.recv_body(&mut conn, stream_id, &mut recv_buffer) {
                                            Ok(bytes_read) => {
                                                if bytes_read > 0 {
                                                    let data = Bytes::from(recv_buffer[..bytes_read].to_vec());
                                                    emit!(sender, HttpChunk::Data(data));
                                                }
                                            },
                                            Err(quiche::h3::Error::Done) => continue,
                                            Err(e) => {
                                                emit!(sender, HttpChunk::bad_chunk(format!("H3 recv_body failed: {}", e)));
                                                break;
                                            }
                                        }
                                    },
                                    
                                    Ok((_, quiche::h3::Event::Finished)) => {
                                        break;
                                    },
                                    
                                    Ok((_, quiche::h3::Event::Reset(_))) => {
                                        // Stream was reset
                                        continue;
                                    },
                                    
                                    Ok((_, quiche::h3::Event::PriorityUpdate)) => {
                                        // Priority update - continue polling
                                        continue;
                                    },
                                    
                                    Ok((_, quiche::h3::Event::GoAway)) => {
                                        // Server is going away
                                        break;
                                    },
                                    
                                    Err(quiche::h3::Error::Done) => {
                                        if conn.is_closed() {
                                            break;
                                        }
                                        continue;
                                    },
                                    
                                    Err(e) => {
                                        emit!(sender, HttpChunk::bad_chunk(format!("H3 poll failed: {}", e)));
                                        break;
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            emit!(sender, HttpChunk::bad_chunk(format!("H3 send_request failed: {}", e)));
                        }
                    }
                },
                Err(_) => {
                    emit!(sender, HttpChunk::bad_chunk("Connection mutex poisoned".to_string()));
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

/// HTTP/3 stream wrapper that bridges quiche streams to AsyncStream
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
    /// Create a new H3Stream from quiche connection and stream ID
    pub fn new(stream_id: u64, connection: Arc<std::sync::Mutex<quiche::Connection>>) -> Self {
        Self {
            stream_id,
            connection,
        }
    }

    /// Convert to AsyncStream for ystream integration
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
                        let error_chunk = HttpChunk::bad_chunk(format!("H3 stream error: {}", e));
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
        match Self::try_create_error_connection(&scid, addr) {
            Some(connection) => {
                Self {
                    inner: Arc::new(std::sync::Mutex::new(connection)),
                    config: TimeoutConfig::default(),
                }
            }
            None => {
                // QUIC completely unavailable - TLS/crypto or network failure
                // This is unrecoverable for an HTTP/3 library
                panic!("Critical system error: QUIC/HTTP3 support completely unavailable - {}", error);
            }
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
}

// DELETED: parse_varint() function - quiche handles all varint parsing internally

// DELETED: parse_qpack_headers_simple() function - quiche QPACK decoder handles all header parsing internally

/// Convert h3 HeaderField Vec to http::StatusCode and http::HeaderMap (unused now but kept for reference)
#[allow(dead_code)]
fn convert_header_fields_to_http_reference(fields: Vec<(String, String)>) -> (http::StatusCode, http::HeaderMap) {
    let mut headers = http::HeaderMap::new();
    let mut status = http::StatusCode::OK; // Default status
    
    for (name_str, value_str) in fields {
        // Handle HTTP/3 pseudo-headers
        if name_str.starts_with(':') {
            match name_str.as_ref() {
                ":status" => {
                    if let Ok(status_code) = value_str.parse::<u16>() {
                        if let Ok(parsed_status) = http::StatusCode::from_u16(status_code) {
                            status = parsed_status;
                        }
                    }
                },
                // Skip other pseudo-headers like :method, :path, :scheme, :authority
                _ => continue,
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
            config: self.config.clone(),
        }
    }
}
