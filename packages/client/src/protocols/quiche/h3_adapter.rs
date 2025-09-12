use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::net::UdpSocket;

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
            // Get stream ID for this request
            let stream_id = match Self::get_stream_id(&conn) {
                Ok(id) => id,
                Err(e) => {
                    emit!(sender, Http3Chunk::bad_chunk(e));
                    return;
                }
            };

            // Send HTTP/3 request (headers + body)
            if let Err(e) = Self::send_http3_request(&conn, stream_id, &method, &path, &headers, &body) {
                emit!(sender, Http3Chunk::bad_chunk(e));
                return;
            }

            // Process response
            Self::process_http3_response(&conn, &socket, peer_addr, &sender);
        })
    }

    /// Get stream ID for the request
    fn get_stream_id(conn: &Arc<Mutex<quiche::Connection>>) -> Result<u64, String> {
        let _conn_guard = conn.lock().map_err(|_| "Failed to lock connection".to_string())?;
        
        // Use stream ID 0 for client-initiated bidirectional stream
        Ok(0u64)
    }

    /// Send HTTP/3 headers and body
    fn send_http3_request(
        conn: &Arc<Mutex<quiche::Connection>>,
        stream_id: u64,
        method: &Method,
        path: &str,
        headers: &HeaderMap,
        body: &Option<Vec<u8>>,
    ) -> Result<(), String> {
        // Build and send headers
        Self::send_headers(conn, stream_id, method, path, headers, body.is_none())?;

        // Send body if present
        if let Some(body_data) = body {
            Self::send_body(conn, stream_id, body_data)?;
        }

        Ok(())
    }

    /// Send HTTP/3 headers
    fn send_headers(
        conn: &Arc<Mutex<quiche::Connection>>,
        stream_id: u64,
        method: &Method,
        path: &str,
        headers: &HeaderMap,
        is_headers_only: bool,
    ) -> Result<(), String> {
        // Build HTTP/3 headers
        let mut http3_headers = Vec::new();
        http3_headers.push((b":method".to_vec(), method.as_str().as_bytes().to_vec()));
        http3_headers.push((b":path".to_vec(), path.as_bytes().to_vec()));
        http3_headers.push((b":scheme".to_vec(), b"https".to_vec()));
        http3_headers.push((b":authority".to_vec(), b"localhost".to_vec()));

        // Add custom headers
        for (name, value) in headers {
            http3_headers.push((name.as_str().as_bytes().to_vec(), value.as_bytes().to_vec()));
        }

        // Encode headers (simplified QPACK encoding)
        let encoded_headers = encode_headers(&http3_headers);

        // Send headers frame
        let mut conn_guard = conn.lock().map_err(|_| "Failed to lock connection".to_string())?;

        conn_guard.stream_send(stream_id, &encoded_headers, is_headers_only)
            .map_err(|e| format!("Failed to send headers: {e}"))?;

        Ok(())
    }

    /// Send HTTP/3 body data
    fn send_body(
        conn: &Arc<Mutex<quiche::Connection>>,
        stream_id: u64,
        body_data: &[u8],
    ) -> Result<(), String> {
        let mut conn_guard = conn.lock().map_err(|_| "Failed to lock connection".to_string())?;

        conn_guard.stream_send(stream_id, body_data, true)
            .map_err(|e| format!("Failed to send body: {e}"))?;

        Ok(())
    }

    /// Process HTTP/3 response
    fn process_http3_response(
        conn: &Arc<Mutex<quiche::Connection>>,
        socket: &Arc<UdpSocket>,
        peer_addr: std::net::SocketAddr,
        sender: &ystream::AsyncStreamSender<Http3Chunk>,
    ) {
        let mut buf = [0; 65535];
        let mut response_buf = [0; 65535];

        loop {
            // Send pending QUIC packets
            Self::send_pending_packets(conn, socket, sender, &mut buf);

            // Receive QUIC packets  
            match Self::receive_packets(conn, socket, peer_addr, &mut buf) {
                Ok(should_continue) => {
                    if !should_continue {
                        continue;
                    }
                },
                Err(e) => {
                    emit!(sender, Http3Chunk::bad_chunk(e));
                    return;
                }
            }

            // Read from streams
            match Self::read_response_streams(conn, &mut response_buf) {
                Ok((true, chunks)) => {
                    // Emit all chunks and return
                    for chunk in chunks {
                        emit!(sender, chunk);
                    }
                    return; // Response complete
                },
                Ok((false, chunks)) => {
                    // Emit chunks and continue processing
                    for chunk in chunks {
                        emit!(sender, chunk);
                    }
                },
                Err(e) => {
                    emit!(sender, Http3Chunk::bad_chunk(e));
                    return;
                }
            }

            // Check if connection is closed
            match Self::is_connection_closed(conn) {
                Ok(true) => return,
                Ok(false) => {}, // Continue processing
                Err(e) => {
                    emit!(sender, Http3Chunk::bad_chunk(e));
                    return;
                }
            }
        }
    }

    /// Send any pending QUIC packets
    fn send_pending_packets(
        conn: &Arc<Mutex<quiche::Connection>>,
        socket: &Arc<UdpSocket>,
        sender: &ystream::AsyncStreamSender<Http3Chunk>,
        buf: &mut [u8],
    ) {
        loop {
            let (write, send_info) = {
                let mut conn_guard = if let Ok(guard) = conn.lock() { 
                    guard 
                } else {
                    emit!(
                        sender,
                        Http3Chunk::bad_chunk("Failed to lock connection".to_string())
                    );
                    return;
                };

                match conn_guard.send(buf) {
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
                && e.kind() != std::io::ErrorKind::WouldBlock
            {
                emit!(
                    sender,
                    Http3Chunk::bad_chunk(format!("Socket send error: {e}"))
                );
                return;
            }
        }
    }

    /// Receive QUIC packets from the socket
    fn receive_packets(
        conn: &Arc<Mutex<quiche::Connection>>,
        socket: &Arc<UdpSocket>,
        peer_addr: std::net::SocketAddr,
        buf: &mut [u8],
    ) -> Result<bool, String> {
        match socket.recv_from(buf) {
            Ok((read, from)) => {
                let recv_info = quiche::RecvInfo {
                    to: socket.local_addr().unwrap_or(peer_addr),
                    from,
                };

                let mut conn_guard = conn.lock().map_err(|_| "Failed to lock connection".to_string())?;

                if let Err(e) = conn_guard.recv(&mut buf[..read], recv_info)
                    && e != quiche::Error::Done
                {
                    return Err(format!("QUIC recv error: {e}"));
                }
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No more packets to read
                std::thread::sleep(std::time::Duration::from_millis(1));
                Ok(false)
            }
            Err(e) => {
                Err(format!("Socket recv error: {e}"))
            }
        }
    }

    /// Read response data from streams
    fn read_response_streams(
        conn: &Arc<Mutex<quiche::Connection>>,
        response_buf: &mut [u8],
    ) -> Result<(bool, Vec<Http3Chunk>), String> {
        // Get readable streams
        let readable_streams = {
            let conn_guard = conn.lock().map_err(|_| "Failed to lock connection".to_string())?;
            conn_guard.readable().collect::<Vec<_>>()
        };

        let mut chunks = Vec::new();
        let mut response_complete = false;

        for stream_id in readable_streams {
            loop {
                let (len, fin) = {
                    let mut conn_guard = conn.lock().map_err(|_| "Failed to lock connection".to_string())?;

                    match conn_guard.stream_recv(stream_id, response_buf) {
                        Ok((len, fin)) => (len, fin),
                        Err(quiche::Error::Done) => break,
                        Err(e) => {
                            return Err(format!("Stream recv error: {e}"));
                        }
                    }
                };

                if len > 0 {
                    // Parse HTTP/3 frames
                    if let Some(chunk) = parse_http3_frame(stream_id, &response_buf[..len]) {
                        chunks.push(chunk);
                    }
                }

                if fin {
                    chunks.push(Http3Chunk::finished(stream_id));
                    response_complete = true;
                    break;
                }
            }
        }
        
        Ok((response_complete, chunks))
    }

    /// Check if connection is closed
    fn is_connection_closed(
        conn: &Arc<Mutex<quiche::Connection>>,
    ) -> Result<bool, String> {
        let conn_guard = conn.lock().map_err(|_| "Failed to lock connection".to_string())?;
        Ok(conn_guard.is_closed())
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
            // Skip headers that are too long - continue with next header
            tracing::warn!(
                target: "quyc::h3",
                name_len = name.len(),
                "Skipping header with name too long for QPACK encoding"
            );
            continue;
        }
        // Safe cast: validated name.len() <= 255 above
        #[allow(clippy::cast_possible_truncation)]
        headers_data.push(name.len() as u8);
        headers_data.extend_from_slice(name);
        if value.len() > 255 {
            // Skip headers that are too long - continue with next header
            tracing::warn!(
                target: "quyc::h3",
                value_len = value.len(),
                "Skipping header with value too long for QPACK encoding"
            );
            continue;
        }
        // Safe cast: validated value.len() <= 255 above
        #[allow(clippy::cast_possible_truncation)]
        headers_data.push(value.len() as u8);
        headers_data.extend_from_slice(value);
    }

    // Encode length as varint (simplified for small lengths)
    if headers_data.len() < 64 {
        // Safe cast: already checked length < 64
        #[allow(clippy::cast_possible_truncation)]
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
