//! Proxy connection establishment with SOCKS5 and HTTP CONNECT support
//!
//! Handles proxy connection establishment including SOCKS5 handshake,
//! HTTP CONNECT tunneling, and proxy authentication.

use std::net::SocketAddr;

use ystream::prelude::MessageChunk;
use ystream::{AsyncStream, emit, spawn_task};
use http::Uri;

use super::super::chunks::TcpConnectionChunk;
use super::core::ConnectorService;

impl ConnectorService {
    /// Connect via proxy with full SOCKS and HTTP CONNECT support
    pub fn connect_via_proxy(&self, dst: &Uri, proxy_scheme: &str) -> AsyncStream<TcpConnectionChunk, 1024> {
        let connector_service = self.clone();
        let destination = dst.clone();
        let scheme = proxy_scheme.to_string();

        AsyncStream::with_channel(move |sender| {
            spawn_task(move || {
                let Some(proxy_config) = connector_service.intercepted.first_proxy() else {
                    emit!(
                        sender,
                        TcpConnectionChunk::bad_chunk(
                            "No proxy configuration available".to_string()
                        )
                    );
                    return;
                };

                let proxy_uri = &proxy_config.uri;
                let Some(proxy_host) = proxy_uri.host() else {
                    emit!(
                        sender,
                        TcpConnectionChunk::bad_chunk("Proxy URI missing host".to_string())
                    );
                    return;
                };

                let proxy_port = proxy_uri.port_u16().unwrap_or(8080);

                // Connect to proxy server first
                let proxy_stream = match super::super::tcp::connect_to_address_list(
                    &[SocketAddr::new(
                        proxy_host.parse().unwrap_or({
                            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
                        }),
                        proxy_port,
                    )],
                    connector_service.connect_timeout,
                ) {
                    Ok(stream) => stream,
                    Err(e) => {
                        emit!(
                            sender,
                            TcpConnectionChunk::bad_chunk(format!("Proxy connection failed: {e}"))
                        );
                        return;
                    }
                };

                // Handle different proxy types
                let final_stream = match scheme.as_str() {
                    "http" | "https" => {
                        // HTTP CONNECT tunnel
                        match super::super::tcp::establish_connect_tunnel(
                            proxy_stream,
                            &destination,
                            proxy_config.basic_auth.as_deref(),
                        ) {
                            Ok(stream) => stream,
                            Err(e) => {
                                emit!(
                                    sender,
                                    TcpConnectionChunk::bad_chunk(format!(
                                        "CONNECT tunnel failed: {e}"
                                    ))
                                );
                                return;
                            }
                        }
                    }
                    "socks5" => {
                        // SOCKS5 proxy
                        let target_host = destination.host().unwrap_or("localhost");
                        let target_port = destination.port_u16().unwrap_or(80);

                        match super::super::tcp::socks5_handshake(
                            proxy_stream,
                            target_host,
                            target_port,
                        ) {
                            Ok(stream) => stream,
                            Err(e) => {
                                emit!(
                                    sender,
                                    TcpConnectionChunk::bad_chunk(format!(
                                        "SOCKS5 handshake failed: {e}"
                                    ))
                                );
                                return;
                            }
                        }
                    }
                    _ => {
                        emit!(
                            sender,
                            TcpConnectionChunk::bad_chunk(format!(
                                "Unsupported proxy scheme: {scheme}"
                            ))
                        );
                        return;
                    }
                };

                // Configure final stream
                if connector_service.nodelay {
                    let _ = final_stream.set_nodelay(true);
                }

                // Extract addresses and emit connection event
                if let Err(error) = super::direct::emit_stream_connection(final_stream, &sender) {
                    emit!(sender, TcpConnectionChunk::bad_chunk(error));
                }
            });
        })
    }
}
