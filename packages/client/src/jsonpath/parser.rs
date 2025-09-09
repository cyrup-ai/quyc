//! High-performance JSONPath expression parser and compiler
//!
//! This module provides compile-time optimization of JSONPath expressions into
//! efficient runtime selectors. Supports the full JSONPath specification with
//! zero-allocation execution paths.

// Re-export main types for backward compatibility
pub use crate::jsonpath::ast::{
    ComparisonOp, ComplexityMetrics, FilterExpression, FilterValue, JsonSelector, LogicalOp,
};
pub use crate::jsonpath::compiler::JsonPathParser;
pub use crate::jsonpath::expression::JsonPathExpression;
pub use crate::jsonpath::normalized_paths::{NormalizedPath, NormalizedPathProcessor, PathSegment};
pub use crate::jsonpath::null_semantics::{NullSemantics, PropertyAccessResult};
pub use crate::jsonpath::safe_parsing::{
    SafeParsingContext, SafeStringBuffer, Utf8Handler, Utf8RecoveryStrategy,
};
pub use crate::jsonpath::tokenizer::ExpressionParser;
pub use crate::jsonpath::tokens::{Token, TokenMatcher};
// Re-export new RFC 9535 implementation modules
pub use crate::jsonpath::type_system::{FunctionSignature, FunctionType, TypeSystem, TypedValue};
