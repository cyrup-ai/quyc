//! Core filter parser structure and initialization
//!
//! Contains the main `FilterParser` struct and basic initialization logic
//! for parsing `JSONPath` filter expressions.

use std::collections::VecDeque;

use crate::jsonpath::{ast::FilterExpression, error::JsonPathResult, tokens::Token};

/// Parser for `JSONPath` filter expressions
pub struct FilterParser<'a> {
    pub(super) tokens: &'a mut VecDeque<Token>,
    pub(super) input: &'a str,
    pub(super) position: usize,
}

impl<'a> FilterParser<'a> {
    /// Create new filter parser
    #[inline]
    pub fn new(tokens: &'a mut VecDeque<Token>, input: &'a str, position: usize) -> Self {
        Self {
            tokens,
            input,
            position,
        }
    }

    /// Parse comparison operator from tokens
    #[inline]
    pub fn parse_comparison_operator(&mut self) -> Option<String> {
        match self.tokens.front() {
            Some(Token::Equal) => {
                self.tokens.pop_front();
                Some("==".to_string())
            }
            Some(Token::NotEqual) => {
                self.tokens.pop_front();
                Some("!=".to_string())
            }
            Some(Token::Less) => {
                self.tokens.pop_front();
                Some("<".to_string())
            }
            Some(Token::LessEq) => {
                self.tokens.pop_front();
                Some("<=".to_string())
            }
            Some(Token::Greater) => {
                self.tokens.pop_front();
                Some(">".to_string())
            }
            Some(Token::GreaterEq) => {
                self.tokens.pop_front();
                Some(">=".to_string())
            }
            _ => None,
        }
    }

    /// Expect a specific token and consume it
    #[inline]
    pub fn expect_token(&mut self, expected: Token) -> JsonPathResult<()> {
        match self.tokens.pop_front() {
            Some(token) if std::mem::discriminant(&token) == std::mem::discriminant(&expected) => {
                Ok(())
            }
            Some(token) => Err(crate::jsonpath::error::JsonPathError::new(
                crate::jsonpath::error::ErrorKind::InvalidPath,
                format!("Expected {expected:?}, found {token:?}"),
            )),
            None => Err(crate::jsonpath::error::JsonPathError::new(
                crate::jsonpath::error::ErrorKind::InvalidPath,
                format!("Expected {expected:?}, but reached end of input"),
            )),
        }
    }

    /// Parse complete filter expression
    #[inline]
    pub fn parse_filter_expression(&mut self) -> JsonPathResult<FilterExpression> {
        self.parse_logical_or()
    }

    /// Consume the next token from the token stream
    #[inline]
    pub fn consume_token(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    /// Peek at the next token without consuming it
    #[inline]
    #[must_use] 
    pub fn peek_token(&self) -> Option<&Token> {
        self.tokens.front()
    }

    // parse_logical_or implementation is in expressions.rs
}
