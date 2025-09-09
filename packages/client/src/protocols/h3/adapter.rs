//! HTTP/3 protocol adapter - Infrastructure Bridge
//!
//! Bridges H3Connection to canonical HttpResponse using existing streaming infrastructure.
//! Leverages H3Connection and response_converter for real response data.

use std::sync::atomic::{AtomicU64, Ordering};

use ystream::{AsyncStream, emit, spawn_task};

use crate::prelude::*;
use crate::protocols::h3::connection::H3Connection;
use crate::protocols::response_converter::convert_http_chunks_to_response;
use crate::protocols::strategy::H3Config;
use crate::protocols::core::TimeoutConfig;
use crate::http::response::HttpResponse;

static STREAM_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Execute HTTP/3 request using existing H3Connection infrastructure
///
/// This is a simple bridge that leverages the sophisticated H3 streaming
/// infrastructure already implemented to get real server response data.
pub fn execute_h3_request(
    request: HttpRequest,
    config: H3Config,
) -> Result<HttpResponse, HttpError> {
    // Create H3Connection using existing infrastructure
    let h3_connection = create_h3_connection(&config, &request)?;
    
    // Serialize request to bytes
    let request_data = serialize_http_request(&request)?;
    
    // Get stream ID (odd numbers for client streams)
    let stream_id = STREAM_ID_COUNTER.fetch_add(2, Ordering::SeqCst);
    
    // Use existing H3Connection.send_request() method
    let h3_chunk_stream = h3_connection.send_request(&request_data, stream_id);
    
    // Convert HttpChunk stream (already produced by H3Connection) to canonical format
    let http_chunk_stream = AsyncStream::with_channel(move |sender| {
        spawn_task(move || {
            for chunk in h3_chunk_stream {
                // H3Connection.send_request already returns HttpChunk - just forward it
                emit!(sender, chunk);
            }
        });
    });
    
    // Use existing response converter to get real HttpResponse with parsed status/headers
    let response = convert_http_chunks_to_response(http_chunk_stream, stream_id);
    
    Ok(response)
}

/// Create H3Connection using existing quiche infrastructure  
fn create_h3_connection(config: &H3Config, request: &HttpRequest) -> Result<H3Connection, HttpError> {
    // Create quiche config
    let mut quiche_config = quiche::Config::new(quiche::PROTOCOL_VERSION)
        .map_err(|e| HttpError::new(crate::error::Kind::Request).with(e))?;
    
    quiche_config.set_application_protos(&[b"h3"])
        .map_err(|e| HttpError::new(crate::error::Kind::Request).with(e))?;
    
    // Apply config settings
    quiche_config.set_max_idle_timeout(config.max_idle_timeout.as_millis() as u64);
    quiche_config.set_max_recv_udp_payload_size(config.max_udp_payload_size.into());
    quiche_config.set_max_send_udp_payload_size(config.max_udp_payload_size.into());
    quiche_config.set_initial_max_data(config.initial_max_data);
    quiche_config.set_initial_max_stream_data_bidi_local(config.initial_max_stream_data_bidi_local);
    quiche_config.set_initial_max_stream_data_bidi_remote(config.initial_max_stream_data_bidi_remote);
    quiche_config.set_initial_max_stream_data_uni(config.initial_max_stream_data_uni);
    quiche_config.set_initial_max_streams_bidi(config.initial_max_streams_bidi);
    quiche_config.set_initial_max_streams_uni(config.initial_max_streams_uni);
    
    // Enable early data if configured
    if config.enable_early_data {
        quiche_config.enable_early_data();
    }
    
    // Generate proper connection ID (not hardcoded)
    let scid = generate_connection_id();
    let local_addr = "127.0.0.1:0".parse()
        .map_err(|e| HttpError::new(crate::error::Kind::Request).with(e))?;
    
    // Extract peer address from request URL
    let peer_addr = extract_peer_addr_from_request(request)?;
    
    let quiche_connection = quiche::connect(None, &scid, local_addr, peer_addr, &mut quiche_config)
        .map_err(|e| HttpError::new(crate::error::Kind::Request).with(e))?;
    
    let timeout_config = TimeoutConfig {
        request_timeout: std::time::Duration::from_secs(60),
        connect_timeout: std::time::Duration::from_secs(5),
        idle_timeout: config.max_idle_timeout,
        keepalive_timeout: Some(config.max_idle_timeout / 2),
    };
    Ok(H3Connection::new(quiche_connection, timeout_config))
}

/// Extract peer address from HTTP request URL
fn extract_peer_addr_from_request(request: &HttpRequest) -> Result<std::net::SocketAddr, HttpError> {
    let uri_string = request.uri();
    
    // Parse string into URI
    let uri: http::Uri = uri_string.parse().map_err(|e| HttpError::new(crate::error::Kind::Request).with(e))?;
    
    // Extract host from URI
    let host = uri.host().ok_or_else(|| HttpError::new(crate::error::Kind::Request))?;
    
    // Extract port from URI, defaulting to 443 for HTTPS
    let port = uri.port_u16().unwrap_or(443);
    
    // Parse host and port into SocketAddr
    let addr_str = if host.contains(':') {
        // IPv6 address - wrap in brackets
        format!("[{}]:{}", host, port)
    } else {
        // IPv4 address or hostname
        format!("{}:{}", host, port)
    };
    
    addr_str.parse()
        .map_err(|e| HttpError::new(crate::error::Kind::Request).with(e))
}

/// Generate proper connection ID using timestamp (not hardcoded zeros)
fn generate_connection_id() -> quiche::ConnectionId<'static> {
    use std::time::SystemTime;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let id_bytes = timestamp.to_be_bytes();
    quiche::ConnectionId::from_vec(id_bytes.to_vec())
}

/// Serialize HttpRequest to bytes for H3 transmission
fn serialize_http_request(request: &HttpRequest) -> Result<Vec<u8>, HttpError> {
    let mut request_data = Vec::new();
    
    // Add HTTP method and path
    let method_line = format!("{} {} HTTP/3\r\n", request.method(), request.uri());
    request_data.extend_from_slice(method_line.as_bytes());
    
    // Add headers
    for (name, value) in request.headers().iter() {
        let header_line = format!("{}: {}\r\n", name, value.to_str()
            .map_err(|e| HttpError::new(crate::error::Kind::Request).with(e))?);
        request_data.extend_from_slice(header_line.as_bytes());
    }
    
    // Add separator and body
    request_data.extend_from_slice(b"\r\n");
    if let Some(body) = request.body() {
        let body_bytes = match body {
            crate::http::request::RequestBody::Bytes(bytes) => bytes.to_vec(),
            crate::http::request::RequestBody::Text(text) => text.as_bytes().to_vec(),
            crate::http::request::RequestBody::Json(json) => {
                serde_json::to_string(json).unwrap_or_default().into_bytes()
            }
            crate::http::request::RequestBody::Form(form) => {
                serde_urlencoded::to_string(form).unwrap_or_default().into_bytes()
            }
            _ => Vec::new(), // Skip complex body types for now
        };
        request_data.extend_from_slice(&body_bytes);
    }
    
    Ok(request_data)
}