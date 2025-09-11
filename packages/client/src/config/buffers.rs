//! Buffer Configuration Module
//!
//! Memory management and buffer size configuration for optimal performance.

/// Buffer configuration provider trait
pub trait BufferConfigProvider {
    fn default_buffer_size(&self) -> usize;
    fn max_buffer_size(&self) -> usize;
    fn initial_window_size(&self) -> usize;
    fn max_frame_size(&self) -> usize;
    fn connection_window_size(&self) -> usize;
    fn stream_window_size(&self) -> usize;
    fn header_table_size(&self) -> usize;
    fn enable_push(&self) -> bool;
}

/// Runtime buffer configuration
#[derive(Debug, Clone)]
pub struct BufferConfig {
    pub default_buffer_size: usize,
    pub max_buffer_size: usize,
    pub initial_window_size: usize,
    pub max_frame_size: usize,
    pub connection_window_size: usize,
    pub stream_window_size: usize,
    pub header_table_size: usize,
    pub enable_push: bool,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            default_buffer_size: 8192,        // 8KB
            max_buffer_size: 1_048_576,         // 1MB
            initial_window_size: 65_535,       // HTTP/2 default
            max_frame_size: 16_384,            // HTTP/2 default
            connection_window_size: 1_048_576,  // 1MB
            stream_window_size: 262_144,       // 256KB
            header_table_size: 4096,          // 4KB
            enable_push: false,
        }
    }
}

impl BufferConfig {
    /// Create high-performance buffer configuration
    #[must_use]
    pub fn high_performance() -> Self {
        Self {
            default_buffer_size: 32_768,       // 32KB
            max_buffer_size: 16_777_216,        // 16MB
            initial_window_size: 1_048_576,     // 1MB
            max_frame_size: 32_768,            // 32KB
            connection_window_size: 16_777_216, // 16MB
            stream_window_size: 1_048_576,      // 1MB
            header_table_size: 8192,          // 8KB
            enable_push: true,
        }
    }
    
    /// Create memory-constrained buffer configuration
    #[must_use]
    pub fn low_memory() -> Self {
        Self {
            default_buffer_size: 4096,        // 4KB
            max_buffer_size: 262_144,          // 256KB
            initial_window_size: 32_768,       // 32KB
            max_frame_size: 8192,             // 8KB
            connection_window_size: 262_144,   // 256KB
            stream_window_size: 65_536,        // 64KB
            header_table_size: 2048,          // 2KB
            enable_push: false,
        }
    }
    
    /// Validate buffer configuration
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - `default_buffer_size` is 0
    /// - `max_buffer_size` is less than `default_buffer_size`
    /// - `max_frame_size` is not between 16384 and 16777215
    /// - Window sizes are invalid or exceed protocol limits
    pub fn validate(&self) -> Result<(), String> {
        if self.default_buffer_size == 0 {
            return Err("default_buffer_size must be greater than 0".to_string());
        }
        
        if self.max_buffer_size < self.default_buffer_size {
            return Err("max_buffer_size must be >= default_buffer_size".to_string());
        }
        
        if self.max_frame_size < 16_384 || self.max_frame_size > 16_777_215 {
            return Err("max_frame_size must be between 16384 and 16777215".to_string());
        }
        
        if self.initial_window_size < 65_535 {
            return Err("initial_window_size must be at least 65535".to_string());
        }
        
        Ok(())
    }
}

impl BufferConfigProvider for BufferConfig {
    #[inline]
    fn default_buffer_size(&self) -> usize {
        self.default_buffer_size
    }
    
    #[inline]
    fn max_buffer_size(&self) -> usize {
        self.max_buffer_size
    }
    
    #[inline]
    fn initial_window_size(&self) -> usize {
        self.initial_window_size
    }
    
    #[inline]
    fn max_frame_size(&self) -> usize {
        self.max_frame_size
    }
    
    #[inline]
    fn connection_window_size(&self) -> usize {
        self.connection_window_size
    }
    
    #[inline]
    fn stream_window_size(&self) -> usize {
        self.stream_window_size
    }
    
    #[inline]
    fn header_table_size(&self) -> usize {
        self.header_table_size
    }
    
    #[inline]
    fn enable_push(&self) -> bool {
        self.enable_push
    }
}

/// Compile-time buffer configuration for zero-allocation access
pub struct StaticBufferConfig<const BUFFER_SIZE: usize = 8192>;

impl<const BUFFER_SIZE: usize> BufferConfigProvider for StaticBufferConfig<BUFFER_SIZE> {
    #[inline]
    fn default_buffer_size(&self) -> usize {
        BUFFER_SIZE
    }
    
    #[inline]
    fn max_buffer_size(&self) -> usize {
        BUFFER_SIZE * 128  // 128x default size
    }
    
    #[inline]
    fn initial_window_size(&self) -> usize {
        65535
    }
    
    #[inline]
    fn max_frame_size(&self) -> usize {
        16384
    }
    
    #[inline]
    fn connection_window_size(&self) -> usize {
        1_048_576
    }
    
    #[inline]
    fn stream_window_size(&self) -> usize {
        262_144
    }
    
    #[inline]
    fn header_table_size(&self) -> usize {
        4096
    }
    
    #[inline]
    fn enable_push(&self) -> bool {
        false
    }
}