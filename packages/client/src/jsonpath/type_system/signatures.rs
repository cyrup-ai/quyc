//! Function signature definitions for RFC 9535 built-in functions
//!
//! Contains the type signatures for all built-in JSONPath functions
//! as specified in RFC 9535.

use super::core::{FunctionSignature, FunctionType, TypeSystem};

impl TypeSystem {
    /// Get function signature for built-in RFC 9535 functions
    ///
    /// Returns the type signature for the specified function name,
    /// or None if the function is not a built-in RFC 9535 function.
    #[inline]
    pub fn get_function_signature(function_name: &str) -> Option<FunctionSignature> {
        match function_name {
            "length" => Some(FunctionSignature {
                parameter_types: vec![FunctionType::ValueType],
                return_type: FunctionType::ValueType,
                name: "length".to_string(),
            }),
            "count" => Some(FunctionSignature {
                parameter_types: vec![FunctionType::NodesType],
                return_type: FunctionType::ValueType,
                name: "count".to_string(),
            }),
            "match" => Some(FunctionSignature {
                parameter_types: vec![FunctionType::ValueType, FunctionType::ValueType],
                return_type: FunctionType::LogicalType,
                name: "match".to_string(),
            }),
            "search" => Some(FunctionSignature {
                parameter_types: vec![FunctionType::ValueType, FunctionType::ValueType],
                return_type: FunctionType::LogicalType,
                name: "search".to_string(),
            }),
            "value" => Some(FunctionSignature {
                parameter_types: vec![FunctionType::NodesType],
                return_type: FunctionType::ValueType,
                name: "value".to_string(),
            }),
            _ => None,
        }
    }
}
