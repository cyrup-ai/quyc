//! Selector matching and evaluation logic for streaming JSONPath processing
//!
//! Contains specialized logic for evaluating array indices, slices, and other
//! selector types against the current streaming context.

use serde::de::DeserializeOwned;

use super::iterator::JsonPathIterator;

impl<'iter, 'data, T> JsonPathIterator<'iter, 'data, T>
where
    T: DeserializeOwned,
{
    /// Evaluate array index selector against current array position
    #[inline]
    pub(super) fn evaluate_index_selector(&self, index: i64, from_end: bool) -> bool {
        if !self.deserializer.in_target_array {
            return false;
        }

        let current_idx = self.deserializer.current_array_index;

        if from_end || index < 0 {
            // Negative indices require knowing array length, which we don't have in streaming
            // For streaming context, we skip negative index matching
            false
        } else {
            current_idx == index
        }
    }

    /// Evaluate array slice selector against current array position
    #[inline]
    pub(super) fn evaluate_slice_selector(
        &self,
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
    ) -> bool {
        if !self.deserializer.in_target_array {
            return false;
        }

        let current_idx = self.deserializer.current_array_index;
        let step = step.unwrap_or(1);

        // Handle step size
        if step <= 0 {
            return false; // Invalid step
        }

        // Check start boundary
        let start_idx = match start {
            Some(s) if s >= 0 => s,
            Some(_) => return false, // Negative start not supported in streaming
            None => 0,               // Default start
        };

        // Check end boundary (None means no upper limit in streaming context)
        let within_end = match end {
            Some(e) if e >= 0 => current_idx < e,
            Some(_) => false, // Negative end not supported in streaming
            None => true,     // No upper limit
        };

        // Check if current index is within slice bounds and matches step
        let within_start = current_idx >= start_idx;
        let matches_step = (current_idx - start_idx) % step == 0;

        within_start && within_end && matches_step
    }
}
