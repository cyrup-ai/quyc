//! Expression parsing logic for filter expressions
//!
//! Contains parsing logic for logical operators, comparisons, and primary expressions
//! including literals, function calls, and basic `JSONPath` patterns.

use super::core::FilterParser;
use crate::jsonpath::{
    ast::{FilterExpression, FilterValue, LogicalOp},
    error::{JsonPathResult, invalid_expression_error},
    tokens::Token,
};

impl FilterParser<'_> {
    /// Parse logical OR expressions (lowest precedence)
    pub(super) fn parse_logical_or(&mut self) -> JsonPathResult<FilterExpression> {
        let mut left = self.parse_logical_and()?;

        while matches!(self.peek_token(), Some(Token::LogicalOr)) {
            self.consume_token();
            let right = self.parse_logical_and()?;
            left = FilterExpression::Logical {
                left: Box::new(left),
                operator: LogicalOp::Or,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse logical AND expressions
    pub(super) fn parse_logical_and(&mut self) -> JsonPathResult<FilterExpression> {
        let mut left = self.parse_comparison()?;

        while matches!(self.peek_token(), Some(Token::LogicalAnd)) {
            self.consume_token();
            let right = self.parse_comparison()?;
            left = FilterExpression::Logical {
                left: Box::new(left),
                operator: LogicalOp::And,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse comparison expressions
    pub(super) fn parse_comparison(&mut self) -> JsonPathResult<FilterExpression> {
        let left = self.parse_primary()?;

        if let Some(op_str) = self.parse_comparison_operator() {
            self.consume_token();
            let right = self.parse_primary()?;
            let operator = match op_str.as_str() {
                "==" => crate::jsonpath::ast::ComparisonOp::Equal,
                "!=" => crate::jsonpath::ast::ComparisonOp::NotEqual,
                "<" => crate::jsonpath::ast::ComparisonOp::Less,
                "<=" => crate::jsonpath::ast::ComparisonOp::LessEq,
                ">" => crate::jsonpath::ast::ComparisonOp::Greater,
                ">=" => crate::jsonpath::ast::ComparisonOp::GreaterEq,
                _ => {
                    return Err(crate::jsonpath::error::JsonPathError::new(
                        crate::jsonpath::error::ErrorKind::InvalidPath,
                        format!("Unknown comparison operator: {op_str}"),
                    ));
                }
            };
            Ok(FilterExpression::Comparison {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    /// Parse primary expressions (property access, literals, parentheses)
    pub(super) fn parse_primary(&mut self) -> JsonPathResult<FilterExpression> {
        match self.peek_token() {
            Some(Token::At) => {
                self.consume_token();
                self.parse_property_access()
            }
            Some(Token::String(s)) => {
                let value = s.clone();
                self.consume_token();
                Ok(FilterExpression::Literal {
                    value: FilterValue::String(value),
                })
            }
            Some(Token::Number(n)) => {
                let value = *n;
                self.consume_token();
                if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
                    // Safe cast: value is whole number within i64 range
                    #[allow(clippy::cast_possible_truncation)]
                    Ok(FilterExpression::Literal {
                        value: FilterValue::Integer(value as i64),
                    })
                } else {
                    Ok(FilterExpression::Literal {
                        value: FilterValue::Number(value),
                    })
                }
            }
            Some(Token::Integer(int_val)) => {
                let value = *int_val;
                self.consume_token();
                Ok(FilterExpression::Literal {
                    value: FilterValue::Integer(value),
                })
            }
            Some(Token::True) => {
                self.consume_token();
                Ok(FilterExpression::Literal {
                    value: FilterValue::Boolean(true),
                })
            }
            Some(Token::False) => {
                self.consume_token();
                Ok(FilterExpression::Literal {
                    value: FilterValue::Boolean(false),
                })
            }
            Some(Token::Null) => {
                self.consume_token();
                Ok(FilterExpression::Literal {
                    value: FilterValue::Null,
                })
            }
            Some(Token::LeftParen) => {
                self.consume_token();
                let expr = self.parse_logical_or()?;
                self.expect_token(Token::RightParen)?;
                Ok(expr)
            }
            Some(Token::Identifier(name)) => {
                // Check if this is a function call
                let name = name.clone();
                self.consume_token();

                if matches!(self.peek_token(), Some(Token::LeftParen)) {
                    self.consume_token(); // consume '('
                    let args = self.parse_function_arguments()?;
                    self.expect_token(Token::RightParen)?;

                    // RFC 9535: Validate function argument count
                    self.validate_function_arguments(&name, &args)?;

                    Ok(FilterExpression::Function { name, args })
                } else {
                    Err(invalid_expression_error(
                        self.input,
                        format!(
                            "unexpected identifier '{name}' - did you mean a function call?"
                        ),
                        Some(self.position),
                    ))
                }
            }
            Some(Token::Root) => self.parse_root_jsonpath(),
            _ => Err(invalid_expression_error(
                self.input,
                "expected property access, literal, or parenthesized expression",
                Some(self.position),
            )),
        }
    }

    // parse_function_arguments implementation is in functions.rs

    /// Parse `JSONPath` expression starting with $ or @
    fn parse_root_jsonpath(&mut self) -> JsonPathResult<FilterExpression> {
        // Parse JSONPath expression starting with $ or @
        let mut jsonpath_tokens = Vec::new();

        // Consume all tokens until we hit a delimiter or end
        while let Some(token) = self.peek_token() {
            match token {
                Token::Root
                | Token::At
                | Token::Dot
                | Token::DoubleDot
                | Token::LeftBracket
                | Token::RightBracket
                | Token::Star
                | Token::Identifier(_)
                | Token::Integer(_)
                | Token::String(_)
                | Token::Colon => {
                    if let Some(consumed_token) = self.consume_token() {
                        jsonpath_tokens.push(consumed_token);
                    }
                }
                _ => break, // Stop at other tokens (comma, right paren, operators, etc.)
            }
        }

        // Parse the collected tokens into selectors
        use crate::jsonpath::ast::JsonSelector;

        if jsonpath_tokens.is_empty() {
            return Err(invalid_expression_error(
                self.input,
                "empty JSONPath expression",
                Some(self.position),
            ));
        }

        // Convert tokens to selectors using a simple direct mapping
        let mut selectors = Vec::new();
        let mut i = 0;

        while i < jsonpath_tokens.len() {
            match &jsonpath_tokens[i] {
                Token::Root => {
                    selectors.push(JsonSelector::Root);
                    i += 1;
                }
                Token::At => {
                    // @ represents current node - in JSONPath context, this becomes root
                    selectors.push(JsonSelector::Root);
                    i += 1;
                }
                Token::DoubleDot => {
                    selectors.push(JsonSelector::RecursiveDescent);
                    i += 1;
                }
                Token::Star => {
                    selectors.push(JsonSelector::Wildcard);
                    i += 1;
                }
                Token::Identifier(name) => {
                    selectors.push(JsonSelector::Child {
                        name: name.clone(),
                        exact_match: true,
                    });
                    i += 1;
                }
                Token::Dot => {
                    // Skip dot tokens as they're structural
                    i += 1;
                }
                _ => {
                    // For now, skip other complex patterns
                    i += 1;
                }
            }
        }

        Ok(FilterExpression::JsonPath { selectors })
    }
}
