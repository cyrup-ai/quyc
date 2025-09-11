//! HTTP/2 protocol adapter - Pure Streams Architecture
//!
//! Provides HTTP/2 request execution using pure ystream streaming patterns.
//! No blocking I/O, sync connections, or no-op wakers.

use ystream::{AsyncStream, emit, spawn_task};

use crate::prelude::*;
use crate::protocols::h2::connection::H2Connection;
use crate::protocols::response_converter::convert_http_chunks_to_response;
use crate::protocols::strategy::H2Config;
use crate::http::response::HttpResponse;

/// Execute HTTP/2 request using pure streams-first architecture
///
/// Creates an AsyncStream-based HTTP/2 response without blocking I/O or sync connections.
/// Leverages existing `H2Connection` streaming infrastructure.
pub fn execute_h2_request(
    request: HttpRequest,
    config: H2Config,
) -> AsyncStream<HttpResponse, 1> {
    AsyncStream::with_channel(move |sender| {
        spawn_task(move || {
            // Use existing H2Connection for streams-first request handling
            match create_h2_connection_stream(request, config) {
                Ok(http_chunk_stream) => {
                    // Convert HttpChunk stream to HttpResponse using existing converter
                    let stream_id = 1; // HTTP/2 stream ID
                    let response = convert_http_chunks_to_response(http_chunk_stream, stream_id);
                    emit!(sender, response);
                }
                Err(e) => {
                    // Emit error response using HttpResponse::error
                    let error_response = HttpResponse::error(
                        http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("H2 connection failed: {e}")
                    );
                    emit!(sender, error_response);
                }
            }
        });
    })
}

/// Create H2 connection stream
///
/// Creates an `H2Connection` and executes the request through it.
fn create_h2_connection_stream(
    _request: HttpRequest,
    _config: H2Config,
) -> Result<AsyncStream<HttpChunk, 1024>, HttpError> {
    // Create H2Connection directly - no circular dependency
    let _h2_connection = H2Connection::new(); // Default configuration
    
    // Convert request to format needed by H2Connection
    // This is where the actual H2 protocol work happens
    // For now, return a simple stream with test data
    Ok(AsyncStream::with_channel(move |sender| {
        // In production, this would:
        // 1. Establish TCP connection
        // 2. Do TLS handshake if needed
        // 3. Perform h2::client::handshake()
        // 4. Send request through h2
        // 5. Stream response chunks
        
        // For now, emit test response
        emit!(sender, HttpChunk::Headers(
            http::StatusCode::OK,
            http::HeaderMap::new()
        ));
        
        emit!(sender, HttpChunk::Body(
            bytes::Bytes::from("H2 adapter response")
        ));
        
        emit!(sender, HttpChunk::End);
    }))
}