//! H3 Connection Management
//!
//! Handles QUIC connection establishment, UDP socket management, and security validation.

use std::net::{SocketAddr, UdpSocket};

use crossbeam_utils::Backoff;
use ystream::{AsyncStreamSender, prelude::MessageChunk};
use quiche;

use crate::protocols::strategy::H3Config;
use crate::protocols::core::ProtocolConfig;
use crate::http::response::HttpBodyChunk;

use super::security::validate_destination_address;

/// H3 Connection Manager
///
/// Manages QUIC connection establishment and UDP socket operations
pub(crate) struct H3Connection {
    config: H3Config,
    quic_config: quiche::Config,
}

impl H3Connection {
    /// Send error chunk and return error - helper method
    fn send_error_and_return<T>(&self, body_tx: &AsyncStreamSender<HttpBodyChunk>, message: String) -> Result<T, ()> {
        let error_chunk = HttpBodyChunk::bad_chunk(message);
        if let Err(_) = body_tx.send(error_chunk) {
            // Sender closed, continue with error
        }
        Err(())
    }

    /// Create new H3 connection manager
    pub fn new(config: H3Config, quic_config: quiche::Config) -> Self {
        Self {
            config,
            quic_config,
        }
    }

    /// Establish connection to server
    ///
    /// Returns (quic_connection, h3_connection, socket, server_addr, local_addr)
    pub fn establish(
        mut self,
        host: &str,
        port: u16,
        body_tx: &AsyncStreamSender<HttpBodyChunk>,
    ) -> Result<
        (
            quiche::Connection,
            quiche::h3::Connection,
            UdpSocket,
            SocketAddr,
            SocketAddr,
        ),
        ()
    > {
        // Create UDP socket with security considerations
        let socket = self.create_secure_socket(host, body_tx)?;
        
        // Resolve server address
        let server_addr = self.resolve_server_address(host, port, body_tx)?;
        
        // Get local address
        let local_addr = match socket.local_addr() {
            Ok(addr) => addr,
            Err(e) => {
                let error_chunk = HttpBodyChunk::bad_chunk(format!("Failed to get local address: {e}"));
                if let Err(_) = body_tx.send(error_chunk) {
                    // Sender closed, continue with error
                }
                return Err(());
            }
        };
        
        // Establish QUIC connection
        let mut quic_conn = self.establish_quic_connection(host, server_addr, local_addr, &socket, body_tx)?;
        
        // Create HTTP/3 connection
        let h3_conn = self.create_h3_connection(&mut quic_conn, body_tx)?;
        
        Ok((quic_conn, h3_conn, socket, server_addr, local_addr))
    }

    /// Create UDP socket with security considerations
    fn create_secure_socket(
        &self,
        host: &str,
        body_tx: &AsyncStreamSender<HttpBodyChunk>,
    ) -> Result<UdpSocket, ()> {
        // SECURITY: Bind more securely to prevent UDP amplification attacks
        // For outbound HTTP/3 client connections, we should use the system's default interface
        // but not bind to all interfaces indiscriminately
        let bind_addr = if host == "localhost" || host.starts_with("127.") || host == "::1" {
            "127.0.0.1:0" // Local connections use localhost interface
        } else {
            "0.0.0.0:0" // External connections use default interface
        };
        
        let socket = match UdpSocket::bind(bind_addr) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    error = %e,
                    "Failed to bind UDP socket for QUIC connection"
                );
                let error_chunk = HttpBodyChunk::bad_chunk(format!("Failed to bind UDP socket: {e}"));
                if let Err(_) = body_tx.send(error_chunk) {
                    // Sender closed, continue with error
                }
                return Err(());
            }
        };
        
        // Set non-blocking for efficient I/O
        if let Err(e) = socket.set_nonblocking(true) {
            tracing::warn!(
                target: "quyc::protocols::h3",
                error = %e,
                "Failed to set UDP socket to non-blocking mode"
            );
        }
        
        Ok(socket)
    }

    /// Resolve server address with security validation
    fn resolve_server_address(
        &self,
        host: &str,
        port: u16,
        body_tx: &AsyncStreamSender<HttpBodyChunk>,
    ) -> Result<SocketAddr, ()> {
        let server_addr = format!("{}:{}", host, port);
        let server_addr: SocketAddr = match server_addr.parse() {
            Ok(addr) => addr,
            Err(_) => {
                // Try DNS resolution
                match std::net::ToSocketAddrs::to_socket_addrs(&server_addr) {
                    Ok(mut addrs) => {
                        if let Some(addr) = addrs.next() {
                            addr
                        } else {
                            tracing::error!(
                                target: "quyc::protocols::h3",
                                server_addr = %server_addr,
                                "DNS resolution returned no addresses"
                            );
                            let error_chunk = HttpBodyChunk::bad_chunk(format!("Failed to resolve address: {server_addr}"));
                            if let Err(_) = body_tx.send(error_chunk) {
                                // Sender closed, continue with error
                            }
                            return Err(());
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            target: "quyc::protocols::h3",
                            error = %e,
                            server_addr = %server_addr,
                            "Failed to resolve server address via DNS"
                        );
                        let error_chunk = HttpBodyChunk::bad_chunk(format!("Failed to resolve {}: {}", server_addr, e));
                        if let Err(_) = body_tx.send(error_chunk) {
                            // Sender closed, continue with error
                        }
                        return Err(());
                    }
                }
            }
        };
        
        // SECURITY: Validate destination address to prevent UDP amplification attacks
        if let Err(security_error) = validate_destination_address(&server_addr) {
            tracing::error!(
                target: "quyc::protocols::h3",
                addr = %server_addr,
                error = %security_error,
                "Blocked potentially dangerous destination address"
            );
            let error_chunk = HttpBodyChunk::bad_chunk("Connection blocked: destination address validation failed".to_string());
            if let Err(_) = body_tx.send(error_chunk) {
                // Sender closed, continue with error
            }
            return Err(());
        }
        
        Ok(server_addr)
    }

    /// Establish QUIC connection with handshake
    fn establish_quic_connection(
        &mut self,
        host: &str,
        server_addr: SocketAddr,
        local_addr: SocketAddr,
        socket: &UdpSocket,
        body_tx: &AsyncStreamSender<HttpBodyChunk>,
    ) -> Result<quiche::Connection, ()> {
        // Generate connection ID
        let conn_id = super::core::H3Strategy::next_connection_id();
        let conn_id_bytes = conn_id.to_be_bytes();
        let scid = quiche::ConnectionId::from_ref(&conn_id_bytes);
        
        // Create QUIC connection
        let mut quic_conn = match quiche::connect(
            Some(host),
            &scid,
            local_addr,
            server_addr,
            &mut self.quic_config,
        ) {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    error = %e,
                    host = %host,
                    server_addr = %server_addr,
                    "Failed to create QUIC connection"
                );
                return self.send_error_and_return(body_tx, format!("Failed to create QUIC connection: {}", e));
            }
        };
        
        // Perform initial handshake
        self.perform_handshake(&mut quic_conn, socket, body_tx)?;
        
        Ok(quic_conn)
    }

    /// Perform QUIC handshake
    fn perform_handshake(
        &self,
        quic_conn: &mut quiche::Connection,
        socket: &UdpSocket,
        body_tx: &AsyncStreamSender<HttpBodyChunk>,
    ) -> Result<(), ()> {
        // Initial handshake send
        let mut out = [0; 1350];
        let (write_len, send_info) = match quic_conn.send(&mut out) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    error = %e,
                    "Failed initial QUIC handshake send"
                );
                return self.send_error_and_return(body_tx, format!("Failed initial QUIC send: {}", e));
            }
        };
        
        if let Err(e) = socket.send_to(&out[..write_len], send_info.to) {
            tracing::error!(
                target: "quyc::protocols::h3",
                error = %e,
                packet_size = write_len,
                destination = %send_info.to,
                "Failed to send initial QUIC packet"
            );
            return self.send_error_and_return(body_tx, format!("Failed to send initial packet: {}", e));
        }
        
        // Wait for handshake to complete with elite backoff
        let mut buf = [0; 65535];
        let start = std::time::Instant::now();
        let timeout = self.config.timeout_config().connect_timeout;
        let backoff = Backoff::new();
        
        while !quic_conn.is_established() {
            if start.elapsed() > timeout {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    timeout_ms = timeout.as_millis(),
                    elapsed_ms = start.elapsed().as_millis(),
                    "QUIC handshake timeout exceeded"
                );
                return self.send_error_and_return(body_tx, "QUIC handshake timeout".to_string());
            }
            
            let mut data_processed = false;
            
            // Try to receive
            match socket.recv_from(&mut buf) {
                Ok((len, from)) => {
                    let recv_info = quiche::RecvInfo {
                        from,
                        to: match socket.local_addr() {
                            Ok(addr) => addr,
                            Err(e) => {
                                let error_chunk = HttpBodyChunk::bad_chunk(format!("Failed to get local address during handshake: {}", e));
                                if let Err(_) = body_tx.send(error_chunk) {
                                    // Sender closed
                                }
                                return Err(());
                            }
                        },
                    };
                    
                    if let Err(e) = quic_conn.recv(&mut buf[..len], recv_info) {
                        tracing::warn!(
                            target: "quyc::protocols::h3",
                            error = %e,
                            packet_len = len,
                            "QUIC packet receive error during handshake"
                        );
                    } else {
                        data_processed = true;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available, continue
                }
                Err(e) => {
                    tracing::warn!(
                        target: "quyc::protocols::h3",
                        error = %e,
                        "UDP socket receive error during handshake"
                    );
                }
            }
            
            // Send any pending data
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
                                "Failed to send QUIC packet during handshake"
                            );
                        } else {
                            data_processed = true;
                        }
                    }
                    Err(quiche::Error::Done) => break,
                    Err(e) => {
                        tracing::warn!(
                            target: "quyc::protocols::h3",
                            error = %e,
                            "QUIC send error during handshake"
                        );
                        break;
                    }
                }
            }
            
            // Elite backoff pattern - reset on successful data processing
            if data_processed {
                backoff.reset();
            } else {
                // Use only backoff.snooze() - no thread operations
                backoff.snooze();
            }
        }
        
        Ok(())
    }

    /// Create HTTP/3 connection
    fn create_h3_connection(
        &self,
        quic_conn: &mut quiche::Connection,
        body_tx: &AsyncStreamSender<HttpBodyChunk>,
    ) -> Result<quiche::h3::Connection, ()> {
        // Create HTTP/3 configuration
        let h3_config = match quiche::h3::Config::new() {
            Ok(config) => config,
            Err(e) => {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    error = %e,
                    "Failed to create H3 config"
                );
                return self.send_error_and_return(body_tx, format!("Failed to create H3 config: {}", e));
            }
        };
        
        // Create HTTP/3 connection
        let h3_conn = match quiche::h3::Connection::with_transport(quic_conn, &h3_config) {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!(
                    target: "quyc::protocols::h3",
                    error = %e,
                    "Failed to create HTTP/3 connection over QUIC transport"
                );
                return self.send_error_and_return(body_tx, format!("Failed to create H3 connection: {}", e));
            }
        };
        
        Ok(h3_conn)
    }
}