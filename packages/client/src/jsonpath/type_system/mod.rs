//! RFC 9535 `JSONPath` Function Type System (Section 2.4.1-2.4.3)
//!
//! Implements the type system for function expressions including:
//! - `ValueType`: The type of any JSON value
//! - `LogicalType`: The type of test or logical expression results (true/false)
//! - `NodesType`: The type of a nodelist
//! - Type conversion rules
//! - Well-typedness validation for function expressions

mod conversions;
mod core;
mod signatures;
mod utilities;

pub use core::{FunctionSignature, FunctionType, TypeSystem, TypedValue};


