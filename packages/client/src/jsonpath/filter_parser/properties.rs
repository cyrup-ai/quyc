//! Property access parsing for JSONPath filter expressions
//!
//! Handles parsing of property access patterns including current node (@),
//! property chains (@.prop1.prop2), and complex JSONPath selectors.

use super::core::FilterParser;
use crate::jsonpath::{
    ast::{FilterExpression, JsonSelector},
    error::{JsonPathResult, invalid_expression_error},
    tokens::Token,
};

impl<'a> FilterParser<'a> {
    /// Parse property access after @ token
    pub(super) fn parse_property_access(&mut self) -> JsonPathResult<FilterExpression> {
        // Check for just @ (current node)
        if !matches!(self.peek_token(), Some(Token::Dot | Token::DoubleDot)) {
            return Ok(FilterExpression::Current);
        }

        // Handle property access patterns like @.author
        // After @, check for property access chain (e.g., @.author.length)
        if matches!(self.peek_token(), Some(Token::Dot)) {
            self.consume_token(); // consume the first dot

            let mut path = Vec::new();

            // Parse property chain: @.prop1.prop2.prop3...
            loop {
                if let Some(Token::Identifier(name)) = self.peek_token() {
                    let name = name.clone();
                    self.consume_token();
                    path.push(name);

                    // Check if there's another dot for chaining
                    if matches!(self.peek_token(), Some(Token::Dot)) {
                        self.consume_token(); // consume the dot and continue
                    } else {
                        // No more dots, end of property chain
                        break;
                    }
                } else {
                    return Err(invalid_expression_error(
                        self.input,
                        "expected property name after '.'",
                        Some(self.position),
                    ));
                }
            }

            if path.is_empty() {
                return Err(invalid_expression_error(
                    self.input,
                    "expected property name after '.'",
                    Some(self.position),
                ));
            }

            // Return property with full chain path
            return Ok(FilterExpression::Property { path });
        }

        // Handle complex JSONPath patterns like @..book, @.*, etc.
        let mut selectors = Vec::new();

        // @ represents current node in filter context
        selectors.push(JsonSelector::Root);

        // Parse the remaining tokens as JSONPath selectors
        while let Some(token) = self.peek_token() {
            match token {
                Token::Dot => {
                    self.consume_token();
                    // After dot, expect identifier
                    if let Some(Token::Identifier(name)) = self.peek_token() {
                        let name = name.clone();
                        self.consume_token();
                        selectors.push(JsonSelector::Child {
                            name,
                            exact_match: true,
                        });
                    } else {
                        return Err(invalid_expression_error(
                            self.input,
                            "expected property name after '.'",
                            Some(self.position),
                        ));
                    }
                }
                Token::DoubleDot => {
                    self.consume_token();
                    selectors.push(JsonSelector::RecursiveDescent);
                }
                Token::Star => {
                    self.consume_token();
                    selectors.push(JsonSelector::Wildcard);
                }
                Token::Identifier(name) => {
                    // Bare identifier (should not happen after @ but handle gracefully)
                    let name = name.clone();
                    self.consume_token();
                    selectors.push(JsonSelector::Child {
                        name,
                        exact_match: true,
                    });
                }
                _ => break, // Stop at other tokens
            }
        }

        // Return appropriate expression type
        if selectors.len() == 1 {
            Ok(FilterExpression::Current) // Just @
        } else {
            Ok(FilterExpression::JsonPath { selectors })
        }
    }
}
