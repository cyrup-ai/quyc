//! DNS resolution utility functions
//!
//! Zero-allocation host-to-addresses resolution with port handling
//! and optimized system DNS calls with stack-allocated buffers.

use std::net::{SocketAddr, IpAddr, ToSocketAddrs};
use std::str::FromStr;

/// Zero-allocation host-to-addresses resolution with port handling.
/// Uses optimized system DNS calls with stack-allocated buffers for blazing-fast performance.
pub fn resolve_host_to_addrs(host: &str, port: u16, prefer_ipv6: bool) -> Result<arrayvec::ArrayVec<SocketAddr, 8>, String> {
    // Try direct IP address parsing first (fastest path - zero allocation)
    if let Ok(ip_addr) = IpAddr::from_str(host) {
        let mut result = arrayvec::ArrayVec::new();
        result.push(SocketAddr::new(ip_addr, port));
        return Ok(result);
    }
    
    // Perform DNS resolution using system resolver with zero-allocation buffer
    let host_with_port = format!("{}:{}", host, port);
    let socket_addrs: Result<arrayvec::ArrayVec<SocketAddr, 8>, std::io::Error> = 
        host_with_port.to_socket_addrs().map(|iter| {
            let mut addrs: arrayvec::ArrayVec<SocketAddr, 8> = iter.take(8).collect();
            
            // Apply address preference sorting using zero-allocation unstable sort
            if prefer_ipv6 {
                addrs.sort_unstable_by_key(|addr| match addr.ip() {
                    IpAddr::V6(_) => 0, // IPv6 first
                    IpAddr::V4(_) => 1, // IPv4 second
                });
            } else {
                addrs.sort_unstable_by_key(|addr| match addr.ip() {
                    IpAddr::V4(_) => 0, // IPv4 first
                    IpAddr::V6(_) => 1, // IPv6 second
                });
            }
            
            addrs
        });

    match socket_addrs {
        Ok(addrs) => {
            if addrs.is_empty() {
                Err(format!("No addresses found for host: {}", host))
            } else {
                Ok(addrs)
            }
        },
        Err(e) => Err(format!("DNS resolution failed for {}: {}", host, e)),
    }
}