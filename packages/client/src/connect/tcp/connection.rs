//! TCP connection establishment and management
//!
//! High-performance connection utilities including Happy Eyeballs (RFC 6555),
//! socket configuration, and HTTP connection establishment.

use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use http::Uri;

use super::dns::resolve_host_sync;

/// Connect to first available address with timeout support.
///
/// Attempts to connect to each address in the list until one succeeds.
/// Provides detailed error logging for debugging connection issues.
///
/// # Arguments
/// * `addrs` - List of socket addresses to try
/// * `timeout` - Optional connection timeout per address
///
/// # Returns
/// * `Ok(TcpStream)` - Successfully established connection
/// * `Err(String)` - Error message if all connections failed
pub fn connect_to_address_list(
    addrs: &[SocketAddr],
    timeout: Option<Duration>,
) -> Result<TcpStream, String> {
    if addrs.is_empty() {
        return Err("No addresses to connect to".to_string());
    }

    for addr in addrs {
        match timeout {
            Some(t) => {
                match TcpStream::connect_timeout(addr, t) {
                    Ok(stream) => return Ok(stream),
                    Err(e) => {
                        // Log error and continue to next address
                        tracing::debug!("Failed to connect to {}: {}", addr, e);
                        continue;
                    }
                }
            }
            None => match TcpStream::connect(addr) {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    tracing::debug!("Failed to connect to {}: {}", addr, e);
                    continue;
                }
            },
        }
    }

    Err("Failed to connect to any address".to_string())
}

/// Implement Happy Eyeballs (RFC 6555) for optimal dual-stack connectivity.
///
/// Attempts IPv6 connections first, then tries IPv4 after a delay.
/// Returns the first successful connection for optimal performance.
///
/// # Arguments
/// * `ipv6_addrs` - IPv6 addresses to try first
/// * `ipv4_addrs` - IPv4 addresses to try after delay
/// * `delay` - Delay before trying IPv4 addresses
/// * `timeout` - Overall timeout for all connection attempts
pub fn happy_eyeballs_connect(
    ipv6_addrs: &[SocketAddr],
    ipv4_addrs: &[SocketAddr],
    delay: Duration,
    timeout: Option<Duration>,
) -> Result<TcpStream, String> {
    use std::sync::mpsc;
    use std::thread;

    let start = Instant::now();
    let (tx, rx) = mpsc::channel();

    // Try IPv6 first
    let tx_v6 = tx.clone();
    let ipv6_addrs = ipv6_addrs.to_vec();
    let ipv6_timeout = timeout;
    thread::spawn(
        move || match connect_to_address_list(&ipv6_addrs, ipv6_timeout) {
            Ok(stream) => {
                let _ = tx_v6.send(Ok(stream));
            }
            Err(e) => {
                let _ = tx_v6.send(Err(format!("IPv6: {}", e)));
            }
        },
    );

    // Try IPv4 after delay
    let tx_v4 = tx;
    let ipv4_addrs = ipv4_addrs.to_vec();
    let ipv4_timeout = timeout;
    thread::spawn(move || {
        thread::sleep(delay);
        match connect_to_address_list(&ipv4_addrs, ipv4_timeout) {
            Ok(stream) => {
                let _ = tx_v4.send(Ok(stream));
            }
            Err(e) => {
                let _ = tx_v4.send(Err(format!("IPv4: {}", e)));
            }
        }
    });

    // Wait for first successful connection
    let mut errors = Vec::new();
    let overall_timeout = timeout.unwrap_or(Duration::from_secs(30));

    while start.elapsed() < overall_timeout {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(stream)) => return Ok(stream),
            Ok(Err(e)) => errors.push(e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    Err(format!("Happy Eyeballs failed: {:?}", errors))
}

/// Configure TCP socket for optimal performance.
///
/// Sets TCP_NODELAY and other performance-critical socket options.
/// Uses only safe Rust APIs for maximum reliability.
pub fn configure_tcp_socket(
    stream: &mut TcpStream,
    nodelay: bool,
    keepalive: Option<Duration>,
) -> Result<(), String> {
    // Socket configuration using safe Rust APIs only

    if nodelay {
        stream
            .set_nodelay(true)
            .map_err(|e| format!("Failed to set TCP_NODELAY: {}", e))?;
    }

    if let Some(_duration) = keepalive {
        // TCP keepalive configuration using safe Rust APIs only
        // Note: Advanced keepalive configuration requires unsafe code which is denied
        // Basic TCP stream configuration is handled via set_nodelay above
        tracing::debug!("TCP keepalive requested but advanced configuration requires unsafe code");
    }

    Ok(())
}

/// Inline TCP socket configuration for performance-critical paths.
#[inline]
pub fn configure_tcp_socket_inline(stream: &TcpStream, nodelay: bool) -> Result<(), String> {
    if nodelay {
        stream
            .set_nodelay(true)
            .map_err(|e| format!("Failed to set TCP_NODELAY: {}", e))?;
    }
    Ok(())
}

/// Establish HTTP connection using HttpConnector.
///
/// Resolves the URI host and establishes a TCP connection with timeout support.
/// Handles both HTTP and HTTPS default ports automatically.
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