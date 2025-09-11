//! Core Configuration Utilities
//!
//! Common configuration patterns, validation, and utilities.

use std::time::Duration;
use std::net::SocketAddr;

/// Configuration validation result type
pub type ConfigResult<T> = Result<T, ConfigurationError>;

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("Invalid timeout value: {0}")]
    InvalidTimeout(String),
    
    #[error("Invalid network address: {0}")]
    InvalidAddress(String),
    
    #[error("Invalid buffer size: {0}")]
    InvalidBufferSize(String),
    
    #[error("Invalid configuration parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Configuration conflict: {0}")]
    Conflict(String),
}

/// Configuration validation trait
pub trait Validator {
    /// Validates the configuration settings
    /// 
    /// # Errors
    /// 
    /// Returns a `ConfigurationError` variant if any validation fails:
    /// - `InvalidTimeout` - if timeout values are zero or exceed limits
    /// - `InvalidAddress` - if network addresses are malformed or invalid
    /// - `InvalidBufferSize` - if buffer sizes are zero or exceed limits  
    /// - `InvalidParameter` - if parameters are outside valid ranges
    /// - `Conflict` - if configuration settings conflict with each other
    fn validate(&self) -> ConfigResult<()>;
}

/// Common configuration validation utilities
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate timeout duration
    /// 
    /// # Errors
    /// 
    /// Returns `ConfigurationError::InvalidTimeout` if:
    /// - The timeout duration is zero
    /// - The timeout duration exceeds 1 hour (3600 seconds)
    pub fn validate_timeout(timeout: Duration, name: &str) -> ConfigResult<()> {
        if timeout.is_zero() {
            return Err(ConfigurationError::InvalidTimeout(
                format!("{name} cannot be zero")
            ));
        }
        
        if timeout.as_secs() > 3600 {
            return Err(ConfigurationError::InvalidTimeout(
                format!("{name} cannot exceed 1 hour")
            ));
        }
        
        Ok(())
    }
    
    /// Validate buffer size
    /// 
    /// # Errors
    /// 
    /// Returns `ConfigurationError::InvalidBufferSize` if:
    /// - The buffer size is zero
    /// - The buffer size exceeds 1GB (1024³ bytes)
    pub fn validate_buffer_size(size: usize, name: &str) -> ConfigResult<()> {
        if size == 0 {
            return Err(ConfigurationError::InvalidBufferSize(
                format!("{name} cannot be zero")
            ));
        }
        
        if size > 1024 * 1024 * 1024 {  // 1GB limit
            return Err(ConfigurationError::InvalidBufferSize(
                format!("{name} cannot exceed 1GB")
            ));
        }
        
        Ok(())
    }
    
    /// Validate socket address
    /// 
    /// # Errors
    /// 
    /// Returns `ConfigurationError::InvalidAddress` if the socket address has a 
    /// specific IP address but port 0 (which is invalid for actual binding).
    pub fn validate_socket_addr(addr: Option<SocketAddr>, name: &str) -> ConfigResult<()> {
        if let Some(addr) = addr
            && addr.port() == 0 && !addr.ip().is_unspecified() {
                return Err(ConfigurationError::InvalidAddress(
                    format!("{name} has specific IP but port 0")
                ));
            }
        
        Ok(())
    }
    
    /// Validate numeric range
    /// 
    /// # Errors
    /// 
    /// Returns `ConfigurationError::InvalidParameter` if the value is outside 
    /// the specified range [min, max] (inclusive).
    pub fn validate_range<T>(value: T, min: T, max: T, name: &str) -> ConfigResult<()> 
    where
        T: PartialOrd + std::fmt::Display + Copy,
    {
        if value < min || value > max {
            return Err(ConfigurationError::InvalidParameter(
                format!("{name} must be between {min} and {max}, got {value}")
            ));
        }
        
        Ok(())
    }
}

/// Configuration builder pattern helper
pub trait ConfigBuilder<T> {
    /// Builds the final configuration object
    /// 
    /// # Errors
    /// 
    /// Returns a `ConfigurationError` if the configuration cannot be built:
    /// - `InvalidTimeout` - if any timeout configuration is invalid
    /// - `InvalidAddress` - if network address configuration is invalid
    /// - `InvalidBufferSize` - if buffer size configuration is invalid
    /// - `InvalidParameter` - if any parameter is outside valid ranges
    /// - `Conflict` - if configuration parameters conflict with each other
    fn build(self) -> ConfigResult<T>;
}

/// Common configuration defaults
pub struct ConfigDefaults;

impl ConfigDefaults {
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
    pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
    pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(300);
    pub const DEFAULT_BUFFER_SIZE: usize = 8192;
    pub const DEFAULT_MAX_CONNECTIONS: usize = 100;
    pub const DEFAULT_MAX_RETRIES: u32 = 3;
    pub const DEFAULT_USER_AGENT: &'static str = "quyc-http3/1.0";
}