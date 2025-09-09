use std::thread;
use ystream::{AsyncStream, emit};

use super::{Addrs, Name, Resolve, DnsResult};
use crate::prelude::*;

struct GaiAddrs {
    addrs: std::vec::IntoIter<std::net::SocketAddr>,
}

impl Iterator for GaiAddrs {
    type Item = std::net::SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.addrs.next()
    }
}

#[derive(Debug)]
pub struct GaiResolver {
    // Pure implementation without external dependencies
}

impl GaiResolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GaiResolver {
    fn default() -> Self {
        GaiResolver::new()
    }
}

impl Resolve for GaiResolver {
    fn resolve(&self, name: Name) -> AsyncStream<DnsResult, 1024> {
        let hostname = name.as_str().to_string();
        
        AsyncStream::with_channel(move |sender| {
            thread::spawn(move || {
                // Use synchronous DNS resolution via ToSocketAddrs
                let host_port = format!("{}:0", hostname);
                match std::net::ToSocketAddrs::to_socket_addrs(&host_port) {
                    Ok(addrs_iter) => {
                        let socket_addrs: Vec<std::net::SocketAddr> = addrs_iter.collect();
                        if socket_addrs.is_empty() {
                            let error_msg = format!("No addresses found for {}", hostname);
                            emit!(sender, DnsResult::bad_chunk(error_msg));
                        } else {
                            let dns_result = DnsResult::from_vec(socket_addrs);
                            emit!(sender, dns_result);
                        }
                    }
                    Err(e) => {
                        // Always sanitize hostnames in error messages (no config required)
                        let sanitized_hostname = if hostname.contains(".local") || 
                                                   hostname.starts_with("192.168.") ||
                                                   hostname.starts_with("10.") ||
                                                   hostname.starts_with("172.") ||
                                                   hostname == "localhost" {
                            "[INTERNAL_HOST]"
                        } else {
                            hostname
                        };
                        let error_msg = format!("GAI resolution failed for {}: {}", sanitized_hostname, e);
                        emit!(sender, DnsResult::bad_chunk(error_msg));
                    }
                }
            });
        })
    }
}
