// HashMap import removed - not used
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use ystream::prelude::*;
use http::{HeaderMap, HeaderValue, Method, StatusCode};
use quiche;

/// HTTP/3 chunk for streaming responses
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct Http3Chunk {
    pub stream_id: u64,
    pub data: Vec<u8>,
    pub headers: Option<HeaderMap>,
    pub status: Option<StatusCode>,
    pub finished: bool,
    pub error: Option<String>,
}

impl MessageChunk for Http3Chunk {
    fn bad_chunk(error: String) -> Self {
        Self {
            stream_id: 0,
            data: Vec::new(),
            headers: None,
            status: None,
            finished: true,
            error: Some(error),
        }
    }

    fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    fn is_error(&self) -> bool {
        self.error.is_some()
    }
}


impl Http3Chunk {
    #[must_use] 
    pub fn data(stream_id: u64, data: Vec<u8>) -> Self {
        Self {
            stream_id,
            data,
            headers: None,
            status: None,
            finished: false,
            error: None,
        }
    }

    #[must_use] 
    pub fn headers(stream_id: u64, headers: HeaderMap, status: StatusCode) -> Self {
        Self {
            stream_id,
            data: Vec::new(),
            headers: Some(headers),
            status: Some(status),
            finished: false,
            error: None,
        }
    }

    #[must_use] 
    pub fn finished(stream_id: u64) -> Self {
        Self {
            stream_id,
            data: Vec::new(),
            headers: None,
            status: None,
            finished: true,
            error: None,
        }
    }
}

/// HTTP/3 connection using quiche with `AsyncStream`
pub struct Http3Connection {
    conn: Arc<Mutex<quiche::Connection>>,
    socket: Arc<std::net::UdpSocket>,
    peer_addr: SocketAddr,
}

impl Http3Connection {
    #[must_use] 
    pub fn new(
        conn: quiche::Connection,
        socket: std::net::UdpSocket,
        peer_addr: SocketAddr,
    ) -> Self {
        // Set socket to non-blocking for proper async operation
        if let Err(io_error) = socket.set_nonblocking(true) {
            log::warn!("Failed to set socket non-blocking: {io_error}");
            // Continue with blocking socket - not critical for functionality
        }

        Self {
            conn: Arc::new(Mutex::new(conn)),
            socket: Arc::new(socket),
            peer_addr,
        }
    }

    /// Send HTTP/3 request and return streaming response
    #[must_use] 
    pub fn send_request(
        &self,
        method: &Method,
        path: &str,
        headers: &HeaderMap,
        body: Option<&[u8]>,
    ) -> AsyncStream<Http3Chunk> {
        let method = method.clone();
        let path = path.to_string();
        let headers = headers.clone();
        let body = body.map(<[u8]>::to_vec);
        let conn = Arc::clone(&self.conn);
        let socket = Arc::clone(&self.socket);
        let peer_addr = self.peer_addr;

        AsyncStream::with_channel(move |sender| {
            // Get next available stream ID
            let stream_id = {
                let _conn_guard = if let Ok(guard) = conn.lock() { guard } else {
                    emit!(
                        sender,
                        Http3Chunk::bad_chunk("Failed to lock connection".to_string())
                    );
                    return;
                };

                // Use stream ID 0 for client-initiated bidirectional stream
                0u64
            };

            // Send HTTP/3 headers
            let mut http3_headers = Vec::new();
            http3_headers.push((b":method".to_vec(), method.as_str().as_bytes().to_vec()));
            http3_headers.push((b":path".to_vec(), path.as_bytes().to_vec()));
            http3_headers.push((b":scheme".to_vec(), b"https".to_vec()));
            http3_headers.push((b":authority".to_vec(), b"localhost".to_vec()));

            // Add custom headers
            for (name, value) in &headers {
                http3_headers.push((name.as_str().as_bytes().to_vec(), value.as_bytes().to_vec()));
            }

            // Encode headers (simplified QPACK encoding)
            let encoded_headers = encode_headers(&http3_headers);

            // Send headers frame
            {
                let mut conn_guard = if let Ok(guard) = conn.lock() { guard } else {
                    emit!(
                        sender,
                        Http3Chunk::bad_chunk("Failed to lock connection".to_string())
                    );
                    return;
                };

                if let Err(e) = conn_guard.stream_send(stream_id, &encoded_headers, body.is_none())
                {
                    emit!(
                        sender,
                        Http3Chunk::bad_chunk(format!("Failed to send headers: {e}"))
                    );
                    return;
                }
            }

            // Send body if present
            if let Some(body_data) = body {
                let mut conn_guard = if let Ok(guard) = conn.lock() { guard } else {
                    emit!(
                        sender,
                        Http3Chunk::bad_chunk("Failed to lock connection".to_string())
                    );
                    return;
                };

                if let Err(e) = conn_guard.stream_send(stream_id, &body_data, true) {
                    emit!(
                        sender,
                        Http3Chunk::bad_chunk(format!("Failed to send body: {e}"))
                    );
                    return;
                }
            }

            // Process QUIC packets and read response
            let mut buf = [0; 65535];
            let mut response_buf = [0; 65535];

            loop {
                // Send any pending QUIC packets
                loop {
                    let (write, send_info) = {
                        let mut conn_guard = if let Ok(guard) = conn.lock() { guard } else {
                            emit!(
                                sender,
                                Http3Chunk::bad_chunk("Failed to lock connection".to_string())
                            );
                            return;
                        };

                        match conn_guard.send(&mut buf) {
                            Ok(v) => v,
                            Err(quiche::Error::Done) => break,
                            Err(e) => {
                                emit!(
                                    sender,
                                    Http3Chunk::bad_chunk(format!("QUIC send error: {e}"))
                                );
                                return;
                            }
                        }
                    };

                    if let Err(e) = socket.send_to(&buf[..write], send_info.to)
                        && e.kind() != std::io::ErrorKind::WouldBlock {
                            emit!(
                                sender,
                                Http3Chunk::bad_chunk(format!("Socket send error: {e}"))
                            );
                            return;
                        }
                }

                // Receive QUIC packets
                match socket.recv_from(&mut buf) {
                    Ok((read, from)) => {
                        let recv_info = quiche::RecvInfo {
                            to: socket.local_addr().unwrap_or(peer_addr),
                            from,
                        };

                        let mut conn_guard = if let Ok(guard) = conn.lock() { guard } else {
                            emit!(
                                sender,
                                Http3Chunk::bad_chunk("Failed to lock connection".to_string())
                            );
                            return;
                        };

                        if let Err(e) = conn_guard.recv(&mut buf[..read], recv_info)
                            && e != quiche::Error::Done {
                                emit!(
                                    sender,
                                    Http3Chunk::bad_chunk(format!("QUIC recv error: {e}"))
                                );
                                return;
                            }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No more packets to read
                        std::thread::sleep(std::time::Duration::from_millis(1));
                        continue;
                    }
                    Err(e) => {
                        emit!(
                            sender,
                            Http3Chunk::bad_chunk(format!("Socket recv error: {e}"))
                        );
                        return;
                    }
                }

                // Read from streams
                let readable_streams = {
                    let conn_guard = if let Ok(guard) = conn.lock() { guard } else {
                        emit!(
                            sender,
                            Http3Chunk::bad_chunk("Failed to lock connection".to_string())
                        );
                        return;
                    };
                    conn_guard.readable().collect::<Vec<_>>()
                };

                for stream_id in readable_streams {
                    loop {
                        let (len, fin) = {
                            let mut conn_guard = if let Ok(guard) = conn.lock() { guard } else {
                                emit!(
                                    sender,
                                    Http3Chunk::bad_chunk(
                                        "Failed to lock connection".to_string()
                                    )
                                );
                                return;
                            };

                            match conn_guard.stream_recv(stream_id, &mut response_buf) {
                                Ok((len, fin)) => (len, fin),
                                Err(quiche::Error::Done) => break,
                                Err(e) => {
                                    emit!(
                                        sender,
                                        Http3Chunk::bad_chunk(format!("Stream recv error: {e}"))
                                    );
                                    return;
                                }
                            }
                        };

                        if len > 0 {
                            // Parse HTTP/3 frames
                            if let Some(chunk) = parse_http3_frame(stream_id, &response_buf[..len])
                            {
                                emit!(sender, chunk);
                            }
                        }

                        if fin {
                            emit!(sender, Http3Chunk::finished(stream_id));
                            return;
                        }
                    }
                }

                // Check if connection is closed
                let is_closed = {
                    let conn_guard = if let Ok(guard) = conn.lock() { guard } else {
                        emit!(
                            sender,
                            Http3Chunk::bad_chunk("Failed to lock connection".to_string())
                        );
                        return;
                    };
                    conn_guard.is_closed()
                };

                if is_closed {
                    return;
                }
            }
        })
    }
}

/// Simplified QPACK header encoding (for basic HTTP/3 support)
fn encode_headers(headers: &[(Vec<u8>, Vec<u8>)]) -> Vec<u8> {
    let mut encoded = Vec::new();

    // HTTP/3 HEADERS frame type (0x01)
    encoded.push(0x01);

    // Calculate headers length (simplified)
    let mut headers_data = Vec::new();
    for (name, value) in headers {
        // Simplified QPACK encoding - literal header field
        headers_data.push(0x20); // Literal header field without name reference
        if name.len() > 255 {
            return Err(format!("Header name too long: {} bytes", name.len()));
        }
        headers_data.push(name.len() as u8);
        headers_data.extend_from_slice(name);
        if value.len() > 255 {
            return Err(format!("Header value too long: {} bytes", value.len()));
        }
        headers_data.push(value.len() as u8);
        headers_data.extend_from_slice(value);
    }

    // Encode length as varint (simplified for small lengths)
    if headers_data.len() < 64 {
        // Safe cast: already checked length < 64
        encoded.push(headers_data.len() as u8);
    } else {
        // For larger lengths, use proper varint encoding
        let len = headers_data.len();
        if len < 16384 {
            // Safe casts: bit manipulation with masking ensures u8 range
            #[allow(clippy::cast_possible_truncation)]
            {
                encoded.push(0x40 | ((len >> 8) as u8));
                encoded.push((len & 0xff) as u8);
            }
        } else {
            // Safe casts: bit manipulation with masking ensures u8 range
            #[allow(clippy::cast_possible_truncation)]
            {
                encoded.push(0x80 | ((len >> 24) as u8));
                encoded.push(((len >> 16) & 0xff) as u8);
                encoded.push(((len >> 8) & 0xff) as u8);
                encoded.push((len & 0xff) as u8);
            }
        }
    }

    encoded.extend_from_slice(&headers_data);
    encoded
}

/// Parse HTTP/3 frames from received data
fn parse_http3_frame(stream_id: u64, data: &[u8]) -> Option<Http3Chunk> {
    if data.is_empty() {
        return None;
    }

    // Simplified HTTP/3 frame parsing
    let frame_type = data[0];

    match frame_type {
        0x01 => {
            // HEADERS frame
            if data.len() < 2 {
                return Some(Http3Chunk::bad_chunk("Invalid HEADERS frame".to_string()));
            }

            let length = data[1] as usize;
            if data.len() < 2 + length {
                return Some(Http3Chunk::bad_chunk(
                    "Incomplete HEADERS frame".to_string(),
                ));
            }

            // Parse headers (simplified)
            let mut headers = HeaderMap::new();
            let status = StatusCode::OK;

            // For now, assume 200 OK response
            headers.insert("content-type", HeaderValue::from_static("application/json"));

            Some(Http3Chunk::headers(stream_id, headers, status))
        }
        0x00 => {
            // DATA frame
            if data.len() < 2 {
                return Some(Http3Chunk::bad_chunk("Invalid DATA frame".to_string()));
            }

            let length = data[1] as usize;
            if data.len() < 2 + length {
                return Some(Http3Chunk::bad_chunk("Incomplete DATA frame".to_string()));
            }

            let payload = data[2..2 + length].to_vec();
            Some(Http3Chunk::data(stream_id, payload))
        }
        _ => {
            // Unknown frame type, skip
            None
        }
    }
}
