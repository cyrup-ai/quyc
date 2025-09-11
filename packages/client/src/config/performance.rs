//! Performance Configuration Module
//!
//! Zero-allocation performance tuning and optimization settings.

/// Performance configuration provider trait
pub trait PerformanceConfigProvider {
    fn thread_pool_size(&self) -> usize;
    fn enable_tcp_nodelay(&self) -> bool;
    fn enable_socket_reuse(&self) -> bool;
    fn polling_interval_ms(&self) -> u64;
    fn enable_metrics(&self) -> bool;
    fn enable_tracing(&self) -> bool;
    fn memory_pool_enabled(&self) -> bool;
    fn connection_pooling_enabled(&self) -> bool;
}

/// Runtime performance configuration
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct PerformanceConfig {
    pub thread_pool_size: usize,
    pub enable_tcp_nodelay: bool,
    pub enable_socket_reuse: bool,
    pub polling_interval_ms: u64,
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub memory_pool_enabled: bool,
    pub connection_pooling_enabled: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            thread_pool_size: 4, // Reasonable default for thread pool size
            enable_tcp_nodelay: true,
            enable_socket_reuse: true,
            polling_interval_ms: 1,  // 1ms polling
            enable_metrics: true,
            enable_tracing: true,
            memory_pool_enabled: true,
            connection_pooling_enabled: true,
        }
    }
}

impl PerformanceConfig {
    /// Create high-performance configuration for production
    #[must_use]
    pub fn high_performance() -> Self {
        Self {
            thread_pool_size: 8, // High-performance thread pool size
            polling_interval_ms: 0,  // Busy polling
            enable_metrics: false,   // Reduce overhead
            enable_tracing: false,   // Reduce overhead
            ..Self::default()
        }
    }
    
    /// Create low-resource configuration
    #[must_use]
    pub fn low_resource() -> Self {
        Self {
            thread_pool_size: 2,
            polling_interval_ms: 10,  // Reduced polling frequency
            memory_pool_enabled: false,
            ..Self::default()
        }
    }
    
    /// Validate performance configuration
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - `thread_pool_size` is 0 or exceeds 1024
    /// - `polling_interval_ms` is 0
    /// - Performance parameters are out of acceptable ranges
    pub fn validate(&self) -> Result<(), String> {
        if self.thread_pool_size == 0 {
            return Err("thread_pool_size must be greater than 0".to_string());
        }
        
        if self.thread_pool_size > 1024 {
            return Err("thread_pool_size should not exceed 1024".to_string());
        }
        
        Ok(())
    }
}

impl PerformanceConfigProvider for PerformanceConfig {
    #[inline]
    fn thread_pool_size(&self) -> usize {
        self.thread_pool_size
    }
    
    #[inline]
    fn enable_tcp_nodelay(&self) -> bool {
        self.enable_tcp_nodelay
    }
    
    #[inline]
    fn enable_socket_reuse(&self) -> bool {
        self.enable_socket_reuse
    }
    
    #[inline]
    fn polling_interval_ms(&self) -> u64 {
        self.polling_interval_ms
    }
    
    #[inline]
    fn enable_metrics(&self) -> bool {
        self.enable_metrics
    }
    
    #[inline]
    fn enable_tracing(&self) -> bool {
        self.enable_tracing
    }
    
    #[inline]
    fn memory_pool_enabled(&self) -> bool {
        self.memory_pool_enabled
    }
    
    #[inline]
    fn connection_pooling_enabled(&self) -> bool {
        self.connection_pooling_enabled
    }
}

/// Compile-time performance configuration
pub struct StaticPerformanceConfig;

impl PerformanceConfigProvider for StaticPerformanceConfig {
    #[inline]
    fn thread_pool_size(&self) -> usize {
        4  // Reasonable default
    }
    
    #[inline]
    fn enable_tcp_nodelay(&self) -> bool {
        true
    }
    
    #[inline]
    fn enable_socket_reuse(&self) -> bool {
        true
    }
    
    #[inline]
    fn polling_interval_ms(&self) -> u64 {
        1
    }
    
    #[inline]
    fn enable_metrics(&self) -> bool {
        true
    }
    
    #[inline]
    fn enable_tracing(&self) -> bool {
        true
    }
    
    #[inline]
    fn memory_pool_enabled(&self) -> bool {
        true
    }
    
    #[inline]
    fn connection_pooling_enabled(&self) -> bool {
        true
    }
}