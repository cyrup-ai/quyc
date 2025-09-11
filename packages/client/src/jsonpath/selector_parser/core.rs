//! Core selector parser structure and main parsing logic
//!
//! Contains the main `SelectorParser` struct and the primary `parse_selector` method
//! that dispatches to specialized parsing functions based on token type.

use std::collections::VecDeque;

use crate::jsonpath::{
    ast::JsonSelector,
    error::{JsonPathResult, invalid_expression_error},
    tokens::Token,
};

/// Parser for individual `JSONPath` selectors
pub struct SelectorParser<'a> {
    pub(super) tokens: &'a mut VecDeque<Token>,
    pub(super) input: &'a str,
    pub(super) position: usize,
}

impl<'a> SelectorParser<'a> {
    /// Create new selector parser
    #[inline]
    pub fn new(tokens: &'a mut VecDeque<Token>, input: &'a str, position: usize) -> Self {
        Self {
            tokens,
            input,
            position,
        }
    }

    /// Parse a single `JSONPath` selector
    pub fn parse_selector(&mut self) -> JsonPathResult<JsonSelector> {
        match self.peek_token() {
            Some(Token::Root) => {
                self.consume_token();
                Ok(JsonSelector::Root)
            }
            Some(Token::Dot) => {
                self.consume_token();
                super::dot::parse_dot_selector(self)
            }
            Some(Token::DoubleDot) => {
                self.consume_token();
                Ok(JsonSelector::RecursiveDescent)
            }
            Some(Token::Star) => {
                self.consume_token();
                Ok(JsonSelector::Wildcard)
            }
            Some(Token::LeftBracket) => {
                self.consume_token();
                super::bracket::parse_bracket_selector(self)
            }
            Some(Token::Identifier(name)) => {
                // Handle standalone identifiers (e.g., 'author' in '$..author')
                let name = name.clone();
                self.consume_token();
                Ok(JsonSelector::Child {
                    name,
                    exact_match: true,
                })
            }
            Some(Token::At) => Err(invalid_expression_error(
                self.input,
                "current node identifier '@' is only valid within filter expressions [?...]",
                Some(self.position),
            )),
            _ => Err(invalid_expression_error(
                self.input,
                "expected selector (.property, [index], identifier, or [expression])",
                Some(self.position),
            )),
        }
    }

    /// Peek at next token without consuming
    #[inline]
    pub(super) fn peek_token(&self) -> Option<&Token> {
        self.tokens.front()
    }

    /// Consume and return next token
    #[inline]
    pub(super) fn consume_token(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    /// Expect specific token and consume it
    pub(super) fn expect_token(&mut self, expected: Token) -> JsonPathResult<()> {
        let token = self.consume_token();
        match token {
            Some(actual) if self.tokens_match(&actual, &expected) => Ok(()),
            Some(actual) => Err(invalid_expression_error(
                self.input,
                format!("expected {expected:?}, found {actual:?}"),
                Some(self.position),
            )),
            None => Err(invalid_expression_error(
                self.input,
                format!("expected {expected:?}, found end of input"),
                Some(self.position),
            )),
        }
    }

    /// Check if two tokens match (handles different variants with same discriminant)
    #[inline]
    fn tokens_match(&self, actual: &Token, expected: &Token) -> bool {
        use crate::jsonpath::tokens::TokenMatcher;
        TokenMatcher::tokens_match(actual, expected)
    }
}
