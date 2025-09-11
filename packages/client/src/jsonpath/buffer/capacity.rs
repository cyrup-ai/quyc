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

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub(super) fn ensure_capacity(&mut self, buffer: &mut BytesMut, needed: usize) {
        let current_capacity = buffer.capacity();
        let current_size = buffer.len();
        let required = current_size + needed;

        if required > current_capacity {
            // Use safe precision-aware buffer growth calculation
            let growth_target = if current_capacity > (1u64 << 53) as usize {
                // For very large capacities, avoid f64 precision loss by using integer arithmetic
                tracing::debug!(
                    target: "quyc::jsonpath::buffer",
                    current_capacity = current_capacity,
                    "Using integer arithmetic for very large buffer capacity to avoid precision loss"
                );
                // Use integer doubling for large buffers to avoid f64 precision issues
                current_capacity.saturating_mul(2).min(self.max_capacity)
            } else {
                // Safe to use f64 for smaller capacities
                let growth_calculation = (current_capacity as f64) * self.growth_factor;
                if growth_calculation > usize::MAX as f64 {
                    tracing::warn!(
                        target: "quyc::jsonpath::buffer",
                        current_capacity = current_capacity,
                        growth_factor = self.growth_factor,
                        growth_calculation = growth_calculation,
                        max_usize = usize::MAX,
                        "Growth calculation exceeds usize limits, using max_capacity"
                    );
                    self.max_capacity
                } else if growth_calculation < 0.0 {
                    tracing::warn!(
                        target: "quyc::jsonpath::buffer",
                        growth_calculation = growth_calculation,
                        "Negative growth calculation, using current capacity"
                    );
                    current_capacity
                } else {
                    // Safe cast: growth_calculation is positive and within usize bounds
                    growth_calculation as usize
                }
            };
            let new_capacity = std::cmp::min(
                self.max_capacity,
                std::cmp::max(required, growth_target),
            );

            buffer.reserve(new_capacity - current_capacity);

            // Track growth operation for hysteresis
            self.growth_operations = self.growth_operations.saturating_add(1);
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    pub(super) fn maybe_shrink(&mut self, buffer: &mut BytesMut) {
        let capacity = buffer.capacity();
        let size = buffer.len();
        // Use safe precision-aware utilization calculation
        let utilization = if capacity == 0 {
            0.0 // Avoid division by zero
        } else if size > (1u64 << 53) as usize || capacity > (1u64 << 53) as usize {
            // For very large sizes, use integer comparison to avoid precision loss
            if size * 4 < capacity { // Less than 25% utilized
                0.2 // Below shrink threshold
            } else if size * 2 < capacity { // Less than 50% utilized
                0.4 // Between thresholds
            } else {
                0.8 // Above shrink threshold
            }
        } else {
            // Safe to use f64 for smaller sizes
            (size as f64) / (capacity as f64)
        };

        // Hysteresis check: don't shrink immediately after growing
        if self.growth_operations < self.hysteresis_threshold {
            return;
        }

        // Don't shrink to a size we recently shrunk from (prevents oscillation)
        if let Some(last_shrink) = self.last_shrink_size
            && capacity <= last_shrink * 2 {
                return;
            }

        // Only shrink if significantly under-utilized and above initial capacity
        // Additional check: only shrink if we can save significant memory (at least 8KB)
        if utilization < self.shrink_threshold
            && capacity > self.initial_capacity * 2
            && capacity > size + 8192
        {
            // Use safe precision-aware shrink target calculation
            let target_capacity = if size > (1u64 << 53) as usize {
                // For very large sizes, use integer arithmetic to avoid precision loss
                tracing::debug!(
                    target: "quyc::jsonpath::buffer",
                    size = size,
                    "Using integer arithmetic for very large buffer size to avoid precision loss"
                );
                // Use integer calculation: target = size / 0.25 = size * 4
                size.saturating_mul(4).max(self.initial_capacity)
            } else {
                // Safe to use f64 for smaller sizes
                let shrink_calculation = (size as f64) / self.shrink_threshold;
                if shrink_calculation > usize::MAX as f64 {
                    tracing::warn!(
                        target: "quyc::jsonpath::buffer",
                        size = size,
                        shrink_threshold = self.shrink_threshold,
                        shrink_calculation = shrink_calculation,
                        max_usize = usize::MAX,
                        "Shrink calculation exceeds usize limits, using current size"
                    );
                    size
                } else if shrink_calculation < 0.0 {
                    tracing::warn!(
                        target: "quyc::jsonpath::buffer",
                        shrink_calculation = shrink_calculation,
                        "Negative shrink calculation, using current size"
                    );
                    size
                } else {
                    // Safe cast: shrink_calculation is positive and within usize bounds
                    std::cmp::max(self.initial_capacity, shrink_calculation as usize)
                }
            };

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
