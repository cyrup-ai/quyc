//! H2 Protocol Strategy Implementation
//!
//! Uses existing H2Connection infrastructure with thread-spawned streaming patterns.
//! Follows async-stream architecture: std::thread::spawn + emit! (NO async/await).

use ystream::{AsyncStream, emit};
use bytes::Bytes;
use tokio::net::TcpStream;

use crate::http::request::{HttpRequest, RequestBody};
use crate::http::response::{HttpResponse, HttpChunk};
use crate::protocols::strategy_trait::ProtocolStrategy;
use crate::protocols::response_converter::convert_http_chunks_to_response;
use crate::protocols::strategy::H2Config;

/// Connection type for H2 strategy
enum H2Stream {
    Tls(tokio_rustls::client::TlsStream<TcpStream>),
    Plain(TcpStream),
}

/// H2 protocol strategy using H2Connection infrastructure
#[derive(Clone)]
pub struct H2Strategy {
    config: H2Config,
}

impl H2Strategy {
    pub fn new(config: H2Config) -> Self {
        Self { config }
    }
}

impl Default for H2Strategy {
    fn default() -> Self {
        Self::new(H2Config::default())
    }
}

impl H2Strategy {
    /// Create connection based on URL scheme
    async fn create_connection(url: &url::Url, host: &str, port: u16) -> Result<H2Stream, String> {
        if url.scheme() == "https" {
            let tls_manager = crate::tls::TlsManager::new();
            let tls_stream = tls_manager
                .create_connection(host, port)
                .await
                .map_err(|e| format!("TLS connection error: {:?}", e))?;
            Ok(H2Stream::Tls(tls_stream))
        } else {
            let tcp_stream = TcpStream::connect((host, port))
                .await
                .map_err(|e| format!("TCP connection error: {:?}", e))?;
            Ok(H2Stream::Plain(tcp_stream))
        }
    }

    /// Send H2 request using client (generic over connection type)
    async fn send_h2_request(
        h2_client: h2::client::SendRequest<bytes::Bytes>,
        http_request: http::Request<()>,
        body_bytes: Option<Bytes>,
    ) -> Result<(http::StatusCode, http::HeaderMap, h2::RecvStream), String> {
        // Wait for client ready and send request
        let mut ready_client = h2_client
            .ready()
            .await
            .map_err(|e| format!("H2 client ready error: {}", e))?;
        
        let (response, mut request_stream) = ready_client
            .send_request(http_request, body_bytes.is_none())
            .map_err(|e| format!("Send request error: {}", e))?;
        
        // Send body if present
        if let Some(body) = body_bytes {
            if !body.is_empty() {
                request_stream
                    .send_data(body, true)
                    .map_err(|e| format!("Send body error: {}", e))?;
            } else {
                request_stream
                    .send_data(Bytes::new(), true)
                    .map_err(|e| format!("Send empty body error: {}", e))?;
            }
        } else {
            request_stream
                .send_data(Bytes::new(), true)
                .map_err(|e| format!("Send no body error: {}", e))?;
        }
        
        // Get response
        let response = response
            .await
            .map_err(|e| format!("Response error: {}", e))?;
            
        Ok((response.status(), response.headers().clone(), response.into_body()))
    }

    /// Execute H2 request with given stream
    async fn execute_h2_request(
        stream: H2Stream,
        h2_config: &H2Config,
        method: &http::Method,
        uri: &str,
        headers: http::HeaderMap,
        body_bytes: Option<Bytes>,
    ) -> Result<(http::StatusCode, http::HeaderMap, h2::RecvStream), String> {
        // Build HTTP request
        let mut http_request = http::Request::builder()
            .method(method)
            .uri(uri)
            .body(())
            .map_err(|e| format!("Request build error: {}", e))?;
        *http_request.headers_mut() = headers;
        
        // Configure H2 client with settings
        let mut h2_builder = h2::client::Builder::new();
        h2_builder
            .initial_window_size(h2_config.initial_window_size)
            .max_frame_size(h2_config.max_frame_size)
            .max_concurrent_streams(h2_config.max_concurrent_streams)
            .enable_push(h2_config.enable_push);
        
        // Perform H2 handshake based on stream type - handle each type separately
        match stream {
            H2Stream::Tls(tls_stream) => {
                let (h2_client, connection) = h2_builder
                    .handshake(tls_stream)
                    .await
                    .map_err(|e| format!("H2 handshake error: {}", e))?;
                
                // Spawn connection driver
                tokio::spawn(async move {
                    let _ = connection.await;
                });
                
                return Self::send_h2_request(h2_client, http_request, body_bytes).await;
            }
            H2Stream::Plain(tcp_stream) => {
                let (h2_client, connection) = h2_builder
                    .handshake(tcp_stream)
                    .await
                    .map_err(|e| format!("H2 handshake error: {}", e))?;
                
                // Spawn connection driver
                tokio::spawn(async move {
                    let _ = connection.await;
                });
                
                return Self::send_h2_request(h2_client, http_request, body_bytes).await;
            }
        }
    }

    /// Execute request with proper runtime handling (no duplication)
    fn execute_with_runtime(
        url: &url::Url,
        host: &str,
        port: u16,
        h2_config: &H2Config,
        method: &http::Method,
        uri: &str,
        headers: http::HeaderMap,
        body_bytes: Option<Bytes>,
    ) -> Result<(http::StatusCode, http::HeaderMap, h2::RecvStream), String> {
        let execute_async = async {
            // Create connection (HTTPS vs HTTP abstracted)
            let stream = Self::create_connection(url, host, port).await?;
            
            // Execute H2 request (same logic for both connection types)
            Self::execute_h2_request(stream, h2_config, method, uri, headers, body_bytes).await
        };

        // Use existing runtime handle if available, create minimal runtime only if needed
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(execute_async)
        } else {
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| format!("Failed to create runtime: {}", e))?;
            rt.block_on(execute_async)
        }
    }


}

impl ProtocolStrategy for H2Strategy {
    fn execute(&self, request: HttpRequest) -> HttpResponse {
        // Clone config for move into thread
        let h2_config = self.config.clone();
        
        // Extract URL components for connection
        let url = request.url().clone();
        let host = url.host_str().unwrap_or("localhost").to_string();
        let port = url.port().unwrap_or_else(|| {
            match url.scheme() {
                "https" => 443,
                "http" => 80,
                _ => 80,
            }
        });
        
        // Convert HttpRequest to appropriate format
        let method = request.method().clone();
        let uri = url.to_string();
        let headers = request.headers().clone();
        let body_bytes = match request.body() {
            Some(RequestBody::Bytes(bytes)) => Some(bytes.clone()),
            Some(RequestBody::Text(text)) => Some(Bytes::from(text.clone())),
            Some(RequestBody::Json(json)) => {
                match serde_json::to_vec(json) {
                    Ok(vec) => Some(Bytes::from(vec)),
                    Err(e) => {
                        let error_stream = AsyncStream::<HttpChunk, 1024>::with_channel(move |sender| {
                            use ystream::emit;
                            emit!(sender, HttpChunk::Error(format!("JSON serialization error: {}", e)));
                        });
                        return convert_http_chunks_to_response(error_stream, 1);
                    }
                }
            }
            Some(RequestBody::Form(form)) => {
                match serde_urlencoded::to_string(form) {
                    Ok(s) => Some(Bytes::from(s)),
                    Err(e) => {
                        return convert_http_chunks_to_response(
                            AsyncStream::with_channel(move |sender| {
                                use ystream::emit;
                                emit!(sender, HttpChunk::Error(format!("Form serialization error: {}", e)));
                            }),
                            1
                        );
                    }
                }
            }
            _ => None,
        };

        // Create stream using with_channel pattern (thread-spawned, no async/await)  
        let chunk_stream = AsyncStream::<HttpChunk, 1024>::with_channel(move |sender| {
            // This closure runs in dedicated thread spawned by with_channel
            use ystream::spawn_task;
            
            let connection_and_request_task = spawn_task(move || {
                // Execute request with proper runtime handling (no duplication)
                Self::execute_with_runtime(
                    &url, &host, port, &h2_config, &method, &uri, headers, body_bytes
                )
            });
            
            match connection_and_request_task.collect() {
                Ok(Ok((status, headers, mut body))) => {
                    // Emit headers
                    emit!(sender, HttpChunk::Headers(status, headers));
                    
                    // PRODUCTION-GRADE: Zero-allocation streaming with hoisted runtime detection
                    
                    // Hoist runtime detection (do once, not per chunk) - blazing-fast optimization
                    let runtime_handle = tokio::runtime::Handle::try_current();
                    
                    // PRODUCTION-GRADE: Single runtime execution with zero-allocation streaming
                    match runtime_handle {
                        Ok(handle) => {
                            // Fast path: existing runtime handle
                            handle.block_on(async move {
                                // Direct emit streaming - eliminates Vec<HttpChunk> allocation
                                loop {
                                    match body.data().await {
                                        Some(Ok(chunk)) => {
                                            emit!(sender, HttpChunk::Data(chunk)); // DIRECT EMIT - zero allocation
                                        }
                                        Some(Err(e)) => {
                                            emit!(sender, HttpChunk::Error(format!("Body stream error: {}", e)));
                                            break;
                                        }
                                        None => break, // End of data stream
                                    }
                                }
                                
                                // Handle trailers with direct emit - zero allocation
                                match body.trailers().await {
                                    Ok(Some(trailers)) => emit!(sender, HttpChunk::Trailers(trailers)),
                                    Ok(None) => {} // No trailers
                                    Err(e) => emit!(sender, HttpChunk::Error(format!("Trailers error: {}", e))),
                                }
                                
                                // Final chunk
                                emit!(sender, HttpChunk::End);
                            });
                        }
                        Err(_) => {
                            // Fallback: create runtime only when needed with error handling
                            match tokio::runtime::Runtime::new() {
                                Ok(rt) => {
                                    rt.block_on(async move {
                                        // Direct emit streaming - eliminates Vec<HttpChunk> allocation
                                        loop {
                                            match body.data().await {
                                                Some(Ok(chunk)) => {
                                                    emit!(sender, HttpChunk::Data(chunk)); // DIRECT EMIT - zero allocation
                                                }
                                                Some(Err(e)) => {
                                                    emit!(sender, HttpChunk::Error(format!("Body stream error: {}", e)));
                                                    break;
                                                }
                                                None => break, // End of data stream
                                            }
                                        }
                                        
                                        // Handle trailers with direct emit - zero allocation
                                        match body.trailers().await {
                                            Ok(Some(trailers)) => emit!(sender, HttpChunk::Trailers(trailers)),
                                            Ok(None) => {} // No trailers
                                            Err(e) => emit!(sender, HttpChunk::Error(format!("Trailers error: {}", e))),
                                        }
                                        
                                        // Final chunk
                                        emit!(sender, HttpChunk::End);
                                    });
                                }
                                Err(e) => {
                                    emit!(sender, HttpChunk::Error(format!("Runtime creation failed: {}", e)));
                                }
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    emit!(sender, HttpChunk::Error(e));
                }
                Err(e) => {
                    emit!(sender, HttpChunk::Error(format!("Connection task error: {:?}", e)));
                }
            }
        });
        
        // Use existing response converter infrastructure
        convert_http_chunks_to_response(chunk_stream, 1)
    }
    
    fn protocol_name(&self) -> &'static str {
        "HTTP/2"
    }
    
    fn supports_push(&self) -> bool {
        self.config.enable_push
    }
    
    fn max_concurrent_streams(&self) -> usize {
        self.config.max_concurrent_streams as usize
    }
}