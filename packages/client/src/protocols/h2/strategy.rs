//! H2 Protocol Strategy Implementation
//!
//! Uses existing H2Connection infrastructure with thread-spawned streaming patterns.
//! Follows async-stream architecture: std::thread::spawn + emit! (NO async/await).

use ystream::{AsyncStream, emit};
use bytes::Bytes;

use crate::http::request::{HttpRequest, RequestBody};
use crate::http::response::{HttpResponse, HttpChunk};
use crate::protocols::strategy_trait::ProtocolStrategy;
use crate::protocols::response_converter::convert_http_chunks_to_response;
use crate::protocols::strategy::H2Config;




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
                serde_json::to_vec(json).ok().map(Bytes::from)
            }
            Some(RequestBody::Form(form)) => {
                serde_urlencoded::to_string(form).ok().map(|s| Bytes::from(s))
            }
            _ => None,
        };

        // Create stream using with_channel pattern (thread-spawned, no async/await)  
        let chunk_stream = AsyncStream::<HttpChunk, 1024>::with_channel(move |sender| {
            // This closure runs in dedicated thread spawned by with_channel
            // Use spawn_task for async operations within the thread context
            use ystream::spawn_task;
            use crate::tls::TlsManager;
            
            let connection_and_request_task = spawn_task(move || {
                // Use existing runtime handle if available, create minimal runtime only if needed
                let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    handle.block_on(async {
                    // Create TLS connection
                    let tls_manager = TlsManager::new();
                    let tls_stream = tls_manager
                        .create_connection(&host, port)
                        .await
                        .map_err(|e| format!("TLS connection error: {:?}", e))?;
                    
                    // Build HTTP request
                    let mut http_request = http::Request::builder()
                        .method(&method)
                        .uri(&uri)
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
                    
                    // Note: H2Config includes additional settings (keepalive, connect protocol, adaptive window, etc.)
                    // but the h2 crate's Builder API doesn't expose all configuration options
                    
                    // Perform H2 handshake
                    let (h2_client, connection) = h2_builder
                        .handshake(tls_stream)
                        .await
                        .map_err(|e| format!("H2 handshake error: {}", e))?;
                    
                    // Spawn connection driver
                    tokio::spawn(async move {
                        let _ = connection.await;
                    });
                    
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
                        
                    Ok::<_, String>((response.status(), response.headers().clone(), response.into_body()))
                })
            } else {
                let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;
                rt.block_on(async {
                    // Create TLS connection
                    let tls_manager = TlsManager::new();
                    let tls_stream = tls_manager
                        .create_connection(&host, port)
                        .await
                        .map_err(|e| format!("TLS connection error: {:?}", e))?;
                    
                    // Build HTTP request
                    let mut http_request = http::Request::builder()
                        .method(&method)
                        .uri(&uri)
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
                    
                    // Note: H2Config includes additional settings (keepalive, connect protocol, adaptive window, etc.)
                    // but the h2 crate's Builder API doesn't expose all configuration options
                    
                    // Perform H2 handshake
                    let (h2_client, connection) = h2_builder
                        .handshake(tls_stream)
                        .await
                        .map_err(|e| format!("H2 handshake error: {}", e))?;
                    
                    // Spawn connection driver
                    tokio::spawn(async move {
                        let _ = connection.await;
                    });
                    
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
                        
                    Ok::<_, String>((response.status(), response.headers().clone(), response.into_body()))
                })
            };
            
            result
        });
            
            match connection_and_request_task.collect() {
                Ok(Ok((status, headers, mut body))) => {
                    // Emit headers
                    emit!(sender, HttpChunk::Headers(status, headers));
                    
                    // Stream body using spawn_task for async operations  
                    let body_task = spawn_task(move || {
                        let mut chunks = Vec::new();
                        
                        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                            handle.block_on(async {
                            // Read body chunks
                            while let Some(chunk_result) = body.data().await {
                                match chunk_result {
                                    Ok(chunk) => chunks.push(HttpChunk::Data(chunk)),
                                    Err(e) => {
                                        chunks.push(HttpChunk::Error(format!("Body stream error: {}", e)));
                                        break;
                                    }
                                }
                            }
                            
                            // Read trailers if any
                            match body.trailers().await {
                                Ok(Some(trailers)) => chunks.push(HttpChunk::Trailers(trailers)),
                                Ok(None) => {} // No trailers
                                Err(e) => chunks.push(HttpChunk::Error(format!("Trailers error: {}", e))),
                            }
                            
                            chunks.push(HttpChunk::End);
                            Ok::<Vec<HttpChunk>, String>(chunks)
                        })
                    } else {
                        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;
                        rt.block_on(async {
                            // Read body chunks
                            while let Some(chunk_result) = body.data().await {
                                match chunk_result {
                                    Ok(chunk) => chunks.push(HttpChunk::Data(chunk)),
                                    Err(e) => {
                                        chunks.push(HttpChunk::Error(format!("Body stream error: {}", e)));
                                        break;
                                    }
                                }
                            }
                            
                            // Read trailers if any
                            match body.trailers().await {
                                Ok(Some(trailers)) => chunks.push(HttpChunk::Trailers(trailers)),
                                Ok(None) => {} // No trailers
                                Err(e) => chunks.push(HttpChunk::Error(format!("Trailers error: {}", e))),
                            }
                            
                            chunks.push(HttpChunk::End);
                            Ok::<Vec<HttpChunk>, String>(chunks)
                        })
                    };
                    
                    result
                });
                    
                    match body_task.collect() {
                        Ok(Ok(chunks)) => {
                            for chunk in chunks {
                                emit!(sender, chunk);
                            }
                        }
                        Ok(Err(e)) => {
                            emit!(sender, HttpChunk::Error(format!("Body processing error: {}", e)));
                        }
                        Err(e) => {
                            emit!(sender, HttpChunk::Error(format!("Body task error: {:?}", e)));
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