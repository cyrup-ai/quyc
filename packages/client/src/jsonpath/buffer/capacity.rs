//! Intelligent buffer capacity management
//!
//! Handles buffer growth and shrinking based on usage patterns to minimize
//! memory usage while avoiding frequent reallocations. Includes hysteresis
//! to prevent size thrashing and adaptive growth patterns.

use bytes::BytesMut;

/// Intelligent buffer capacity management
///
/// Handles buffer growth and shrinking based on usage patterns to minimize
/// memory usage while avoiding frequent reallocations. Includes hysteresis
/// to prevent size thrashing and adaptive growth patterns.
#[derive(Debug)]
pub struct CapacityManager {
    pub(super) initial_capacity: usize,
    pub(super) max_capacity: usize,
    growth_factor: f64,
    shrink_threshold: f64,
    /// Last time buffer was shrunk to prevent thrashing
    pub(super) last_shrink_size: Option<usize>,
    /// Number of growth operations since last shrink
    pub(super) growth_operations: u32,
    /// Minimum operations before allowing shrink after growth
    pub(super) hysteresis_threshold: u32,
}

impl CapacityManager {
    pub(super) fn new(initial_capacity: usize) -> Self {
        Self {
            initial_capacity,
            max_capacity: initial_capacity * 16, // Max 16x growth
            growth_factor: 2.0,
            shrink_threshold: 0.25, // Shrink when less than 25% utilized
            last_shrink_size: None,
            growth_operations: 0,
            hysteresis_threshold: 3, // Wait for 3 growth ops before allowing shrink
        }
    }

    pub(super) fn ensure_capacity(&mut self, buffer: &mut BytesMut, needed: usize) {
        let current_capacity = buffer.capacity();
        let current_size = buffer.len();
        let required = current_size + needed;

        if required > current_capacity {
            let new_capacity = std::cmp::min(
                self.max_capacity,
                std::cmp::max(
                    required,
                    (current_capacity as f64 * self.growth_factor) as usize,
                ),
            );

            buffer.reserve(new_capacity - current_capacity);

            // Track growth operation for hysteresis
            self.growth_operations = self.growth_operations.saturating_add(1);
        }
    }

    pub(super) fn maybe_shrink(&mut self, buffer: &mut BytesMut) {
        let capacity = buffer.capacity();
        let size = buffer.len();
        let utilization = size as f64 / capacity as f64;

        // Hysteresis check: don't shrink immediately after growing
        if self.growth_operations < self.hysteresis_threshold {
            return;
        }

        // Don't shrink to a size we recently shrunk from (prevents oscillation)
        if let Some(last_shrink) = self.last_shrink_size {
            if capacity <= last_shrink * 2 {
                return;
            }
        }

        // Only shrink if significantly under-utilized and above initial capacity
        // Additional check: only shrink if we can save significant memory (at least 8KB)
        if utilization < self.shrink_threshold
            && capacity > self.initial_capacity * 2
            && capacity > size + 8192
        {
            let target_capacity = std::cmp::max(
                self.initial_capacity,
                (size as f64 / self.shrink_threshold) as usize,
            );

            // Only proceed if the saving is significant (at least 50% capacity reduction)
            if target_capacity < capacity / 2 {
                // Create new buffer with optimal capacity
                let mut new_buffer = BytesMut::with_capacity(target_capacity);

                // Zero-copy the existing data into the new buffer
                if size > 0 {
                    new_buffer.extend_from_slice(&buffer[..size]);
                }

                // Replace the old buffer with the optimized one
                *buffer = new_buffer;

                // Update shrink tracking
                self.last_shrink_size = Some(capacity);
                self.growth_operations = 0; // Reset growth counter after shrink

                log::debug!(
                    "Buffer shrunk: {} bytes -> {} bytes (saved {} bytes)",
                    capacity,
                    target_capacity,
                    capacity - target_capacity
                );
            }
        }
    }

    pub(super) fn reset(&mut self) {
        // Reset capacity management state for new stream
        self.last_shrink_size = None;
        self.growth_operations = 0;
    }
}
