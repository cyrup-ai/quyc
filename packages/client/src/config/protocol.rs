//! Protocol Configuration Module
//!
//! HTTP/2 and HTTP/3 protocol-specific configuration settings.

/// Protocol configuration provider trait
pub trait ProtocolConfigProvider {
    fn http2_prior_knowledge(&self) -> bool;
    fn http2_adaptive_window(&self) -> bool;
    fn http3_max_idle_timeout_ms(&self) -> u64;
    fn http3_enable_0rtt(&self) -> bool;
    fn http3_enable_early_data(&self) -> bool;
    fn compression_enabled(&self) -> bool;
    fn compression_level(&self) -> u32;
}

/// Runtime protocol configuration
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct ProtocolConfig {
    pub http2_prior_knowledge: bool,
    pub http2_adaptive_window: bool,
    pub http3_max_idle_timeout_ms: u64,
    pub http3_enable_0rtt: bool,
    pub http3_enable_early_data: bool,
    pub compression_enabled: bool,
    pub compression_level: u32,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            http2_prior_knowledge: false,
            http2_adaptive_window: true,
            http3_max_idle_timeout_ms: 30000,  // 30 seconds
            http3_enable_0rtt: true,
            http3_enable_early_data: true,
            compression_enabled: true,
            compression_level: 6,  // Balanced compression
        }
    }
}

impl ProtocolConfig {
    /// Create production-optimized protocol configuration
    #[must_use]
    pub fn production() -> Self {
        Self {
            compression_level: 4,  // Lower CPU usage
            ..Self::default()
        }
    }
    
    /// Validate protocol configuration
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - `compression_level` is greater than 9
    /// - `http3_max_idle_timeout_ms` is 0
    /// - Protocol configuration parameters are out of valid ranges
    pub fn validate(&self) -> Result<(), String> {
        if self.compression_level > 9 {
            return Err("compression_level must be between 0 and 9".to_string());
        }
        
        if self.http3_max_idle_timeout_ms == 0 {
            return Err("http3_max_idle_timeout_ms must be greater than 0".to_string());
        }
        
        Ok(())
    }
}

impl ProtocolConfigProvider for ProtocolConfig {
    #[inline]
    fn http2_prior_knowledge(&self) -> bool {
        self.http2_prior_knowledge
    }
    
    #[inline]
    fn http2_adaptive_window(&self) -> bool {
        self.http2_adaptive_window
    }
    
    #[inline]
    fn http3_max_idle_timeout_ms(&self) -> u64 {
        self.http3_max_idle_timeout_ms
    }
    
    #[inline]
    fn http3_enable_0rtt(&self) -> bool {
        self.http3_enable_0rtt
    }
    
    #[inline]
    fn http3_enable_early_data(&self) -> bool {
        self.http3_enable_early_data
    }
    
    #[inline]
    fn compression_enabled(&self) -> bool {
        self.compression_enabled
    }
    
    #[inline]
    fn compression_level(&self) -> u32 {
        self.compression_level
    }
}

/// Compile-time protocol configuration
pub struct StaticProtocolConfig;

impl ProtocolConfigProvider for StaticProtocolConfig {
    #[inline]
    fn http2_prior_knowledge(&self) -> bool {
        false
    }
    
    #[inline]
    fn http2_adaptive_window(&self) -> bool {
        true
    }
    
    #[inline]
    fn http3_max_idle_timeout_ms(&self) -> u64 {
        30000
    }
    
    #[inline]
    fn http3_enable_0rtt(&self) -> bool {
        true
    }
    
    #[inline]
    fn http3_enable_early_data(&self) -> bool {
        true
    }
    
    #[inline]
    fn compression_enabled(&self) -> bool {
        true
    }
    
    #[inline]
    fn compression_level(&self) -> u32 {
        6
    }
}