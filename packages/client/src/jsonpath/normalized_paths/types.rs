//! Core types for RFC 9535 Normalized Paths Implementation
//!
//! This module defines the fundamental types used for normalized `JSONPath` expressions
//! that uniquely identify single nodes in JSON values using canonical syntax.

use std::fmt;

/// A normalized `JSONPath` expression that uniquely identifies a single node
///
/// Normalized paths use a canonical syntax as defined in RFC 9535 Section 2.7.
/// They are used for reliable node identification and path comparison.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedPath {
    /// The canonical path segments in normalized form
    pub(crate) segments: Vec<PathSegment>,
    /// The complete normalized path string
    pub(crate) normalized_string: String,
}

/// Individual segment in a normalized path
///
/// Each segment represents one level of navigation through the JSON structure
/// using the canonical bracket notation syntax.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegment {
    /// Root segment ($)
    Root,
    /// Object member access (['`member_name`'])
    Member(String),
    /// Array index access ([index])
    Index(i64),
}

/// Normalized Path Generator and Validator
pub struct NormalizedPathProcessor;

impl NormalizedPath {
    /// Create a root normalized path ($)
    #[inline]
    #[must_use] 
    pub fn root() -> Self {
        Self {
            segments: vec![PathSegment::Root],
            normalized_string: "$".to_string(),
        }
    }

    /// Get the canonical string representation
    #[inline]
    #[must_use] 
    pub fn as_str(&self) -> &str {
        &self.normalized_string
    }

    /// Get the path segments
    #[inline]
    #[must_use] 
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// Check if this is the root path ($)
    #[inline]
    #[must_use] 
    pub fn is_root(&self) -> bool {
        self.segments.len() == 1 && matches!(self.segments[0], PathSegment::Root)
    }

    /// Get the depth of this path (number of non-root segments)
    #[inline]
    #[must_use] 
    pub fn depth(&self) -> usize {
        self.segments.len().saturating_sub(1)
    }
}

impl fmt::Display for NormalizedPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.normalized_string)
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathSegment::Root => write!(f, "$"),
            PathSegment::Member(name) => write!(f, "['{name}']"),
            PathSegment::Index(index) => write!(f, "[{index}]"),
        }
    }
}
