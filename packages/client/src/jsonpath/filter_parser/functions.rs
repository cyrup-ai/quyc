//! Function parsing and validation for JSONPath filter expressions
//!
//! Handles parsing of function arguments and validation according to RFC 9535
//! specifications for standard JSONPath functions.

use super::core::FilterParser;
use crate::jsonpath::{
    ast::FilterExpression,
    error::{JsonPathResult, invalid_expression_error},
    tokens::Token,
};

impl<'a> FilterParser<'a> {
    /// Parse function arguments (comma-separated filter expressions)
    pub(super) fn parse_function_arguments(&mut self) -> JsonPathResult<Vec<FilterExpression>> {
        let mut args = Vec::new();

        // Handle empty argument list
        if matches!(self.peek_token(), Some(Token::RightParen)) {
            return Ok(args);
        }

        // Parse first argument
        args.push(self.parse_logical_or()?);

        // Parse remaining arguments
        while matches!(self.peek_token(), Some(Token::Comma)) {
            self.consume_token(); // consume comma
            args.push(self.parse_logical_or()?);
        }

        Ok(args)
    }

    /// Validate function arguments according to RFC 9535
    pub(super) fn validate_function_arguments(
        &self,
        function_name: &str,
        args: &[FilterExpression],
    ) -> JsonPathResult<()> {
        // Check for known functions with case sensitivity
        let expected_count = match function_name {
            "count" => 1,
            "length" => 1,
            "value" => 1,
            "match" => 2,
            "search" => 2,
            _ => {
                // Check if this might be a case-sensitivity error
                let lowercase_name = function_name.to_lowercase();
                if matches!(
                    lowercase_name.as_str(),
                    "count" | "length" | "value" | "match" | "search"
                ) {
                    return Err(invalid_expression_error(
                        self.input,
                        &format!(
                            "unknown function '{}' - did you mean '{}'? (function names are case-sensitive)",
                            function_name, lowercase_name
                        ),
                        Some(self.position),
                    ));
                }

                // Unknown function - let it pass for now (could be user-defined)
                return Ok(());
            }
        };

        if args.len() != expected_count {
            return Err(invalid_expression_error(
                self.input,
                &format!(
                    "function '{}' requires exactly {} argument{}, found {}",
                    function_name,
                    expected_count,
                    if expected_count == 1 { "" } else { "s" },
                    args.len()
                ),
                Some(self.position),
            ));
        }

        Ok(())
    }
}
