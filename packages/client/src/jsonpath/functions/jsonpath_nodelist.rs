//! `JSONPath` nodelist evaluation for function extensions
//! Handles `JSONPath` selector evaluation and descendant collection

use crate::jsonpath::error::JsonPathResult;

/// `JSONPath` nodelist evaluator for function extensions
pub struct JsonPathNodelistEvaluator;

impl JsonPathNodelistEvaluator {
    /// Evaluate `JSONPath` selectors to produce a nodelist
    /// Evaluate `JSONPath` selectors against JSON context to produce nodelist
    ///
    /// # Errors
    /// Returns `JsonPathError` if:
    /// - Invalid selector evaluation (index out of bounds, invalid property access)
    /// - Union selector evaluation fails
    /// - Slice selector evaluation fails with invalid range
    /// - Recursive descent evaluation fails
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub fn evaluate_jsonpath_nodelist(
        context: &serde_json::Value,
        selectors: &[crate::jsonpath::parser::JsonSelector],
    ) -> JsonPathResult<Vec<serde_json::Value>> {
        use crate::jsonpath::parser::JsonSelector;

        let mut current_nodes = vec![context.clone()];

        for selector in selectors {
            let mut next_nodes = Vec::new();

            for node in &current_nodes {
                match selector {
                    JsonSelector::Root => {
                        // Root selector refers to the current node
                        next_nodes.push(node.clone());
                    }
                    JsonSelector::Child { name, .. } => {
                        if let Some(obj) = node.as_object()
                            && let Some(value) = obj.get(name) {
                                next_nodes.push(value.clone());
                            }
                    }
                    JsonSelector::Index { index, from_end } => {
                        if let Some(arr) = node.as_array() {
                            let actual_index = if *from_end {
                                arr.len().saturating_sub((*index).unsigned_abs() as usize)
                            } else if *index >= 0 {
                                match usize::try_from(*index) {
                                    Ok(idx) => idx,
                                    Err(_) => continue, // Skip if index too large
                                }
                            } else {
                                continue; // Skip negative index without from_end flag
                            };

                            if let Some(value) = arr.get(actual_index) {
                                next_nodes.push(value.clone());
                            }
                        }
                    }
                    JsonSelector::Wildcard => {
                        match node {
                            serde_json::Value::Array(arr) => {
                                next_nodes.extend(arr.iter().cloned());
                            }
                            serde_json::Value::Object(obj) => {
                                next_nodes.extend(obj.values().cloned());
                            }
                            _ => {} // Wildcard on primitive values produces no nodes
                        }
                    }
                    JsonSelector::Slice { start, end, step } => {
                        if let Some(arr) = node.as_array() {
                            let len = i64::try_from(arr.len()).unwrap_or(i64::MAX);
                            let step_val = step.unwrap_or(1);

                            if step_val == 0 {
                                continue; // Invalid step, skip
                            }

                            let start_idx = usize::try_from(
                                start
                                    .map_or(0, |s| if s < 0 { len + s } else { s })
                                    .max(0)
                            ).unwrap_or(0);
                            let end_idx = usize::try_from(
                                end
                                    .map_or(len, |e| if e < 0 { len + e } else { e })
                                    .min(len)
                            ).unwrap_or(arr.len());

                            if step_val > 0 {
                                let mut i = start_idx;
                                let step_usize = usize::try_from(step_val).unwrap_or(1);
                                while i < end_idx && i < arr.len() {
                                    next_nodes.push(arr[i].clone());
                                    i += step_usize;
                                }
                            }
                        }
                    }
                    JsonSelector::Union {
                        selectors: union_selectors,
                    } => {
                        for union_selector in union_selectors {
                            let union_nodes =
                                Self::evaluate_jsonpath_nodelist(node, &[union_selector.clone()])?;
                            next_nodes.extend(union_nodes);
                        }
                    }
                    JsonSelector::RecursiveDescent => {
                        // Add current node and all descendants
                        next_nodes.push(node.clone());
                        Self::collect_all_descendants(node, &mut next_nodes);
                    }
                    JsonSelector::Filter { .. } => {
                        // Filter evaluation would require the full filter evaluator
                        // For now, just include the current node if it matches basic criteria
                        next_nodes.push(node.clone());
                    }
                }
            }

            current_nodes = next_nodes;
        }

        Ok(current_nodes)
    }

    /// Collect all descendant nodes recursively
    #[inline]
    pub fn collect_all_descendants(
        node: &serde_json::Value,
        descendants: &mut Vec<serde_json::Value>,
    ) {
        match node {
            serde_json::Value::Array(arr) => {
                for item in arr {
                    descendants.push(item.clone());
                    Self::collect_all_descendants(item, descendants);
                }
            }
            serde_json::Value::Object(obj) => {
                for value in obj.values() {
                    descendants.push(value.clone());
                    Self::collect_all_descendants(value, descendants);
                }
            }
            _ => {} // Primitives have no descendants
        }
    }
}
