//! Path operations and relationships for normalized paths
//!
//! This module provides methods for creating child paths, finding parents,
//! and determining relationships between normalized paths.

use super::types::{NormalizedPath, NormalizedPathProcessor, PathSegment};
use crate::jsonpath::error::{JsonPathResult, invalid_expression_error};

impl NormalizedPath {
    /// Create a child path by appending a member access
    #[inline]
    pub fn child_member(&self, member_name: &str) -> JsonPathResult<Self> {
        NormalizedPathProcessor::validate_member_name(member_name)?;

        let mut new_segments = self.segments.clone();
        new_segments.push(PathSegment::Member(member_name.to_string()));

        let normalized_string = NormalizedPathProcessor::segments_to_string(&new_segments);

        Ok(Self {
            segments: new_segments,
            normalized_string,
        })
    }

    /// Create a child path by appending an array index access
    #[inline]
    pub fn child_index(&self, index: i64) -> JsonPathResult<Self> {
        if index < 0 {
            return Err(invalid_expression_error(
                "",
                "normalized paths require non-negative array indices",
                None,
            ));
        }

        let mut new_segments = self.segments.clone();
        new_segments.push(PathSegment::Index(index));

        let normalized_string = NormalizedPathProcessor::segments_to_string(&new_segments);

        Ok(Self {
            segments: new_segments,
            normalized_string,
        })
    }

    /// Get the parent path (all segments except the last)
    #[inline]
    #[must_use] 
    pub fn parent(&self) -> Option<Self> {
        if self.segments.len() <= 1 {
            return None; // Root has no parent
        }

        let parent_segments = self.segments[..self.segments.len() - 1].to_vec();
        let normalized_string = NormalizedPathProcessor::segments_to_string(&parent_segments);

        Some(Self {
            segments: parent_segments,
            normalized_string,
        })
    }

    /// Check if this path is a descendant of another path
    #[inline]
    #[must_use] 
    pub fn is_descendant_of(&self, ancestor: &NormalizedPath) -> bool {
        if self.segments.len() <= ancestor.segments.len() {
            return false;
        }

        self.segments[..ancestor.segments.len()] == ancestor.segments
    }

    /// Check if this path is an ancestor of another path
    #[inline]
    #[must_use] 
    pub fn is_ancestor_of(&self, descendant: &NormalizedPath) -> bool {
        descendant.is_descendant_of(self)
    }
}
