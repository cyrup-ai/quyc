//! `JSONPath` evaluation logic for streaming deserialization
//!
//! Contains the logic for evaluating `JSONPath` expressions against the current
//! parsing position during streaming JSON processing.

use serde::de::DeserializeOwned;

use super::iterator::JsonPathIterator;

impl<T> JsonPathIterator<'_, '_, T>
where
    T: DeserializeOwned,
{
    /// Check if current position matches `JSONPath` root object selector
    #[inline]
    pub(super) fn matches_root_object_path(&self) -> bool {
        // Check if the root selector matches object access
        match self.deserializer.path_expression.root_selector() {
            Some(crate::jsonpath::parser::JsonSelector::Child { .. }) => true,
            Some(crate::jsonpath::parser::JsonSelector::Filter { .. }) => true,
            _ => false,
        }
    }

    /// Check if current position matches `JSONPath` root array selector
    #[inline]
    pub(super) fn matches_root_array_path(&self) -> bool {
        matches!(
            self.deserializer.path_expression.root_selector(),
            Some(
                crate::jsonpath::parser::JsonSelector::Wildcard
                | crate::jsonpath::parser::JsonSelector::Index { .. }
                | crate::jsonpath::parser::JsonSelector::Slice { .. }
            )
        )
    }

    /// Check if current position matches `JSONPath` expression
    #[inline]
    pub(super) fn matches_current_path(&self) -> bool {
        self.evaluate_jsonpath_at_current_position()
    }

    /// Evaluate `JSONPath` expression at current parsing position
    #[inline]
    pub(super) fn evaluate_jsonpath_at_current_position(&self) -> bool {
        if self.deserializer.streaming_state.in_recursive_descent {
            // Evaluate recursive descent match using the implemented logic
            self.evaluate_recursive_descent_match()
        } else {
            // Check if we should enter recursive descent mode
            if self.should_enter_recursive_descent() {
                // Note: We need a mutable reference to enter recursive descent mode
                // This will be handled by the caller that has mutable access
                false
            } else {
                self.evaluate_selector_match()
            }
        }
    }

    /// Evaluate current selector considering array indices and slice notation
    #[inline]
    pub(super) fn evaluate_selector_match(&self) -> bool {
        // Get the current selector from the path expression
        let selectors = self.deserializer.path_expression.selectors();
        let selector_index = self
            .deserializer
            .streaming_state.current_selector_index
            .min(selectors.len().saturating_sub(1));

        if selector_index >= selectors.len() {
            return false;
        }

        let current_selector = &selectors[selector_index];
        self.evaluate_single_selector(current_selector)
    }

    /// Evaluate a single selector against current streaming context
    #[inline]
    pub(super) fn evaluate_single_selector(
        &self,
        selector: &crate::jsonpath::parser::JsonSelector,
    ) -> bool {
        use crate::jsonpath::parser::JsonSelector;

        match selector {
            JsonSelector::Root => self.deserializer.current_depth == 0,
            JsonSelector::Child { .. } | JsonSelector::Wildcard => self.deserializer.current_depth > 0,
            JsonSelector::RecursiveDescent => true, // Always matches
            JsonSelector::Index { index, from_end } => {
                self.evaluate_index_selector(*index, *from_end)
            }
            JsonSelector::Slice { start, end, step } => {
                self.evaluate_slice_selector(*start, *end, *step)
            }
            JsonSelector::Filter { expression } => {
                // Evaluate the filter expression against the current JSON context
                if !self.deserializer.object_buffer.is_empty()
                    && let Ok(json_str) = std::str::from_utf8(&self.deserializer.object_buffer)
                        && let Ok(context) = serde_json::from_str::<serde_json::Value>(json_str) {
                            // JsonPathResultExt removed - not available
                            use crate::jsonpath::filter::FilterEvaluator;
                            return FilterEvaluator::evaluate_predicate(&context, expression)
                                .unwrap_or(false);
                        }
                false
            }
            JsonSelector::Union { selectors } => {
                // Union matches if any selector matches
                selectors.iter().any(|s| self.evaluate_single_selector(s))
            }
        }
    }
}
