//! Direct H2 protocol implementation using ystream AsyncStream
//!
//! NO middleware, NO Futures - pure streaming from H2 to AsyncStream

use std::collections::HashMap;
// TcpStream import removed - not used

use ystream::prelude::MessageChunk;
use ystream::{AsyncStream, emit};
use futures::executor::block_on;
use http::{HeaderMap, Method, Uri};

/// H2 response chunk for streaming
#[derive(Debug, Clone, Default)]
pub struct H2Chunk {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub stream_id: Option<u32>,
    pub is_complete: bool,
    pub error_message: Option<String>,
}

impl MessageChunk for H2Chunk {
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

/// Direct H2 request execution
pub fn execute_h2_request(
    uri: Uri,
    method: Method,
    _headers: HeaderMap,
    body: Vec<u8>,
) -> AsyncStream<H2Chunk, 1024> {
    AsyncStream::with_channel(move |sender| {
        std::thread::spawn(move || {
            let host = uri.host().unwrap_or("localhost");
            let port = uri.port_u16().unwrap_or(80);

            // Create tokio runtime for async operations
            let rt = match tokio::runtime::Runtime::new() {
                Ok(runtime) => runtime,
                Err(e) => {
                    emit!(
                        sender,
                        H2Chunk::bad_chunk(format!("Failed to create tokio runtime: {}", e))
                    );
                    return;
                }
            };
            
            // Establish TCP connection using tokio
            let tcp = match rt.block_on(tokio::net::TcpStream::connect(format!("{}:{}", host, port))) {
                Ok(tcp) => tcp,
                Err(e) => {
                    emit!(
                        sender,
                        H2Chunk::bad_chunk(format!("TCP connection failed: {}", e))
                    );
                    return;
                }
            };

            // Perform H2 handshake with tokio TcpStream
            match rt.block_on(h2::client::handshake(tcp)) {
                Ok((h2, connection)) => {
                    // Spawn connection task
                    std::thread::spawn(move || {
                        let _ = connection;
                    });

                    // LOOP pattern for H2 streaming
                    loop {
                        match rt.block_on(h2.ready()) {
                            Ok(mut ready_h2) => {
                                // Use the ready h2 client
                                let req = match http::Request::builder()
                                    .method(method.clone())
                                    .uri(uri.clone())
                                    .body(())
                                {
                                    Ok(request) => request,
                                    Err(e) => {
                                        emit!(
                                            sender,
                                            H2Chunk::bad_chunk(format!("Failed to build HTTP request: {}", e))
                                        );
                                        return;
                                    }
                                };

                                match ready_h2.send_request(req, false) {
                                    Ok((response_future, mut stream)) => {
                                        // Send request body
                                        if !body.is_empty() {
                                            let _ = stream.send_data(body.clone().into(), true);
                                        }

                                        // Read response (blocking)
                                        match block_on(response_future) {
                                            Ok(resp) => {
                                                let chunk = H2Chunk {
                                                    status: resp.status().as_u16(),
                                                    headers: HashMap::new(),
                                                    body: Vec::new(),
                                                    stream_id: None,
                                                    is_complete: true,
                                                    error_message: None,
                                                };
                                                emit!(sender, chunk);
                                                break;
                                            }
                                            Err(e) => {
                                                emit!(
                                                    sender,
                                                    H2Chunk::bad_chunk(format!(
                                                        "H2 response error: {}",
                                                        e
                                                    ))
                                                );
                                                break;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        emit!(
                                            sender,
                                            H2Chunk::bad_chunk(format!("H2 send error: {}", e))
                                        );
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                emit!(sender, H2Chunk::bad_chunk(format!("H2 ready error: {}", e)));
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    emit!(
                        sender,
                        H2Chunk::bad_chunk(format!("H2 handshake failed: {}", e))
                    );
                }
            }
        });
    })
}
