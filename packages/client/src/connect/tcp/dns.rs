//! DNS resolution utilities for TCP connections
//!
//! Fast DNS resolution with IP address detection and error handling.
//! Optimized for zero-allocation networking with synchronous resolution.

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;

/// Resolve hostname to socket addresses synchronously with optimal performance.
///
/// This function provides fast-path optimization for IP addresses and handles
/// DNS resolution with comprehensive error reporting.
///
/// # Arguments
/// * `host` - The hostname or IP address to resolve
/// * `port` - The port number to use
///
/// # Returns
/// * `Ok(Vec<SocketAddr>)` - List of resolved socket addresses
/// * `Err(String)` - Error message describing the resolution failure
///
/// # Examples
/// ```rust
/// use quyc::hyper::connect::tcp::dns::resolve_host_sync;
///
/// // Resolve a hostname
/// let addrs = resolve_host_sync("example.com", 80)?;
/// assert!(!addrs.is_empty());
///
/// // Fast path for IP addresses
/// let ip_addrs = resolve_host_sync("127.0.0.1", 8080)?;
/// assert_eq!(ip_addrs.len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn resolve_host_sync(host: &str, port: u16) -> Result<Vec<SocketAddr>, String> {
    // Fast path for IP addresses - avoid DNS lookup
    if let Ok(ip) = IpAddr::from_str(host) {
        return Ok(vec![SocketAddr::new(ip, port)]);
    }

    // DNS resolution for hostnames
    let host_port = format!("{}:{}", host, port);
    match host_port.to_socket_addrs() {
        Ok(addrs) => {
            let addr_vec: Vec<SocketAddr> = addrs.collect();
            if addr_vec.is_empty() {
                Err(format!("No addresses resolved for {}", host))
            } else {
                Ok(addr_vec)
            }
        }
        Err(e) => Err(format!("DNS resolution failed for {}: {}", host, e)),
    }
}

