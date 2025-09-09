//! Statistics and monitoring types for buffer performance
//!
//! Contains statistics structures for monitoring buffer performance,
//! capacity utilization, and other metrics useful for optimization.

/// Buffer performance and utilization statistics
#[derive(Debug, Clone, Copy)]
pub struct BufferStats {
    /// Current buffer size in bytes
    pub current_size: usize,
    /// Buffer capacity in bytes
    pub capacity: usize,
    /// Total bytes processed since creation
    pub total_processed: u64,
    /// Buffer utilization ratio (0.0 to 1.0)
    pub utilization_ratio: f64,
}

/// Detailed capacity management statistics for advanced monitoring
#[derive(Debug, Clone, Copy)]
pub struct CapacityStats {
    /// Initial buffer capacity
    pub initial_capacity: usize,
    /// Maximum allowed capacity
    pub max_capacity: usize,
    /// Current buffer capacity
    pub current_capacity: usize,
    /// Number of growth operations since last shrink
    pub growth_operations: u32,
    /// Size of buffer when it was last shrunk
    pub last_shrink_size: Option<usize>,
    /// Whether buffer is eligible for shrinking
    pub can_shrink: bool,
}

/// Legacy alias for backward compatibility
pub type JsonBuffer = crate::jsonpath::buffer::StreamBuffer;
