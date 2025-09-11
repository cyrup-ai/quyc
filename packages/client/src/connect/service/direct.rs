//! Direct TCP connection establishment with DNS resolution
//!
//! Handles direct connection establishment with zero-allocation streaming,
//! DNS resolution, and connection timeout management.

use std::net::TcpStream;

use ystream::prelude::MessageChunk;
use ystream::{AsyncStream, emit, spawn_task};
use http::Uri;

use super::super::chunks::TcpConnectionChunk;
use super::core::ConnectorService;

/// Extract local and remote addresses from stream and emit connection event
/// Preserves connection attempt ordering for IPv4/IPv6 happy eyeballs logic
pub(super) fn emit_stream_connection(
    stream: TcpStream,
    sender: &ystream::AsyncStreamSender<TcpConnectionChunk>,
) -> Result<(), String> {
    let local_addr = stream.local_addr()
        .map_err(|e| format!("Failed to get local address: {e}"))?;
    let remote_addr = stream.peer_addr()
        .map_err(|e| format!("Failed to get remote address: {e}"))?;
    
    // Set stream to non-blocking mode before converting to Tokio stream
    stream.set_nonblocking(true)
        .map_err(|e| format!("Failed to set stream to non-blocking: {e}"))?;
    
    // Convert std::net::TcpStream to tokio::net::TcpStream and wrap in ConnectionTrait
    // We need to do this within a Tokio runtime context
    let tokio_stream = tokio::task::block_in_place(|| {
        // Try to use current runtime handle if available
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(async {
                tokio::net::TcpStream::from_std(stream)
                    .map_err(|e| format!("Failed to convert TcpStream: {e}"))
            })
        } else {
            // If no runtime exists, create a temporary one just for this operation
            let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {e}"))?;
            rt.block_on(async {
                tokio::net::TcpStream::from_std(stream)
                    .map_err(|e| format!("Failed to convert TcpStream: {e}"))
            })
        }
    })?;
    let conn_trait: Box<dyn crate::connect::types::ConnectionTrait + Send> = 
        Box::new(crate::connect::types::TcpConnection::new(tokio_stream));
    
    let connection_chunk = TcpConnectionChunk::connected(local_addr, remote_addr, Some(conn_trait));
    if sender.send(connection_chunk).is_err() {
        return Err("Failed to send connection chunk".to_string());
    }
    Ok(())
}

impl ConnectorService {
    /// Connect with optional proxy handling
    pub fn connect_with_maybe_proxy(
        &self,
        dst: &Uri,
        _via_proxy: bool,
    ) -> AsyncStream<TcpConnectionChunk, 1024> {
        let connector_service = self.clone();
        let destination = dst.clone();

        AsyncStream::with_channel(move |sender| {
            spawn_task(move || {
                let host = if let Some(h) = destination.host() { h } else {
                    let () = emit!(
                        sender,
                        TcpConnectionChunk::bad_chunk("URI missing host".to_string())
                    );
                    return;
                };

                let port =
                    destination
                        .port_u16()
                        .unwrap_or_else(|| match destination.scheme_str() {
                            Some("https") => 443,
                            Some("http") => 80,
                            _ => 80,
                        });

                // Resolve addresses with zero allocation
                let addresses = match super::super::tcp::resolve_host_sync(host, port) {
                    Ok(addrs) => addrs,
                    Err(e) => {
                        let () = emit!(
                            sender,
                            TcpConnectionChunk::bad_chunk(format!("DNS resolution failed: {e}"))
                        );
                        return;
                    }
                };

                if addresses.is_empty() {
                    emit!(
                        sender,
                        TcpConnectionChunk::bad_chunk("No addresses resolved".to_string())
                    );
                    return ;
                }

                // Try connecting to each address with elite polling
                for addr in addresses {
                    match connector_service.connect_timeout {
                        Some(timeout) => {
                            if let Ok(stream) = TcpStream::connect_timeout(&addr, timeout) {
                                // Configure socket for optimal performance
                                if connector_service.nodelay {
                                    let _ = stream.set_nodelay(true);
                                }

                                // Extract addresses and emit connection event
                                if let Err(error) = emit_stream_connection(stream, &sender) {
                                    let () = emit!(sender, TcpConnectionChunk::bad_chunk(error));
                                    return;
                                }
                                return;
                            }
                        }
                        None => if let Ok(stream) = TcpStream::connect(addr) {
                            if connector_service.nodelay {
                                let _ = stream.set_nodelay(true);
                            }

                            // Extract addresses and emit connection event
                            if let Err(error) = emit_stream_connection(stream, &sender) {
                                let () = emit!(sender, TcpConnectionChunk::bad_chunk(error));
                                return;
                            }
                            return;
                        },
                    }
                }

                let () = emit!(
                    sender,
                    TcpConnectionChunk::bad_chunk("Failed to connect to any address".to_string())
                );
            });
        })
    }
}
