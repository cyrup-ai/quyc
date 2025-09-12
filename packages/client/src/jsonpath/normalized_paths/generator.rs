//! Normalized path generation from `JSONPath` selectors
//!
//! This module handles the conversion of `JSONPath` selectors into normalized paths,
//! validating that they represent unique single-node paths.

use super::types::{NormalizedPath, NormalizedPathProcessor, PathSegment};
use crate::jsonpath::{
    ast::JsonSelector,
    error::{JsonPathResult, invalid_expression_error},
};

impl NormalizedPathProcessor {
    /// Generate normalized path from `JSONPath` selectors
    ///
    /// Takes a sequence of `JSONPath` selectors and converts them to
    /// a normalized path if they represent a unique single-node path.
    ///
    /// # Errors
    ///
    /// Returns error if the selectors don't represent a normalized path:
    /// - Contains wildcards, filters, or other multi-node selectors
    /// - Contains recursive descent that doesn't target a specific node
    /// - Contains union selectors
    /// - Invalid or malformed selectors
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub fn generate_normalized_path(selectors: &[JsonSelector]) -> JsonPathResult<NormalizedPath> {
        let mut segments = Vec::new();

        // First selector should always be Root for normalized paths
        if selectors.is_empty() {
            return Ok(NormalizedPath::root());
        }

        // Validate and convert each selector
        for (index, selector) in selectors.iter().enumerate() {
            match selector {
                JsonSelector::Root => {
                    if index != 0 {
                        return Err(invalid_expression_error(
                            "",
                            "root selector can only appear at the beginning",
                            Some(index),
                        ));
                    }
                    segments.push(PathSegment::Root);
                }

                JsonSelector::Child { name, .. } => {
                    Self::validate_member_name(name)?;
                    segments.push(PathSegment::Member(name.clone()));
                }

                JsonSelector::Index { index, from_end } => {
                    if *from_end {
                        let abs_index = index.wrapping_abs();
                        let safe_index = usize::try_from(abs_index).unwrap_or_else(|_| {
                            tracing::warn!("Index {} too large for usize, clamping to max", abs_index);
                            usize::MAX
                        });
                        return Err(invalid_expression_error(
                            "",
                            "normalized paths cannot contain negative indices",
                            Some(safe_index),
                        ));
                    }
                    if *index < 0 {
                        let abs_index = usize::try_from((*index).wrapping_abs()).unwrap_or(0);
                        return Err(invalid_expression_error(
                            "",
                            "normalized paths require non-negative array indices",
                            Some(abs_index),
                        ));
                    }
                    segments.push(PathSegment::Index(*index));
                }

                // These selectors cannot appear in normalized paths
                JsonSelector::Wildcard => {
                    return Err(invalid_expression_error(
                        "",
                        "normalized paths cannot contain wildcard selectors",
                        Some(index),
                    ));
                }
                JsonSelector::Slice { .. } => {
                    return Err(invalid_expression_error(
                        "",
                        "normalized paths cannot contain slice selectors",
                        Some(index),
                    ));
                }
                JsonSelector::Filter { .. } => {
                    return Err(invalid_expression_error(
                        "",
                        "normalized paths cannot contain filter selectors",
                        Some(index),
                    ));
                }
                JsonSelector::Union { .. } => {
                    return Err(invalid_expression_error(
                        "",
                        "normalized paths cannot contain union selectors",
                        Some(index),
                    ));
                }
                JsonSelector::RecursiveDescent => {
                    return Err(invalid_expression_error(
                        "",
                        "normalized paths cannot contain recursive descent",
                        Some(index),
                    ));
                }
            }
        }

        // Generate the normalized string representation
        let normalized_string = Self::segments_to_string(&segments);

        Ok(NormalizedPath {
            segments,
            normalized_string,
        })
    }

    /// Validate member name according to normalized path rules
    #[inline]
    pub(crate) fn validate_member_name(name: &str) -> JsonPathResult<()> {
        // Member names in normalized paths must be valid UTF-8 strings
        // No additional restrictions beyond basic JSON string requirements
        if name
            .chars()
            .any(|c| c.is_control() && c != '\t' && c != '\n' && c != '\r')
        {
            return Err(invalid_expression_error(
                "",
                "member names cannot contain control characters",
                None,
            ));
        }
        Ok(())
    }

    /// Convert segments to normalized string representation
    #[inline]
    pub(crate) fn segments_to_string(segments: &[PathSegment]) -> String {
        let mut result = String::new();

        for segment in segments {
            match segment {
                PathSegment::Root => result.push('$'),
                PathSegment::Member(name) => {
                    result.push('[');
                    result.push('\'');
                    // Escape quotes and backslashes in member names
                    for ch in name.chars() {
                        if ch == '\'' || ch == '\\' {
                            result.push('\\');
                        }
                        result.push(ch);
                    }
                    result.push('\'');
                    result.push(']');
                }
                PathSegment::Index(index) => {
                    result.push('[');
                    result.push_str(&index.to_string());
                    result.push(']');
                }
            }
        }

        result
    }
}
