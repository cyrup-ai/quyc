//! TLS connection establishment utilities
//!
//! Provides TLS connection establishment for both native-tls and rustls
//! with proper certificate validation and secure connection setup.

use std::net::TcpStream;
use std::time::Duration;

use http::Uri;

// native-tls import removed - using rustls universally

use super::basic_connection::connect_to_address_list;
use super::dns_resolution::resolve_host_sync;

/// Establish HTTP connection using `HttpConnector`.
pub fn establish_http_connection(
    _connector: &hyper_util::client::legacy::connect::HttpConnector,
    uri: &Uri,
    timeout: Option<Duration>,
) -> Result<TcpStream, String> {
    let host = uri.host().ok_or("URI missing host")?;
    let port = uri.port_u16().unwrap_or_else(|| match uri.scheme_str() {
        Some("https") => 443,
        Some("http") => 80,
        _ => 80,
    });

    let addresses = resolve_host_sync(host, port)?;
    connect_to_address_list(&addresses, timeout)
}

// native-TLS function removed - using rustls universally through TlsManager

/// Establish TLS connection using rustls.
#[cfg(feature = "__rustls")]
pub fn establish_rustls_connection(
    stream: TcpStream,
    host: String,
    config: std::sync::Arc<rustls::ClientConfig>,
) -> Result<rustls::StreamOwned<rustls::ClientConnection, TcpStream>, String> {
    let server_name = match rustls::pki_types::DnsName::try_from(host.clone()) {
        Ok(dns_name) => rustls::pki_types::ServerName::DnsName(dns_name),
        Err(e) => return Err(format!("Invalid server name {host}: {e}")),
    };

    let client = rustls::ClientConnection::new(config, server_name)
        .map_err(|e| format!("Failed to create TLS connection: {e}"))?;

    Ok(rustls::StreamOwned::new(client, stream))
}
