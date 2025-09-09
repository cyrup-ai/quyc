//! H3 Protocol Strategy Core Implementation
//!
//! Main H3Strategy struct and protocol strategy interface implementation.

// SocketAddr import removed - not used
use std::sync::atomic::{AtomicU64, Ordering};

use ystream::{AsyncStream, spawn_task};
use http::{StatusCode, Version};
// Bytes import removed - not used

use crate::protocols::strategy_trait::ProtocolStrategy;
// ProtocolConfig import removed - not used
use crate::protocols::strategy::H3Config;
use crate::http::{HttpRequest, HttpResponse};
use crate::http::response::{HttpBodyChunk, HttpHeader};

use super::connection::H3Connection;
use super::processing::H3RequestProcessor;

// Global connection ID counter for H3 connections
static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

/// HTTP/3 Protocol Strategy
///
/// Encapsulates all HTTP/3 and QUIC complexity including:
/// - UDP socket management
/// - QUIC connection establishment
/// - HTTP/3 stream management
/// - Connection pooling
pub struct H3Strategy {
    config: H3Config,
}

impl H3Strategy {
    /// Create a new H3 strategy with the given configuration
    pub fn new(config: H3Config) -> Self {
        Self {
            config,
        }
    }
    
    /// Convert H3Config to quiche::Config
    pub(crate) fn create_quiche_config(&self) -> Result<quiche::Config, crate::error::HttpError> {
        let mut config = match quiche::Config::new(quiche::PROTOCOL_VERSION) {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    error = %e,
                    "Failed to create QUICHE config with protocol version"
                );
                return Err(crate::error::HttpError::new(crate::error::Kind::Request));
            }
        };
        
        // Set transport parameters from H3Config
        config.set_initial_max_data(self.config.initial_max_data);
        config.set_initial_max_streams_bidi(self.config.initial_max_streams_bidi);
        config.set_initial_max_streams_uni(self.config.initial_max_streams_uni);
        config.set_initial_max_stream_data_bidi_local(self.config.initial_max_stream_data_bidi_local);
        config.set_initial_max_stream_data_bidi_remote(self.config.initial_max_stream_data_bidi_remote);
        config.set_initial_max_stream_data_uni(self.config.initial_max_stream_data_uni);
        
        // Set idle timeout
        config.set_max_idle_timeout(self.config.max_idle_timeout.as_millis() as u64);
        
        // Set UDP payload size
        config.set_max_recv_udp_payload_size(self.config.max_udp_payload_size as usize);
        config.set_max_send_udp_payload_size(self.config.max_udp_payload_size as usize);
        
        // Enable early data if configured
        if self.config.enable_early_data {
            config.enable_early_data();
        }
        
        // Set congestion control algorithm
        use crate::protocols::strategy::CongestionControl;
        match self.config.congestion_control {
            CongestionControl::Cubic => config.set_cc_algorithm(quiche::CongestionControlAlgorithm::CUBIC),
            CongestionControl::Reno => config.set_cc_algorithm(quiche::CongestionControlAlgorithm::Reno),
            CongestionControl::Bbr => config.set_cc_algorithm(quiche::CongestionControlAlgorithm::BBR),
            CongestionControl::BbrV2 => config.set_cc_algorithm(quiche::CongestionControlAlgorithm::BBR2),
        }
        
        // Set HTTP/3 application protocol
        if let Err(e) = config.set_application_protos(&[b"h3"]) {
            tracing::error!(
                target: "quyc::protocols::h3",
                error = %e,
                "Failed to set H3 application protocols"
            );
            return Err(crate::error::HttpError::new(crate::error::Kind::Request)
                .with(std::io::Error::new(std::io::ErrorKind::Other, 
                    format!("Critical H3 protocol configuration failure: {}", e))));
        }
        
        // SECURITY: Enable certificate verification using TlsManager infrastructure
        config.verify_peer(true);
        
        // Integrate with existing TLS infrastructure - QUICHE has its own certificate loading
        // Since QUICHE uses its own TLS backend (BoringSSL), we cannot directly integrate 
        // with rustls-based TlsManager. Instead, we let QUICHE use its default CA bundle
        // which provides the same security as the TlsManager approach.
        
        tracing::debug!("HTTP/3 using QUICHE default certificate verification (BoringSSL CA bundle)");
        
        // Note: QUICHE automatically uses system CA certificates through BoringSSL.
        // This provides equivalent security to the TlsManager approach but uses
        // a different TLS backend optimized for QUIC performance.
        
        Ok(config)
    }

    /// Get the next connection ID
    pub(crate) fn next_connection_id() -> u64 {
        NEXT_CONNECTION_ID.fetch_add(1, Ordering::SeqCst)
    }
}

impl ProtocolStrategy for H3Strategy {
    fn execute(&self, request: HttpRequest) -> HttpResponse {
        // Create response streams
        let (headers_tx, headers_internal) = AsyncStream::<HttpHeader, 256>::channel();
        let (body_tx, body_internal) = AsyncStream::<HttpBodyChunk, 1024>::channel();
        let (trailers_tx, trailers_internal) = AsyncStream::<HttpHeader, 64>::channel();
        
        // Extract request details for task
        let method = request.method().clone();
        let url = request.url().clone();
        let headers = request.headers().clone();
        let body_data = request.body().cloned();
        
        // Parse URL components
        let host = url.host_str().unwrap_or("localhost").to_string();
        let port = url.port().unwrap_or(443);
        let path = match url.query() {
            Some(query) => format!("{}?{}", url.path(), query),
            None => url.path().to_string(),
        };
        let scheme = url.scheme().to_string();
        
        // Clone config for async task
        let config = self.config.clone();
        let quic_config = match self.create_quiche_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                // Return error response instead of panicking
                return HttpResponse::error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
            }
        };
        
        // Spawn task to handle H3 protocol
        spawn_task(move || {
            // Create H3 connection manager
            let connection = H3Connection::new(config.clone(), quic_config);
            
            // Establish connection to server
            let (mut quic_conn, mut h3_conn, socket, server_addr, local_addr) = match connection.establish(
                &host, 
                port, 
                &body_tx
            ) {
                Ok(conn_data) => conn_data,
                Err(_) => return, // Error already emitted to stream
            };
            
            // Create request processor
            let processor = H3RequestProcessor::new();
            
            // Send request and process response
            processor.process_request(
                &mut quic_conn,
                &mut h3_conn,
                &socket,
                server_addr,
                local_addr,
                method,
                scheme,
                host,
                path,
                headers,
                body_data,
                config,
                headers_tx,
                body_tx,
                trailers_tx,
            );
        });
        
        // Create and return HttpResponse
        let response = HttpResponse::new(
            headers_internal,
            body_internal,
            trailers_internal,
            Version::HTTP_3,
            0, // stream_id
        );
        
        // Set initial status
        response.set_status(StatusCode::OK);
        
        response
    }
    
    fn protocol_name(&self) -> &'static str {
        "HTTP/3"
    }
    
    fn supports_push(&self) -> bool {
        false // HTTP/3 doesn't use server push like HTTP/2
    }
    
    fn max_concurrent_streams(&self) -> usize {
        self.config.initial_max_streams_bidi as usize
    }
}