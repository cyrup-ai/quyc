//! DNS resolution engine with retry logic
//!
//! This module provides the low-level DNS resolution implementation with timeout and retry handling.

use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::mpsc;
use std::time::Duration;

use ystream::thread_pool::global_executor;
use tracing::warn;

use super::{config::RetryConfig, error::ResolverError};

/// DNS resolution engine with timeout and retry capabilities
#[derive(Debug)]
pub struct ResolutionEngine {
    retry_config: RetryConfig,
}

impl ResolutionEngine {
    #[must_use] 
    pub fn new(retry_config: RetryConfig) -> Self {
        Self { retry_config }
    }

    /// Perform DNS resolution with timeout and retry logic
    pub fn resolve_with_timeout_and_retry(
        &self,
        hostname: &str,
        port: u16,
        timeout: Duration,
        ipv6_preference: bool,
    ) -> Result<Vec<SocketAddr>, ResolverError> {
        let max_retries = self.retry_config.max_retries;
        let mut retry_delay = self.retry_config.initial_delay;

        for attempt in 0..max_retries {
            match Self::resolve_with_std(hostname, port, timeout) {
                Ok(mut addresses) => {
                    if ipv6_preference {
                        // IPv6 preference: prioritize IPv6 addresses in results
                        // This implements Happy Eyeballs-style preference by sorting IPv6 first
                        addresses.sort_by_key(|addr| match addr {
                            SocketAddr::V6(_) => 0, // IPv6 addresses first
                            SocketAddr::V4(_) => 1, // IPv4 addresses second
                        });
                    } else {
                        // IPv4 preference: prioritize IPv4 addresses in results
                        addresses.sort_by_key(|addr| match addr {
                            SocketAddr::V4(_) => 0, // IPv4 addresses first
                            SocketAddr::V6(_) => 1, // IPv6 addresses second
                        });
                    }
                    return Ok(addresses);
                }
                Err(err) => {
                    warn!(
                        "DNS resolution attempt {} failed for {}: {}",
                        attempt + 1,
                        hostname,
                        err
                    );
                    if attempt < max_retries - 1 {
                        // Note: Sleep is acceptable here since we're already in a spawned thread
                        // This prevents overwhelming DNS servers with rapid retries
                        std::thread::sleep(retry_delay);
                        retry_delay *= self.retry_config.backoff_multiplier; // Configurable exponential backoff
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        Err(ResolverError::NoAddresses)
    }

    /// Perform DNS resolution using `std::net`
    fn resolve_with_std(
        hostname: &str,
        port: u16,
        timeout: Duration,
    ) -> Result<Vec<SocketAddr>, ResolverError> {
        let (tx, rx) = mpsc::channel();
        let hostname = hostname.to_string();

        // Use global thread pool instead of spawning new threads
        global_executor().execute(move || {
            let host_port = format!("{hostname}:{port}");
            let result = match host_port.to_socket_addrs() {
                Ok(addrs) => {
                    let addr_vec: Vec<SocketAddr> = addrs.collect();
                    if addr_vec.is_empty() {
                        Err(ResolverError::EmptyResult)
                    } else {
                        Ok(addr_vec)
                    }
                }
                Err(_) => Err(ResolverError::LookupFailed),
            };
            if tx.send(result).is_err() {
                warn!("Failed to send DNS resolution result for {}", hostname);
            }
        });

        match rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => Err(ResolverError::Timeout),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(ResolverError::NoAddresses),
        }
    }
}
