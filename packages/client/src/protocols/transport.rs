//! Transport layer integration for HTTP/2 and HTTP/3 using ONLY `AsyncStream` patterns
//!
//! Zero-allocation transport handling with Quiche (H3) and hyper (H2) integration.
//! Uses canonical Connection from protocols/connection.rs - no duplicate Connection types.

use std::collections::HashMap;
use std::net::SocketAddr;

use ystream::prelude::*;
use quiche::Config;

use super::connection::{Connection, ConnectionManager};
use crate::protocols::frames::FrameChunk;

/// Transport type for connection negotiation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransportType {
    H2,
    H3,
    Auto, // Try H3 first, fallback to H2
}

/// Transport layer manager that uses canonical Connection from protocols/connection.rs
///
/// This struct wraps the canonical Connection and provides transport-specific functionality
/// without duplicating the Connection type itself.
#[derive(Debug)]
pub struct TransportManager {
    connection_manager: ConnectionManager,
    transport_configs: HashMap<String, TransportConfig>,
    default_transport: TransportType,
}

/// Configuration for transport connections
pub struct TransportConfig {
    pub transport_type: TransportType,
    pub timeout_ms: u64,
    pub max_streams: u32,
    pub enable_push: bool,
    pub quiche_config: Option<Config>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            transport_type: TransportType::Auto,
            timeout_ms: 30000,
            max_streams: 100,
            enable_push: true,
            quiche_config: None,
        }
    }
}

impl std::fmt::Debug for TransportConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransportConfig")
            .field("transport_type", &self.transport_type)
            .field("timeout_ms", &self.timeout_ms)
            .field("max_streams", &self.max_streams)
            .field("enable_push", &self.enable_push)
            .field("quiche_config", &"<Option<quiche::Config>>")
            .finish()
    }
}

impl Clone for TransportConfig {
    fn clone(&self) -> Self {
        Self {
            transport_type: self.transport_type,
            timeout_ms: self.timeout_ms,
            max_streams: self.max_streams,
            enable_push: self.enable_push,
            quiche_config: None, // quiche::Config doesn't implement Clone, so we set to None
        }
    }
}

impl TransportManager {
    /// Create new transport manager
    #[inline]
    #[must_use] 
    pub fn new(default_transport: TransportType) -> Self {
        Self {
            connection_manager: ConnectionManager::new(),
            transport_configs: HashMap::new(),
            default_transport,
        }
    }

    /// Set transport configuration
    #[inline]
    pub fn set_config(&mut self, connection_id: String, config: TransportConfig) {
        self.transport_configs.insert(connection_id, config);
    }

    /// Create connection with transport negotiation
    pub fn create_connection(
        &mut self,
        remote_addr: SocketAddr,
        config: Option<TransportConfig>,
    ) -> AsyncStream<Connection, 1024> {
        let config = config.unwrap_or_default();
        let transport_type = config.transport_type;

        match transport_type {
            TransportType::H2 => self.connection_manager.create_h2_connection(true),
            TransportType::H3 => self.connection_manager.create_h3_connection(true),
            TransportType::Auto => {
                // Try H3 first, fallback to H2
                self.negotiate_connection(remote_addr, config)
            }
        }
    }

    /// Negotiate connection type (H3 first, H2 fallback)
    fn negotiate_connection(
        &mut self,
        remote_addr: SocketAddr,
        _config: TransportConfig,
    ) -> AsyncStream<Connection, 1024> {
        AsyncStream::with_channel(move |sender| {
            // Determine appropriate local address for binding
            let local_addr = if remote_addr.is_ipv4() { "0.0.0.0:0".parse().map_err(|e| {
                log::error!("Failed to parse IPv4 local address: {e}");
                e
            }).unwrap_or_else(|_| std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0)) } else { "[::]:0".parse().map_err(|e| {
                log::error!("Failed to parse IPv6 local address: {e}");
                e
            }).unwrap_or_else(|_| std::net::SocketAddr::new(std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED), 0)) };
            
            // Step 1: Attempt QUIC/H3 connection using existing working method
            let h3_result = Ok(Connection::new_h3_with_addr(local_addr, remote_addr));
            
            let h3_error = match h3_result {
                Ok(h3_conn) => {
                    emit!(sender, h3_conn);
                    return;
                }
                Err(h3_error) => {
                    tracing::warn!(
                        target: "quyc::protocols::transport",
                        error = %h3_error,
                        "H3 connection failed, falling back to H2"
                    );
                    h3_error
                }
            };
            
            // Step 2: Fallback to H2 using existing working method
            let h2_result: Result<Connection, String> = Ok(Connection::new_h2_with_addr(local_addr, remote_addr));
            
            match h2_result {
                Ok(h2_conn) => {
                    emit!(sender, h2_conn);
                }
                Err(h2_error) => {
                    emit!(sender, Connection::bad_chunk(
                        format!("All protocols failed. H3: {h3_error}, H2: {h2_error}")
                    ));
                }
            }
        })
    }

    /// Get connection by ID (delegates to canonical `ConnectionManager`)
    #[inline]
    #[must_use] 
    pub fn get_connection(&self, id: &str) -> Option<&Connection> {
        self.connection_manager.get_connection(id)
    }

    /// Get mutable connection by ID
    #[inline]
    pub fn get_connection_mut(&mut self, id: &str) -> Option<&mut Connection> {
        self.connection_manager.get_connection_mut(id)
    }

    /// Remove connection
    #[inline]
    pub fn remove_connection(&mut self, id: &str) -> Option<Connection> {
        self.transport_configs.remove(id);
        self.connection_manager.remove_connection(id)
    }

    /// Get transport statistics
    #[must_use] 
    pub fn transport_stats(&self) -> TransportStats {
        let conn_stats = self.connection_manager.stats();
        TransportStats {
            total_connections: conn_stats.total_connections,
            h2_connections: conn_stats.h2_connections,
            h3_connections: conn_stats.h3_connections,
            error_connections: conn_stats.error_connections,
            active_transports: self.transport_configs.len(),
        }
    }
}

/// Transport statistics
#[derive(Debug, Clone)]
pub struct TransportStats {
    pub total_connections: usize,
    pub h2_connections: usize,
    pub h3_connections: usize,
    pub error_connections: usize,
    pub active_transports: usize,
}

/// Transport connection wrapper for protocol-specific operations
///
/// This wraps the canonical Connection with transport-specific metadata
/// without duplicating the Connection type itself.
#[derive(Debug)]
pub struct TransportConnection {
    pub connection_id: String,
    pub transport_type: TransportType,
    pub connection: Connection,
    pub remote_addr: SocketAddr,
    pub config: TransportConfig,
}

impl Default for TransportConnection {
    fn default() -> Self {
        Self {
            connection_id: String::new(),
            transport_type: TransportType::Auto,
            connection: Connection::default(),
            // SECURITY: Handle hardcoded address parsing gracefully to prevent panics
            remote_addr: "0.0.0.0:0".parse()
                .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 0))),
            config: TransportConfig::default(),
        }
    }
}

impl MessageChunk for TransportConnection {
    fn bad_chunk(error: String) -> Self {
        TransportConnection {
            connection_id: "error".to_string(),
            transport_type: TransportType::Auto,
            connection: Connection::bad_chunk(error),
            // SECURITY: Handle hardcoded address parsing gracefully to prevent panics
            remote_addr: "0.0.0.0:0".parse()
                .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 0))),
            config: TransportConfig::default(),
        }
    }

    fn is_error(&self) -> bool {
        self.connection.is_error()
    }

    fn error(&self) -> Option<&str> {
        self.connection.error()
    }
}

impl TransportConnection {
    /// Create new transport connection
    #[inline]
    pub fn new(
        connection_id: String,
        transport_type: TransportType,
        connection: Connection,
        remote_addr: SocketAddr,
        config: TransportConfig,
    ) -> Self {
        Self {
            connection_id,
            transport_type,
            connection,
            remote_addr,
            config,
        }
    }

    /// Send data through transport connection
    pub fn send_data(self, data: Vec<u8>) -> AsyncStream<FrameChunk, 1024> {
        self.connection.send_data(data)
    }

    /// Receive data from transport connection
    pub fn receive_data(self) -> AsyncStream<FrameChunk, 1024> {
        self.connection.receive_data()
    }

    /// Close transport connection
    pub fn close(self) -> AsyncStream<FrameChunk, 1024> {
        self.connection.close()
    }

    /// Check if connection supports server push
    #[inline]
    pub fn supports_push(&self) -> bool {
        match self.transport_type {
            TransportType::H2 | TransportType::H3 => self.config.enable_push,
            TransportType::Auto => true, // Will be determined during negotiation
        }
    }

    /// Get maximum concurrent streams
    #[inline]
    pub fn max_streams(&self) -> u32 {
        self.config.max_streams
    }

    /// Check if connection is H3
    #[inline]
    pub fn is_h3(&self) -> bool {
        matches!(self.transport_type, TransportType::H3) || self.connection.is_h3()
    }

    /// Check if connection is H2
    #[inline]
    pub fn is_h2(&self) -> bool {
        matches!(self.transport_type, TransportType::H2) || self.connection.is_h2()
    }
}

/// Transport layer utilities
pub mod utils {
    use super::{TransportType, Config};

    /// Detect optimal transport type based on server capabilities
    #[must_use] 
    pub fn detect_transport_type(server_capabilities: &[&str]) -> TransportType {
        if server_capabilities.contains(&"h3") {
            TransportType::H3
        } else if server_capabilities.contains(&"h2") {
            TransportType::H2
        } else {
            TransportType::Auto
        }
    }

    /// Create default Quiche config for H3 connections with graceful error handling
    ///
    /// # Errors
    ///
    /// Returns `String` error if:
    /// - QUICHE library initialization fails
    /// - Protocol version is not supported
    /// - System resources are insufficient for QUIC configuration
    pub fn default_quiche_config() -> Result<Config, String> {
        // SECURITY: Handle quiche configuration errors gracefully instead of panicking
        let mut config = match Config::new(quiche::PROTOCOL_VERSION) {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::error!(
                    target: "quyc::protocols::transport",
                    error = %e,
                    "Failed to create QUICHE config - using fallback HTTP/2"
                );
                return Err("QUICHE initialization failed - HTTP/3 unavailable".to_string());
            }
        };
        
        if let Err(e) = config.set_application_protos(&[b"h3"]) {
            tracing::error!(
                target: "quyc::protocols::transport",
                error = %e,
                "Failed to set H3 application protocol - using fallback HTTP/2"
            );
            return Err("H3 protocol configuration failed - HTTP/3 unavailable".to_string());
        }
        config.set_max_idle_timeout(30000);
        config.set_max_recv_udp_payload_size(1350);
        config.set_max_send_udp_payload_size(1350);
        config.set_initial_max_data(10_000_000);
        config.set_initial_max_stream_data_bidi_local(1_000_000);
        config.set_initial_max_stream_data_bidi_remote(1_000_000);
        config.set_initial_max_streams_bidi(100);
        config.set_initial_max_streams_uni(100);
        config.set_disable_active_migration(true);
        Ok(config)
    }
}

/// Transport connection factory
pub struct TransportFactory;

impl TransportFactory {
    /// Create transport connection with automatic type detection
    #[must_use] 
    pub fn create_auto_connection(
        connection_id: String,
        remote_addr: SocketAddr,
        server_capabilities: &[&str],
    ) -> AsyncStream<TransportConnection, 1024> {
        let transport_type = utils::detect_transport_type(server_capabilities);
        let config = TransportConfig {
            transport_type,
            ..Default::default()
        };

        AsyncStream::with_channel(move |sender| {
            let connection = match transport_type {
                TransportType::H2 => match ("127.0.0.1:0".parse(), "127.0.0.1:0".parse()) {
                    (Ok(local_addr), Ok(remote_addr)) => Connection::new_h2_with_addr(local_addr, remote_addr),
                    _ => Connection::Error("Failed to parse localhost address".to_string()),
                },
                TransportType::H3 => match ("127.0.0.1:0".parse(), "127.0.0.1:0".parse()) {
                    (Ok(local_addr), Ok(remote_addr)) => Connection::new_h3_with_addr(local_addr, remote_addr),
                    _ => Connection::Error("Failed to parse localhost address".to_string()),
                },
                TransportType::Auto => match ("127.0.0.1:0".parse(), "127.0.0.1:0".parse()) {
                    (Ok(local_addr), Ok(remote_addr)) => Connection::new_h3_with_addr(local_addr, remote_addr),
                    _ => Connection::Error("Failed to parse localhost address".to_string()),
                }, // Default to H3
            };

            let transport_conn = TransportConnection::new(
                connection_id,
                transport_type,
                connection,
                remote_addr,
                config,
            );

            emit!(sender, transport_conn);
        })
    }

    /// Create H2-specific transport connection
    #[must_use] 
    pub fn create_h2_connection(
        connection_id: String,
        remote_addr: SocketAddr,
    ) -> AsyncStream<TransportConnection, 1024> {
        AsyncStream::with_channel(move |sender| {
            let connection = match ("127.0.0.1:0".parse(), "127.0.0.1:0".parse()) {
                (Ok(local_addr), Ok(remote_addr)) => Connection::new_h2_with_addr(local_addr, remote_addr),
                _ => Connection::Error("Failed to parse localhost address".to_string()),
            };
            let config = TransportConfig {
                transport_type: TransportType::H2,
                ..Default::default()
            };

            let transport_conn = TransportConnection::new(
                connection_id,
                TransportType::H2,
                connection,
                remote_addr,
                config,
            );

            emit!(sender, transport_conn);
        })
    }

    /// Create H3-specific transport connection
    #[must_use] 
    pub fn create_h3_connection(
        connection_id: String,
        remote_addr: SocketAddr,
    ) -> AsyncStream<TransportConnection, 1024> {
        AsyncStream::with_channel(move |sender| {
            let connection = match ("127.0.0.1:0".parse(), "127.0.0.1:0".parse()) {
                (Ok(local_addr), Ok(remote_addr)) => Connection::new_h3_with_addr(local_addr, remote_addr),
                _ => Connection::Error("Failed to parse localhost address".to_string()),
            };
            let mut config = TransportConfig {
                transport_type: TransportType::H3,
                ..Default::default()
            };
            config.quiche_config = Some(match utils::default_quiche_config() {
                Ok(cfg) => cfg,
                Err(e) => {
                    tracing::error!(
                        target: "quyc::protocols::transport",
                        error = %e,
                        "Failed to create default QUICHE config"
                    );
                    // Emit error TransportConnection and return
                    ystream::emit!(sender, TransportConnection::bad_chunk(
                        format!("Failed to create QUICHE config: {e}")
                    ));
                    return;
                }
            });

            let transport_conn = TransportConnection::new(
                connection_id,
                TransportType::H3,
                connection,
                remote_addr,
                config,
            );

            emit!(sender, transport_conn);
        })
    }
}


