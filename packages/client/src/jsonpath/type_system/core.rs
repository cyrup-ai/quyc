//! Core types and enums for RFC 9535 `JSONPath` Function Type System
//!
//! Contains the fundamental types used in the `JSONPath` function type system
//! including `FunctionType`, `TypedValue`, and `FunctionSignature`.

/// RFC 9535 Function Expression Type System
///
/// Defines the three core types used in `JSONPath` function expressions
/// and provides type checking and conversion capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionType {
    /// `ValueType`: The type of any JSON value
    /// Can represent strings, numbers, booleans, null, arrays, or objects
    ValueType,

    /// `LogicalType`: The type of test or logical expression results
    /// Represents boolean true/false values from comparisons and logical operations
    LogicalType,

    /// `NodesType`: The type of a nodelist
    /// Represents the result of `JSONPath` expressions that select multiple nodes
    NodesType,
}

/// Type-safe wrapper for function expression values
///
/// Provides compile-time type safety and runtime type checking
/// for function arguments and return values.
#[derive(Debug, Clone)]
pub enum TypedValue {
    /// A JSON value with `ValueType`
    Value(serde_json::Value),

    /// A boolean result with `LogicalType`
    Logical(bool),

    /// A nodelist with `NodesType`
    Nodes(Vec<serde_json::Value>),
}

/// Function type signature definition
///
/// Defines the expected parameter types and return type for a function.
/// Used for compile-time type checking of function expressions.
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// Expected parameter types in order
    pub parameter_types: Vec<FunctionType>,
    /// Return type of the function
    pub return_type: FunctionType,
    /// Function name for error reporting
    pub name: String,
}

/// RFC 9535 Function Type System Implementation
pub struct TypeSystem;
