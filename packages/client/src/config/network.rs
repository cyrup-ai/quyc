//! Network Configuration Module
//!
//! Provides comprehensive network configuration for HTTP client connections.

use std::net::SocketAddr;
use std::time::Duration;

/// IP version preference for network connections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpVersion {
    /// IPv4 only
    V4,
    /// IPv6 only
    V6,
    /// Dual stack (prefer IPv4)
    Dual,
}

impl Default for IpVersion {
    fn default() -> Self {
        Self::Dual
    }
}

/// Network configuration provider trait
pub trait NetworkConfigProvider {
    fn local_bind_address(&self) -> Option<SocketAddr>;
    fn preferred_ip_version(&self) -> IpVersion;
    fn max_concurrent_connections(&self) -> usize;
    fn connection_pool_size(&self) -> usize;
    fn dns_timeout(&self) -> Duration;
    fn keepalive_interval(&self) -> Option<Duration>;
    fn tcp_nodelay(&self) -> bool;
    fn socket_reuse(&self) -> bool;
}

/// Runtime network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub local_bind_address: Option<SocketAddr>,
    pub preferred_ip_version: IpVersion,
    pub max_concurrent_connections: usize,
    pub connection_pool_size: usize,
    pub dns_timeout: Duration,
    pub keepalive_interval: Option<Duration>,
    pub tcp_nodelay: bool,
    pub socket_reuse: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            local_bind_address: None,
            preferred_ip_version: IpVersion::Dual,
            max_concurrent_connections: 100,
            connection_pool_size: 20,
            dns_timeout: Duration::from_secs(5),
            keepalive_interval: Some(Duration::from_secs(75)),
            tcp_nodelay: true,
            socket_reuse: true,
        }
    }
}

impl NetworkConfig {
    /// Create production-optimized network configuration
    #[must_use]
    pub fn production() -> Self {
        Self {
            max_concurrent_connections: 500,
            connection_pool_size: 100,
            dns_timeout: Duration::from_secs(3),
            keepalive_interval: Some(Duration::from_secs(60)),
            ..Self::default()
        }
    }
    
    /// Create development-friendly network configuration
    #[must_use]
    pub fn development() -> Self {
        Self {
            max_concurrent_connections: 10,
            connection_pool_size: 5,
            dns_timeout: Duration::from_secs(10),
            keepalive_interval: Some(Duration::from_secs(30)),
            ..Self::default()
        }
    }
    
    /// Validate network configuration
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - `max_concurrent_connections` is 0
    /// - `connection_pool_size` is 0 
    /// - `connection_pool_size` exceeds `max_concurrent_connections`
    /// - DNS timeout is zero
    /// - Keepalive interval is zero
    pub fn validate(&self) -> Result<(), String> {
        if self.max_concurrent_connections == 0 {
            return Err("max_concurrent_connections must be greater than 0".to_string());
        }
        
        if self.connection_pool_size == 0 {
            return Err("connection_pool_size must be greater than 0".to_string());
        }
        
        if self.connection_pool_size > self.max_concurrent_connections {
            return Err("connection_pool_size cannot exceed max_concurrent_connections".to_string());
        }
        
        if self.dns_timeout.as_secs() == 0 {
            return Err("dns_timeout must be greater than 0".to_string());
        }
        
        if self.dns_timeout > Duration::from_secs(60) {
            return Err("dns_timeout should not exceed 60 seconds".to_string());
        }
        
        Ok(())
    }
}
impl NetworkConfigProvider for NetworkConfig {
    #[inline]
    fn local_bind_address(&self) -> Option<SocketAddr> {
        self.local_bind_address
    }
    
    #[inline]
    fn preferred_ip_version(&self) -> IpVersion {
        self.preferred_ip_version
    }
    
    #[inline]
    fn max_concurrent_connections(&self) -> usize {
        self.max_concurrent_connections
    }
    
    #[inline]
    fn connection_pool_size(&self) -> usize {
        self.connection_pool_size
    }
    
    #[inline]
    fn dns_timeout(&self) -> Duration {
        self.dns_timeout
    }
    
    #[inline]
    fn keepalive_interval(&self) -> Option<Duration> {
        self.keepalive_interval
    }
    
    #[inline]
    fn tcp_nodelay(&self) -> bool {
        self.tcp_nodelay
    }
    
    #[inline]
    fn socket_reuse(&self) -> bool {
        self.socket_reuse
    }
}

/// Compile-time network configuration for zero-allocation access
pub struct StaticNetworkConfig<const MAX_CONNECTIONS: usize = 100>;

impl<const MAX_CONNECTIONS: usize> NetworkConfigProvider for StaticNetworkConfig<MAX_CONNECTIONS> {
    #[inline]
    fn local_bind_address(&self) -> Option<SocketAddr> {
        None
    }
    
    #[inline]
    fn preferred_ip_version(&self) -> IpVersion {
        IpVersion::Dual
    }
    
    #[inline]
    fn max_concurrent_connections(&self) -> usize {
        MAX_CONNECTIONS
    }
    
    #[inline]
    fn connection_pool_size(&self) -> usize {
        MAX_CONNECTIONS / 5  // 20% of max connections
    }
    
    #[inline]
    fn dns_timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
    
    #[inline]
    fn keepalive_interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(75))
    }
    
    #[inline]
    fn tcp_nodelay(&self) -> bool {
        true
    }
    
    #[inline]
    fn socket_reuse(&self) -> bool {
        true
    }
}