//! H3 Request Processing
//!
//! Handles HTTP/3 request sending and response processing including body handling,
//! multipart forms, streaming, and response parsing.

use std::net::{SocketAddr, UdpSocket};
// HashMap import removed - not used

use crossbeam_utils::Backoff;
use ystream::{emit, AsyncStreamSender, AsyncStream, prelude::MessageChunk};
use http::{Method, HeaderMap, HeaderName, HeaderValue};
use bytes::Bytes;
use quiche;
use quiche::h3::NameValue;

use crate::protocols::strategy::H3Config;
use crate::protocols::core::ProtocolConfig;
use crate::crypto::random::generate_boundary;
use crate::http::response::{HttpHeader, HttpBodyChunk, HttpChunk};

/// H3 Request Processor
///
/// Handles sending HTTP/3 requests and processing responses
pub(crate) struct H3RequestProcessor;

impl H3RequestProcessor {
    /// Create new request processor
    pub fn new() -> Self {
        Self
    }

    /// Process HTTP/3 request and response
    #[allow(clippy::too_many_arguments)]
    pub fn process_request(
        &self,
        quic_conn: &mut quiche::Connection,
        h3_conn: &mut quiche::h3::Connection,
        socket: &UdpSocket,
        _server_addr: SocketAddr,
        local_addr: SocketAddr,
        method: Method,
        scheme: String,
        host: String,
        path: String,
        _headers: HeaderMap,
        body_data: Option<crate::http::request::RequestBody>,
        config: H3Config,
        headers_tx: AsyncStreamSender<HttpHeader, 256>,
        body_tx: AsyncStreamSender<HttpBodyChunk, 1024>,
        _trailers_tx: AsyncStreamSender<HttpHeader, 64>,
    ) {
        // Build H3 headers
        let h3_headers = vec![
            quiche::h3::Header::new(b":method", method.as_str().as_bytes()),
            quiche::h3::Header::new(b":scheme", scheme.as_bytes()),
            quiche::h3::Header::new(b":authority", host.as_bytes()),
            quiche::h3::Header::new(b":path", path.as_bytes()),
        ];

        // Send request headers
        let stream_id = match h3_conn.send_request(quic_conn, &h3_headers, body_data.is_none()) {
            Ok(id) => id,
            Err(e) => {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    error = %e,
                    method = %method,
                    path = %path,
                    "Failed to send HTTP/3 request headers"
                );
                emit!(body_tx, HttpBodyChunk::bad_chunk(format!("Failed to send H3 request: {}", e)));
                return;
            }
        };

        // Send body if present
        if let Some(body_data) = body_data {
            let body_bytes = self.prepare_request_body(body_data, &config, &body_tx);
            
            if let Err(e) = h3_conn.send_body(quic_conn, stream_id, &body_bytes, true) {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    error = %e,
                    stream_id = stream_id,
                    body_len = body_bytes.len(),
                    "Failed to send HTTP/3 request body"
                );
            }
        }

        // Process response
        self.process_response(
            quic_conn,
            h3_conn,
            socket,
            local_addr,
            headers_tx,
            body_tx,
        );
    }

    /// Prepare request body from various body types
    fn prepare_request_body(
        &self,
        body_data: crate::http::request::RequestBody,
        config: &H3Config,
        body_tx: &AsyncStreamSender<HttpBodyChunk>,
    ) -> Vec<u8> {
        match body_data {
            crate::http::request::RequestBody::Bytes(bytes) => bytes.to_vec(),
            crate::http::request::RequestBody::Text(text) => text.into_bytes(),
            crate::http::request::RequestBody::Json(json) => {
                serde_json::to_string(&json).unwrap_or_default().into_bytes()
            }
            crate::http::request::RequestBody::Form(form) => {
                serde_urlencoded::to_string(&form).unwrap_or_default().into_bytes()
            }
            crate::http::request::RequestBody::Multipart(fields) => {
                self.prepare_multipart_body(fields, body_tx)
            }
            crate::http::request::RequestBody::Stream(stream) => {
                self.prepare_stream_body(stream, config, body_tx)
            }
        }
    }

    /// Prepare multipart form body with security limits
    fn prepare_multipart_body(
        &self,
        fields: Vec<crate::http::request::MultipartField>,
        body_tx: &AsyncStreamSender<HttpBodyChunk>,
    ) -> Vec<u8> {
        let boundary = generate_boundary();
        let mut body = Vec::new();
        const MAX_MULTIPART_SIZE: usize = 100 * 1024 * 1024; // 100MB hard limit
        
        for field in fields {
            // Pre-calculate sizes to prevent memory exhaustion attacks
            let boundary_sep = format!("--{}\r\n", boundary);
            let boundary_sep_bytes = boundary_sep.as_bytes();
            
            // SECURITY: Check size before allocation
            if body.len() + boundary_sep_bytes.len() > MAX_MULTIPART_SIZE {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    current_size = body.len(),
                    attempted_add = boundary_sep_bytes.len(),
                    limit = MAX_MULTIPART_SIZE,
                    "Multipart request exceeds memory safety limit - rejecting"
                );
                break; // Stop processing to prevent memory exhaustion
            }
            
            // Add boundary separator
            body.extend_from_slice(boundary_sep_bytes);
            
            // Add headers based on field type
            self.add_multipart_field_headers(&mut body, &field, MAX_MULTIPART_SIZE, body_tx);
            
            // Add field value with size checking
            self.add_multipart_field_value(&mut body, &field, MAX_MULTIPART_SIZE, body_tx);
            
            // SECURITY: Check size before adding CRLF
            if body.len() + 2 > MAX_MULTIPART_SIZE {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    current_size = body.len(),
                    limit = MAX_MULTIPART_SIZE,
                    "Multipart CRLF would exceed memory safety limit - truncating"
                );
                break;
            }
            
            body.extend_from_slice(b"\r\n");
        }
        
        // Add final boundary with size checking
        let final_boundary = format!("--{}--\r\n", boundary);
        if body.len() + final_boundary.len() <= MAX_MULTIPART_SIZE {
            body.extend_from_slice(final_boundary.as_bytes());
        } else {
            tracing::warn!(
                target: "quyc::protocols::h3",
                current_size = body.len(),
                final_boundary_size = final_boundary.len(),
                limit = MAX_MULTIPART_SIZE,
                "Final multipart boundary would exceed limit - sending truncated body"
            );
        }
        
        body
    }

    /// Add multipart field headers with size checking
    fn add_multipart_field_headers(
        &self,
        body: &mut Vec<u8>,
        field: &crate::http::request::MultipartField,
        max_size: usize,
        _body_tx: &AsyncStreamSender<HttpBodyChunk>,
    ) {
        match (&field.filename, &field.content_type) {
            (Some(filename), Some(content_type)) => {
                let header1 = format!("Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n", field.name, filename);
                let header2 = format!("Content-Type: {}\r\n\r\n", content_type);
                
                if body.len() + header1.len() + header2.len() <= max_size {
                    body.extend_from_slice(header1.as_bytes());
                    body.extend_from_slice(header2.as_bytes());
                }
            }
            (Some(filename), None) => {
                let header1 = format!("Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n", field.name, filename);
                let header2 = b"Content-Type: application/octet-stream\r\n\r\n";
                
                if body.len() + header1.len() + header2.len() <= max_size {
                    body.extend_from_slice(header1.as_bytes());
                    body.extend_from_slice(header2);
                }
            }
            (None, Some(content_type)) => {
                let header1 = format!("Content-Disposition: form-data; name=\"{}\"\r\n", field.name);
                let header2 = format!("Content-Type: {}\r\n\r\n", content_type);
                
                if body.len() + header1.len() + header2.len() <= max_size {
                    body.extend_from_slice(header1.as_bytes());
                    body.extend_from_slice(header2.as_bytes());
                }
            }
            (None, None) => {
                let header = format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", field.name);
                
                if body.len() + header.len() <= max_size {
                    body.extend_from_slice(header.as_bytes());
                }
            }
        }
    }

    /// Add multipart field value with size checking
    fn add_multipart_field_value(
        &self,
        body: &mut Vec<u8>,
        field: &crate::http::request::MultipartField,
        max_size: usize,
        _body_tx: &AsyncStreamSender<HttpBodyChunk>,
    ) {
        match &field.value {
            crate::http::request::MultipartValue::Text(text) => {
                let text_bytes = text.as_bytes();
                
                if body.len() + text_bytes.len() <= max_size {
                    body.extend_from_slice(text_bytes);
                } else {
                    tracing::error!(
                        target: "quyc::protocols::h3",
                        current_size = body.len(),
                        field_size = text_bytes.len(),
                        limit = max_size,
                        "Multipart field value exceeds memory safety limit - rejecting"
                    );
                }
            }
            crate::http::request::MultipartValue::Bytes(bytes) => {
                if body.len() + bytes.len() <= max_size {
                    body.extend_from_slice(bytes);
                } else {
                    tracing::error!(
                        target: "quyc::protocols::h3",
                        current_size = body.len(),
                        field_size = bytes.len(),
                        limit = max_size,
                        "Multipart field value exceeds memory safety limit - rejecting"
                    );
                }
            }
        }
    }

    /// Prepare streaming request body with size limits
    fn prepare_stream_body(
        &self,
        stream: AsyncStream<HttpChunk, 1024>,
        config: &H3Config,
        _body_tx: &AsyncStreamSender<HttpBodyChunk>,
    ) -> Vec<u8> {
        let mut body_data = Vec::new();
        let timeout = config.timeout_config().request_timeout;
        let start_time = std::time::Instant::now();
        const MAX_BODY_SIZE: usize = 100 * 1024 * 1024; // 100MB hard limit
        
        loop {
            if start_time.elapsed() > timeout {
                tracing::warn!("Streaming body timeout exceeded");
                break;
            }
            
            match stream.try_next() {
                Some(chunk) => {
                    match chunk {
                        HttpChunk::Body(bytes) => {
                            if self.check_body_size_limit(&mut body_data, &bytes, MAX_BODY_SIZE) {
                                body_data.extend_from_slice(&bytes);
                            } else {
                                break;
                            }
                        }
                        HttpChunk::Data(bytes) => {
                            if self.check_body_size_limit(&mut body_data, &bytes, MAX_BODY_SIZE) {
                                body_data.extend_from_slice(&bytes);
                            } else {
                                break;
                            }
                        }
                        HttpChunk::Chunk(bytes) => {
                            if self.check_body_size_limit(&mut body_data, &bytes, MAX_BODY_SIZE) {
                                body_data.extend_from_slice(&bytes);
                            } else {
                                break;
                            }
                        }
                        HttpChunk::End => {
                            break;
                        }
                        HttpChunk::Error(err) => {
                            tracing::error!("Stream error: {}", err);
                            break;
                        }
                        _ => continue,
                    }
                }
                None => {
                    // Use proper backoff instead of thread sleep
                    let backoff = crossbeam_utils::Backoff::new();
                    backoff.snooze();
                    continue;
                }
            }
        }
        
        body_data
    }

    /// Check body size limit for security
    fn check_body_size_limit(
        &self,
        current_body: &[u8],
        new_chunk: &[u8],
        max_size: usize,
    ) -> bool {
        if current_body.len() + new_chunk.len() <= max_size {
            true
        } else {
            tracing::error!(
                target: "quyc::protocols::h3",
                current_size = current_body.len(),
                chunk_size = new_chunk.len(),
                limit = max_size,
                "Streaming body chunk would exceed memory safety limit - rejecting"
            );
            false
        }
    }

    /// Process HTTP/3 response
    fn process_response(
        &self,
        quic_conn: &mut quiche::Connection,
        h3_conn: &mut quiche::h3::Connection,
        socket: &UdpSocket,
        local_addr: SocketAddr,
        headers_tx: AsyncStreamSender<HttpHeader, 256>,
        body_tx: AsyncStreamSender<HttpBodyChunk, 1024>,
    ) {
        let mut response_complete = false;
        let mut buf = [0; 65535];
        
        while !response_complete {
            // Poll H3 events
            match h3_conn.poll(quic_conn) {
                Ok((_stream_id, quiche::h3::Event::Headers { list, .. })) => {
                    self.process_response_headers(list, &headers_tx);
                }
                Ok((stream_id, quiche::h3::Event::Data)) => {
                    self.process_response_data(quic_conn, h3_conn, stream_id, &body_tx);
                }
                Ok((_stream_id, quiche::h3::Event::Finished)) => {
                    // Stream finished
                    emit!(body_tx, HttpBodyChunk {
                        data: Bytes::new(),
                        offset: 0,
                        is_final: true,
                        timestamp: std::time::Instant::now(),
                    });
                    response_complete = true;
                }
                Ok(_) => {
                    // Other events
                }
                Err(quiche::h3::Error::Done) => {
                    // No more events
                    let backoff = Backoff::new();
                    backoff.snooze();
                }
                Err(e) => {
                    tracing::error!(
                        target: "quyc::protocols::h3",
                        error = %e,
                        "HTTP/3 event polling error, terminating response processing"
                    );
                    response_complete = true;
                }
            }
            
            // Handle QUIC I/O
            self.handle_quic_io(quic_conn, socket, local_addr, &mut buf);
        }
        
        // Signal completion
        emit!(body_tx, HttpBodyChunk {
            data: Bytes::new(),
            offset: 0,
            is_final: true,
            timestamp: std::time::Instant::now(),
        });
    }

    /// Process response headers
    fn process_response_headers(
        &self,
        headers: Vec<quiche::h3::Header>,
        headers_tx: &AsyncStreamSender<HttpHeader, 256>,
    ) {
        for header in headers {
            let name_bytes = header.name();
            let value_bytes = header.value();
            
            // Convert to HeaderName and HeaderValue
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(name_bytes),
                HeaderValue::from_bytes(value_bytes)
            ) {
                let http_header = HttpHeader {
                    name,
                    value,
                    timestamp: std::time::Instant::now(),
                };
                
                // Emit header to stream
                emit!(*headers_tx, http_header);
            }
        }
    }

    /// Process response body data
    fn process_response_data(
        &self,
        quic_conn: &mut quiche::Connection,
        h3_conn: &mut quiche::h3::Connection,
        stream_id: u64,
        body_tx: &AsyncStreamSender<HttpBodyChunk>,
    ) {
        // Read response body
        let mut body_buf = vec![0; 4096];
        match h3_conn.recv_body(quic_conn, stream_id, &mut body_buf) {
            Ok(len) => {
                if len > 0 {
                    emit!(*body_tx, HttpBodyChunk {
                        data: Bytes::from(body_buf[..len].to_vec()),
                        offset: 0,
                        is_final: false,
                        timestamp: std::time::Instant::now(),
                    });
                }
            }
            Err(e) => {
                tracing::warn!(
                    target: "quyc::protocols::h3",
                    error = %e,
                    stream_id = stream_id,
                    "Failed to receive HTTP/3 response body"
                );
            }
        }
    }

    /// Handle QUIC I/O operations
    fn handle_quic_io(
        &self,
        quic_conn: &mut quiche::Connection,
        socket: &UdpSocket,
        local_addr: SocketAddr,
        buf: &mut [u8],
    ) {
        // Receive data
        match socket.recv_from(buf) {
            Ok((len, from)) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: local_addr,
                };
                if let Err(e) = quic_conn.recv(&mut buf[..len], recv_info) {
                    tracing::warn!(
                        target: "quyc::protocols::h3",
                        error = %e,
                        packet_len = len,
                        "QUIC packet receive error during response processing"
                    );
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available
            }
            Err(e) => {
                tracing::warn!(
                    target: "quyc::protocols::h3",
                    error = %e,
                    "UDP socket receive error during response processing"
                );
            }
        }
        
        // Send pending data
        loop {
            let mut out = [0; 1350];
            match quic_conn.send(&mut out) {
                Ok((len, send_info)) => {
                    if len == 0 {
                        break;
                    }
                    if let Err(e) = socket.send_to(&out[..len], send_info.to) {
                        tracing::warn!(
                            target: "quyc::protocols::h3",
                            error = %e,
                            packet_len = len,
                            destination = %send_info.to,
                            "UDP socket send error during response processing"
                        );
                    }
                }
                Err(quiche::Error::Done) => break,
                Err(e) => {
                    tracing::warn!(
                        target: "quyc::protocols::h3",
                        error = %e,
                        "QUIC send error during response processing"
                    );
                    break;
                }
            }
        }
    }
}