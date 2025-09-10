//! DNS resolution and address handling utilities
//!
//! Provides fast DNS resolution with IP address fast-path optimization
//! for optimal network connection establishment.

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;

/// Resolve hostname to socket addresses synchronously with optimal performance.
pub fn resolve_host_sync(host: &str, port: u16) -> Result<Vec<SocketAddr>, String> {
    // Fast path for IP addresses
    if let Ok(ip) = IpAddr::from_str(host) {
        return Ok(vec![SocketAddr::new(ip, port)]);
    }

    // DNS resolution
    let host_port = format!("{host}:{port}");
    match host_port.to_socket_addrs() {
        Ok(addrs) => {
            let addr_vec: Vec<SocketAddr> = addrs.collect();
            if addr_vec.is_empty() {
                Err(format!("No addresses resolved for {host}"))
            } else {
                Ok(addr_vec)
            }
        }
        Err(e) => Err(format!("DNS resolution failed for {host}: {e}")),
    }
}
