//! TLS connection establishment utilities
//!
//! Support for rustls TLS implementation with comprehensive error handling and type safety.
//! Native-TLS support removed - using rustls universally.

use std::net::TcpStream;

// Native-TLS function removed - using rustls universally through TlsManager

/// Establish TLS connection using rustls.
///
/// Creates a secure TLS connection over an existing TCP stream using the rustls crate.
/// Provides modern TLS implementation with comprehensive security features.
///
/// # Arguments
/// * `stream` - The underlying TCP stream
/// * `host` - The hostname for TLS verification
/// * `config` - The rustls client configuration
///
/// # Returns
/// * `Ok(StreamOwned)` - Successfully established TLS connection
/// * `Err(String)` - Error message if TLS handshake failed
#[cfg(feature = "__rustls")]
pub fn establish_rustls_connection(
    stream: TcpStream,
    host: String,
    config: std::sync::Arc<rustls::ClientConfig>,
) -> Result<rustls::StreamOwned<rustls::ClientConnection, TcpStream>, String> {
    let server_name = match rustls::pki_types::DnsName::try_from(host.clone()) {
        Ok(dns_name) => rustls::pki_types::ServerName::DnsName(dns_name),
        Err(e) => return Err(format!("Invalid server name {}: {}", host, e)),
    };

    let client = rustls::ClientConnection::new(config, server_name)
        .map_err(|e| format!("Failed to create TLS connection: {}", e))?;

    Ok(rustls::StreamOwned::new(client, stream))
}

