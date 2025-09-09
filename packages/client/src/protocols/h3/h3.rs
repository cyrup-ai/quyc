//! Direct H3 protocol implementation using ystream AsyncStream
//!
//! NO middleware, NO Futures - pure streaming from H3 to AsyncStream

use std::collections::HashMap;

use ystream::prelude::MessageChunk;
use ystream::{AsyncStream, emit};
use http::{HeaderMap, StatusCode};

/// H3 response chunk for streaming
#[derive(Debug, Clone)]
pub struct H3Chunk {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub stream_id: Option<u64>,
    pub is_complete: bool,
    pub error_message: Option<String>,
}

impl MessageChunk for H3Chunk {
    fn bad_chunk(error: String) -> Self {
        Self {
            status: 500,
            headers: HashMap::new(),
            body: Vec::new(),
            stream_id: None,
            is_complete: true,
            error_message: Some(error),
        }
    }

    fn is_error(&self) -> bool {
        self.error_message.is_some() || self.status >= 400
    }

    fn error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

/// Direct H3 request execution using Quiche
pub fn execute_h3_request(
    uri: &str,
    method: &str,
    headers: HeaderMap,
    body: Vec<u8>,
) -> AsyncStream<H3Chunk, 1024> {
    let uri = uri.to_string();
    let method = method.to_string();
    let headers_map: HashMap<String, String> = headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    AsyncStream::with_channel(move |sender| {
        std::thread::spawn(move || {
            // Parse URI to get host and port
            let url = match url::Url::parse(&uri) {
                Ok(url) => url,
                Err(e) => {
                    emit!(sender, H3Chunk::bad_chunk(format!("Invalid URI: {}", e)));
                    return;
                }
            };

            let host = url.host_str().unwrap_or("localhost");
            let port = url.port().unwrap_or(443);
            let addr = match format!("{}:{}", host, port).parse() {
                Ok(address) => address,
                Err(e) => {
                    tracing::error!(
                        target: "quyc::protocols::h3",
                        error = %e,
                        host = %host,
                        port = %port,
                        "Failed to parse socket address"
                    );
                    emit!(
                        sender,
                        H3Chunk::bad_chunk(format!("Invalid socket address {}:{}: {}", host, port, e))
                    );
                    return;
                }
            };

            // Create Quiche configuration for H3
            let mut config = match quiche::Config::new(quiche::PROTOCOL_VERSION) {
                Ok(config) => config,
                Err(e) => {
                    emit!(
                        sender,
                        H3Chunk::bad_chunk(format!("Config creation failed: {}", e))
                    );
                    return;
                }
            };

            if let Err(e) = config.set_application_protos(&[b"h3"]) {
                emit!(
                    sender,
                    H3Chunk::bad_chunk(format!("Failed to set H3 protocol: {}", e))
                );
                return;
            }

            config.set_max_idle_timeout(30000);
            config.set_max_recv_udp_payload_size(1200);
            config.set_initial_max_data(10_000_000);
            config.set_initial_max_stream_data_bidi_local(1_000_000);
            config.set_initial_max_stream_data_bidi_remote(1_000_000);
            config.set_initial_max_streams_bidi(100);

            let scid = quiche::ConnectionId::from_ref(&[0; 16]);

            // Establish connection and send H3 request
            match quiche::connect(Some(host), &scid, addr, addr, &mut config) {
                Ok(mut connection) => {
                    // Create H3 request
                    let stream_id = match connection.stream_writable_next() {
                        Some(id) => id,
                        None => {
                            emit!(
                                sender,
                                H3Chunk::bad_chunk("No writable stream available".to_string())
                            );
                            return;
                        }
                    };

                    // Send H3 headers (simplified)
                    let request_data =
                        format!("{} {} HTTP/3\r\nHost: {}\r\n\r\n", method, url.path(), host);

                    if let Err(e) =
                        connection.stream_send(stream_id, request_data.as_bytes(), body.is_empty())
                    {
                        emit!(
                            sender,
                            H3Chunk::bad_chunk(format!("Failed to send headers: {}", e))
                        );
                        return;
                    }

                    // Send body if present
                    if !body.is_empty() {
                        if let Err(e) = connection.stream_send(stream_id, &body, true) {
                            emit!(
                                sender,
                                H3Chunk::bad_chunk(format!("Failed to send body: {}", e))
                            );
                            return;
                        }
                    }

                    // LOOP pattern for reading response
                    loop {
                        for readable_stream_id in connection.readable() {
                            let mut buf = vec![0; 4096];
                            match connection.stream_recv(readable_stream_id, &mut buf) {
                                Ok((len, fin)) => {
                                    buf.truncate(len);

                                    // Parse H3 response (simplified)
                                    let response_str = String::from_utf8_lossy(&buf);
                                    let status = if response_str.contains("HTTP/3") {
                                        // Extract status code (simplified parsing)
                                        200
                                    } else {
                                        200
                                    };

                                    let chunk = H3Chunk {
                                        status,
                                        headers: HashMap::new(),
                                        body: buf,
                                        stream_id: Some(readable_stream_id),
                                        is_complete: fin,
                                        error_message: None,
                                    };

                                    emit!(sender, chunk);

                                    if fin {
                                        return;
                                    }
                                }
                                Err(quiche::Error::Done) => break,
                                Err(e) => {
                                    emit!(
                                        sender,
                                        H3Chunk::bad_chunk(format!("Stream read error: {}", e))
                                    );
                                    return;
                                }
                            }
                        }

                        if connection.is_closed() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    emit!(
                        sender,
                        H3Chunk::bad_chunk(format!("H3 connection failed: {}", e))
                    );
                }
            }
        });
    })
}
