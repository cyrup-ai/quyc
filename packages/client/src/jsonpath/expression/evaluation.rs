//! Depth evaluation and selector matching for JsonPathExpression
//!
//! Sophisticated depth-based evaluation algorithms for streaming JSONPath
//! processing with recursive descent handling and efficient selector matching.

use super::core::JsonPathExpression;
use crate::jsonpath::ast::JsonSelector;

impl JsonPathExpression {
    /// Check if expression matches at specified JSON depth
    ///
    /// Used during streaming to determine if current parsing position
    /// matches the JSONPath expression navigation requirements.
    ///
    /// # Arguments
    ///
    /// * `depth` - Current JSON nesting depth (0 = root level)
    ///
    /// # Returns
    ///
    /// `true` if the current depth matches the expression's navigation pattern.
    #[inline]
    pub fn matches_at_depth(&self, depth: usize) -> bool {
        self.evaluate_selectors_at_depth(depth, 0).is_some()
    }

    /// Evaluate selector chain recursively to determine if current depth matches
    ///
    /// Handles recursive descent (..) by exploring all possible paths through the JSON structure.
    /// Returns the next selector index to continue evaluation, or None if no match.
    ///
    /// # Arguments
    ///
    /// * `current_depth` - Current JSON nesting depth
    /// * `selector_index` - Index in the selector chain being evaluated
    ///
    /// # Performance
    ///
    /// Uses early termination and efficient recursive evaluation for optimal performance.
    #[inline]
    pub(super) fn evaluate_selectors_at_depth(
        &self,
        current_depth: usize,
        selector_index: usize,
    ) -> Option<usize> {
        // Base case: reached end of selectors
        if selector_index >= self.selectors().len() {
            return Some(selector_index);
        }

        // Base case: depth 0 should only match root selector
        if current_depth == 0 && selector_index == 0 {
            return if matches!(self.selectors()[0], JsonSelector::Root) {
                self.evaluate_selectors_at_depth(current_depth, 1)
            } else {
                None
            };
        }

        let selector = &self.selectors()[selector_index];

        match selector {
            JsonSelector::Root => {
                // Root can only match at the beginning
                if selector_index == 0 {
                    self.evaluate_selectors_at_depth(current_depth, selector_index + 1)
                } else {
                    None
                }
            }

            JsonSelector::RecursiveDescent => {
                // Recursive descent matches at any depth
                // Try to match the next selector at current depth or any deeper depth
                let next_selector_index = selector_index + 1;

                if next_selector_index >= self.selectors().len() {
                    // Recursive descent at end matches everything
                    return Some(next_selector_index);
                }

                // Try to match next selector at current depth
                if let Some(result) =
                    self.evaluate_selectors_at_depth(current_depth, next_selector_index)
                {
                    return Some(result);
                }

                // Try to match recursive descent at deeper levels (simulated)
                // In streaming context, this means we stay in recursive descent mode
                // until we find a matching structure
                if current_depth < 20 {
                    // Reasonable depth limit
                    self.evaluate_selectors_at_depth(current_depth + 1, selector_index)
                } else {
                    None
                }
            }

            JsonSelector::Child { .. } => {
                // Child selectors require exact depth progression
                if current_depth > 0 {
                    self.evaluate_selectors_at_depth(current_depth, selector_index + 1)
                } else {
                    None
                }
            }

            JsonSelector::Index { .. } | JsonSelector::Slice { .. } | JsonSelector::Wildcard => {
                // Array selectors require being inside an array
                if current_depth > 0 {
                    self.evaluate_selectors_at_depth(current_depth, selector_index + 1)
                } else {
                    None
                }
            }

            JsonSelector::Filter { .. } => {
                // Filter expressions require context evaluation (handled at runtime)
                if current_depth > 0 {
                    self.evaluate_selectors_at_depth(current_depth, selector_index + 1)
                } else {
                    None
                }
            }

            JsonSelector::Union { selectors } => {
                // Union matches if any selector matches at this depth
                for union_selector in selectors {
                    if self.evaluate_single_selector_at_depth(union_selector, current_depth) {
                        return self.evaluate_selectors_at_depth(current_depth, selector_index + 1);
                    }
                }
                None
            }
        }
    }

    /// Evaluate a single selector at specific depth
    #[inline]
    pub(super) fn evaluate_single_selector_at_depth(
        &self,
        selector: &JsonSelector,
        depth: usize,
    ) -> bool {
        match selector {
            JsonSelector::Root => depth == 0,
            JsonSelector::RecursiveDescent => true, // Always matches
            JsonSelector::Child { .. }
            | JsonSelector::Index { .. }
            | JsonSelector::Slice { .. }
            | JsonSelector::Wildcard
            | JsonSelector::Filter { .. } => depth > 0,
            JsonSelector::Union { selectors } => selectors
                .iter()
                .any(|s| self.evaluate_single_selector_at_depth(s, depth)),
        }
    }
}


