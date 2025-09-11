//! Client Configuration Module
//!
//! High-level client configuration combining all configuration aspects.


use super::{
    NetworkConfig, BufferConfig, ProtocolConfig, RetryConfig, 
    PerformanceConfig, SecurityConfig, HttpConfig
};

/// Complete client configuration
#[derive(Debug, Clone, Default)]
pub struct ClientConfig {
    pub network: NetworkConfig,
    pub buffers: BufferConfig,
    pub protocol: ProtocolConfig,
    pub retry: RetryConfig,
    pub performance: PerformanceConfig,
    pub security: SecurityConfig,
    pub http: HttpConfig,
}

impl ClientConfig {
    /// Create production-optimized client configuration
    #[must_use]
    pub fn production() -> Self {
        Self {
            buffers: BufferConfig::high_performance(),
            protocol: ProtocolConfig::production(),
            retry: RetryConfig::conservative(),
            performance: PerformanceConfig::high_performance(),
            security: SecurityConfig::high_security(),
            ..Self::default()
        }
    }
    
    /// Create development-friendly client configuration
    #[must_use]
    pub fn development() -> Self {
        Self {
            retry: RetryConfig::aggressive(),
            security: SecurityConfig::development(),
            ..Self::default()
        }
    }
    
    /// Create low-resource client configuration
    #[must_use]
    pub fn low_resource() -> Self {
        Self {
            buffers: BufferConfig::low_memory(),
            performance: PerformanceConfig::low_resource(),
            ..Self::default()
        }
    }
    
    /// Validate complete client configuration
    /// 
    /// # Errors
    /// 
    /// Returns a `ConfigurationError` if:
    /// - Network configuration is invalid
    /// - Buffer configuration is invalid  
    /// - Protocol configuration is invalid
    /// - Retry configuration is invalid
    /// - Performance configuration is invalid
    /// - Security configuration is invalid
    /// - Any sub-configuration validation fails
    pub fn validate(&self) -> Result<(), crate::config::ConfigurationError> {
        use crate::config::ConfigurationError;
        
        self.network.validate()
            .map_err(ConfigurationError::Network)?;
        
        self.buffers.validate()
            .map_err(|e| ConfigurationError::Validation(format!("Buffer config: {e}")))?;
        
        self.protocol.validate()
            .map_err(|e| ConfigurationError::Validation(format!("Protocol config: {e}")))?;
        
        self.retry.validate()
            .map_err(|e| ConfigurationError::Validation(format!("Retry config: {e}")))?;
        
        self.performance.validate()
            .map_err(|e| ConfigurationError::Validation(format!("Performance config: {e}")))?;
        
        self.security.validate()
            .map_err(|e| ConfigurationError::Validation(format!("Security config: {e}")))?;
        
        Ok(())
    }
}