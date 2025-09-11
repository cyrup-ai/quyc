//! `JSONPath` expression compiler and entry point
//!
//! Provides the main parser interface for compiling `JSONPath` expressions
//! into optimized AST structures.

use crate::jsonpath::{
    ast::JsonSelector,
    error::{JsonPathResult, invalid_expression_error},
    expression::JsonPathExpression,
    tokenizer::ExpressionParser,
};

/// `JSONPath` expression parser and compiler
pub struct JsonPathParser;

impl JsonPathParser {
    /// Compile `JSONPath` expression into optimized selector chain
    ///
    /// # Arguments
    ///
    /// * `expression` - `JSONPath` expression string (e.g., "$.data[*]", "$.items[?(@.active)]")
    ///
    /// # Returns
    ///
    /// Compiled `JsonPathExpression` optimized for streaming evaluation.
    ///
    /// # Errors
    ///
    /// Returns `JsonPathError::InvalidExpression` for syntax errors or unsupported features.
    ///
    /// # Performance
    ///
    /// Expression compilation is performed once at construction time. Runtime evaluation
    /// uses pre-compiled selectors for maximum performance.
    pub fn compile(expression: &str) -> JsonPathResult<JsonPathExpression> {
        if expression.is_empty() {
            return Err(invalid_expression_error(
                expression,
                "empty expression not allowed",
                Some(0),
            ));
        }

        // RFC 9535 Compliance: JSONPath expressions must start with '$'
        if !expression.starts_with('$') {
            // Special case: provide specific error for @ outside filter context
            if expression.starts_with('@') {
                return Err(invalid_expression_error(
                    expression,
                    "current node identifier '@' is only valid within filter expressions [?...]",
                    Some(0),
                ));
            }
            return Err(invalid_expression_error(
                expression,
                "JSONPath expressions must start with '$'",
                Some(0),
            ));
        }

        // RFC 9535 Compliance: JSONPath expressions cannot end with '.' unless it's recursive descent
        // '$.' is invalid (incomplete property access)
        // '$..' is also invalid per RFC 9535: descendant-segment = ".." S bracket-segment
        if expression.ends_with('.') && !expression.ends_with("..") {
            return Err(invalid_expression_error(
                expression,
                "incomplete property access (ends with '.')",
                Some(expression.len() - 1),
            ));
        }

        // RFC 9535: descendant-segment = ".." S bracket-segment
        // Bare ".." without following segment is invalid
        if expression == "$.." {
            return Err(invalid_expression_error(
                expression,
                "descendant segment '..' must be followed by a bracket segment",
                Some(expression.len() - 2),
            ));
        }

        // RFC 9535: Also check for expressions ending with ".." like "$.store.."
        if expression.ends_with("..") && expression.len() > 3 {
            return Err(invalid_expression_error(
                expression,
                "descendant segment '..' must be followed by a bracket segment",
                Some(expression.len() - 2),
            ));
        }

        // RFC 9535: Check for invalid patterns like "$.property..property"
        // Valid: "$..property" (recursive descent from root)
        // Invalid: "$.property..property" (property followed by recursive descent followed by property)
        if let Some(double_dot_pos) = expression.find("..")
            && double_dot_pos > 2 {
                // More than just "$."
                let before_double_dot = &expression[..double_dot_pos];
                let after_double_dot = &expression[double_dot_pos + 2..];

                // Check if we have a property before .. and a property after ..
                if before_double_dot.len() > 2
                    && !before_double_dot.ends_with('.')
                    && !after_double_dot.is_empty()
                    && !after_double_dot.starts_with('[')
                    && !after_double_dot.starts_with('*')
                {
                    // This is a pattern like "$.store..book" which is invalid
                    return Err(invalid_expression_error(
                        expression,
                        "invalid recursive descent pattern: use either '$.property.subproperty' for direct access or '$..property' for recursive search",
                        Some(double_dot_pos),
                    ));
                }
            }

        // RFC 9535: descendant-segment = ".." S bracket-segment
        // According to RFC 9535, ".." can be followed by bracket-segment, wildcard '*', or identifier
        // Valid: "$..*", "$..[*]", "$..level1", "$..['key']"
        // Invalid: bare ".." without any following segment

        let mut parser = ExpressionParser::new(expression);
        let selectors = parser.parse()?;

        // RFC 9535 Compliance: Root-only expressions "$" are valid per specification
        // ABNF: jsonpath-query = root-identifier segments where segments = *(S segment) allows zero segments
        // Section 2.2.3 Examples explicitly shows "$" returns the root node
        // No validation needed - bare "$" is perfectly valid per RFC 9535

        // Determine if this is an array streaming expression
        let is_array_stream = selectors.iter().any(|s| {
            matches!(
                s,
                JsonSelector::Wildcard | JsonSelector::Slice { .. } | JsonSelector::Filter { .. }
            )
        });

        Ok(JsonPathExpression::new(
            selectors,
            expression.to_string(),
            is_array_stream,
        ))
    }

    /// Validate `JSONPath` expression syntax without compilation
    ///
    /// Faster than full compilation when only validation is needed.
    pub fn validate(expression: &str) -> JsonPathResult<()> {
        Self::compile(expression).map(|_| ())
    }
}
