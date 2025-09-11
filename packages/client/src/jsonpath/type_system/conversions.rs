//! Type conversion rules and validation logic for RFC 9535
//!
//! Contains type conversion implementations and validation methods
//! for the `JSONPath` function type system.

use super::core::{FunctionType, TypeSystem, TypedValue};
use crate::jsonpath::{
    ast::{FilterExpression, FilterValue},
    error::{JsonPathResult, invalid_expression_error},
};

impl TypeSystem {
    /// RFC 9535 Section 2.4.2: Type Conversion Rules
    ///
    /// Performs type conversion according to RFC 9535 specifications:
    /// - `ValueType` can be converted to `LogicalType` using test expression conversion
    /// - `NodesType` can be converted to `ValueType` if nodelist has exactly one node
    #[inline]
    pub fn convert_type(
        value: TypedValue,
        target_type: FunctionType,
    ) -> JsonPathResult<TypedValue> {
        match (value, target_type) {
            // ValueType to LogicalType conversion (test expression conversion)
            (TypedValue::Value(json_val), FunctionType::LogicalType) => {
                let logical_result = Self::value_to_logical(&json_val);
                Ok(TypedValue::Logical(logical_result))
            }

            // NodesType to ValueType conversion (single node requirement)
            (TypedValue::Nodes(nodes), FunctionType::ValueType) => {
                if nodes.len() == 1 {
                    // Safe to access index 0 since we verified length == 1
                    let mut node_iter = nodes.into_iter();
                    match node_iter.next() {
                        Some(node) => Ok(TypedValue::Value(node)),
                        None => {
                            // This should never happen since len() == 1, but handle gracefully
                            Err(invalid_expression_error(
                                "",
                                "internal error: expected single node but iterator was empty",
                                None,
                            ))
                        }
                    }
                } else {
                    Err(invalid_expression_error(
                        "",
                        format!(
                            "NodesType to ValueType conversion requires exactly one node, found {}",
                            nodes.len()
                        ),
                        None,
                    ))
                }
            }

            // Same type conversions (no-op)
            (TypedValue::Value(val), FunctionType::ValueType) => Ok(TypedValue::Value(val)),
            (TypedValue::Logical(val), FunctionType::LogicalType) => Ok(TypedValue::Logical(val)),
            (TypedValue::Nodes(val), FunctionType::NodesType) => Ok(TypedValue::Nodes(val)),

            // Invalid conversions
            (TypedValue::Logical(_), FunctionType::ValueType) => Err(invalid_expression_error(
                "",
                "LogicalType cannot be converted to ValueType",
                None,
            )),
            (TypedValue::Logical(_), FunctionType::NodesType) => Err(invalid_expression_error(
                "",
                "LogicalType cannot be converted to NodesType",
                None,
            )),
            (TypedValue::Value(_), FunctionType::NodesType) => Err(invalid_expression_error(
                "",
                "ValueType cannot be converted to NodesType",
                None,
            )),
            (TypedValue::Nodes(_), FunctionType::LogicalType) => Err(invalid_expression_error(
                "",
                "NodesType cannot be converted to LogicalType",
                None,
            )),
        }
    }

    /// RFC 9535 Section 2.4.3: Well-Typedness Validation
    ///
    /// Validates that a function expression is well-typed according to RFC rules:
    /// 1. The function is known (defined in RFC 9535 or registered extension)
    /// 2. The function is applied to the correct number of arguments
    /// 3. All function arguments are well-typed
    /// 4. All function arguments can be converted to declared parameter types
    #[inline]
    pub fn validate_function_expression(
        function_name: &str,
        arguments: &[FilterExpression],
    ) -> JsonPathResult<super::core::FunctionSignature> {
        // 1. Check if function is known
        let signature = Self::get_function_signature(function_name).ok_or_else(|| {
            invalid_expression_error("", format!("unknown function: {function_name}"), None)
        })?;

        // 2. Check argument count
        if arguments.len() != signature.parameter_types.len() {
            return Err(invalid_expression_error(
                "",
                format!(
                    "function {} expects {} arguments, got {}",
                    function_name,
                    signature.parameter_types.len(),
                    arguments.len()
                ),
                None,
            ));
        }

        // 3. Validate each argument is well-typed (recursive validation)
        for (i, arg) in arguments.iter().enumerate() {
            let expected_type = &signature.parameter_types[i];
            Self::validate_expression_type(arg, expected_type)?;
        }

        Ok(signature)
    }

    /// Validate that an expression produces the expected type
    ///
    /// Performs static type analysis on filter expressions to ensure
    /// they can produce values of the expected type.
    #[inline]
    fn validate_expression_type(
        expr: &FilterExpression,
        expected_type: &FunctionType,
    ) -> JsonPathResult<()> {
        let actual_type = Self::infer_expression_type(expr)?;

        // Check if types match exactly or can be converted
        if actual_type == *expected_type {
            return Ok(());
        }

        // Check if conversion is possible
        match (&actual_type, expected_type) {
            // ValueType can be converted to LogicalType
            (FunctionType::ValueType, FunctionType::LogicalType) => Ok(()),
            // NodesType can be converted to ValueType (runtime check needed)
            (FunctionType::NodesType, FunctionType::ValueType) => Ok(()),
            _ => Err(invalid_expression_error(
                "",
                format!(
                    "type mismatch: expected {expected_type:?}, found {actual_type:?}"
                ),
                None,
            )),
        }
    }

    /// Infer the type that an expression will produce
    ///
    /// Performs static type inference on filter expressions to determine
    /// their return type without executing them.
    #[inline]
    fn infer_expression_type(expr: &FilterExpression) -> JsonPathResult<FunctionType> {
        match expr {
            FilterExpression::Current => Ok(FunctionType::ValueType),
            FilterExpression::Property { .. } => Ok(FunctionType::ValueType),
            FilterExpression::JsonPath { .. } => Ok(FunctionType::NodesType),
            FilterExpression::Literal { value } => match value {
                FilterValue::Boolean(_) => Ok(FunctionType::LogicalType),
                _ => Ok(FunctionType::ValueType),
            },
            FilterExpression::Comparison { .. } => Ok(FunctionType::LogicalType),
            FilterExpression::Logical { .. } => Ok(FunctionType::LogicalType),
            FilterExpression::Regex { .. } => Ok(FunctionType::LogicalType),
            FilterExpression::Function { name, args } => {
                // Validate the function expression and get its return type
                let signature = Self::validate_function_expression(name, args)?;
                Ok(signature.return_type)
            }
        }
    }
}
