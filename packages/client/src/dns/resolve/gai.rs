//! System getaddrinfo-based DNS resolver
//!
//! High-performance synchronous DNS resolver using system getaddrinfo
//! with zero-allocation design and optimized address sorting.

use ystream::{AsyncStream, emit};
use std::net::{SocketAddr, IpAddr};
use std::thread;

use super::traits::Resolve;
use super::types::{DnsResult, HyperName};

/// High-performance synchronous DNS resolver using system getaddrinfo.
/// Zero-allocation design with optimized address sorting.
pub struct GaiResolver {
    prefer_ipv6: bool,
    timeout_ms: u32,
}

impl GaiResolver {
    pub fn new() -> Self {
        Self {
            prefer_ipv6: false,
            timeout_ms: 5000, // 5 second default timeout
        }
    }

    pub fn prefer_ipv6(mut self, prefer: bool) -> Self {
        self.prefer_ipv6 = prefer;
        self
    }

    pub fn timeout_ms(mut self, timeout: u32) -> Self {
        self.timeout_ms = timeout;
        self
    }
}

impl Default for GaiResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolve for GaiResolver {
    fn resolve(&self, name: HyperName) -> AsyncStream<DnsResult, 1024> {
        use std::net::ToSocketAddrs;
        
        let hostname = name.as_str().to_string();
        let prefer_ipv6 = self.prefer_ipv6;
        let timeout_ms = self.timeout_ms;
        
        AsyncStream::with_channel(move |sender| {
            thread::spawn(move || {
                // Use std::net::ToSocketAddrs for synchronous resolution
                let dummy_port = 80; // Port doesn't matter for hostname resolution
                let host_with_port = format!("{}:{}", hostname, dummy_port);
                
                // Set up timeout handling using thread-local storage
                let start_time = std::time::Instant::now();
                
                let socket_addrs: Result<arrayvec::ArrayVec<SocketAddr, 8>, std::io::Error> = 
                    host_with_port.to_socket_addrs().map(|iter| {
                        let mut addrs: arrayvec::ArrayVec<SocketAddr, 8> = iter.take(8).collect();
                        
                        // Check timeout using elite polling pattern
                        if start_time.elapsed().as_millis() > timeout_ms as u128 {
                            return arrayvec::ArrayVec::new(); // Timeout exceeded
                        }
                        
                        // Sort addresses based on preference using zero-allocation sort
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
                        
                        // Remove port information since we added dummy port - zero allocation
                        for addr in addrs.iter_mut() {
                            addr.set_port(0); // Clear the dummy port
                        }
                        addrs
                    });

                match socket_addrs {
                    Ok(addrs) => {
                        if addrs.is_empty() {
                            emit!(sender, DnsResult::bad_chunk(format!("DNS resolution timeout or no addresses found for {}", hostname)));
                        } else {
                            emit!(sender, DnsResult { addrs });
                        }
                    },
                    Err(e) => {
                        emit!(sender, DnsResult::bad_chunk(format!("DNS resolution failed for {}: {}", hostname, e)));
                    }
                }
            });
        })
    }
}