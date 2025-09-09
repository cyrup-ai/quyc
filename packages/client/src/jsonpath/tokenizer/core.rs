//! Core tokenizer types and parser implementation
//!
//! Provides the main ExpressionParser struct and core parsing logic
//! for JSONPath expressions with RFC 9535 validation.

use std::collections::VecDeque;

use crate::jsonpath::{
    ast::JsonSelector,
    error::{JsonPathResult, invalid_expression_error},
    selector_parser::SelectorParser,
    tokens::Token,
};

/// Main expression parser that combines tokenization and parsing
pub struct ExpressionParser {
    pub(crate) input: String,
    pub(crate) tokens: VecDeque<Token>,
    pub(crate) position: usize,
}

impl ExpressionParser {
    /// Create new expression parser
    #[inline]
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            tokens: VecDeque::new(),
            position: 0,
        }
    }

    /// Parse complete JSONPath expression into selector chain
    pub fn parse(&mut self) -> JsonPathResult<Vec<JsonSelector>> {
        self.tokenize()?;

        // RFC 9535 validation: Check for invalid token sequences
        // Valid: [$, Dot, Identifier] or [$, LeftBracket, ...] or [$, DoubleDot, ...]
        // Invalid: [$, Identifier] (direct identifier after root without dot or bracket)
        // Note: Multiple root identifiers are allowed in complex expressions with functions
        let tokens_vec: Vec<_> = self.tokens.iter().collect();

        if self.tokens.len() >= 3 {
            if matches!(tokens_vec[0], Token::Root) && matches!(tokens_vec[1], Token::Identifier(_))
            {
                // This is the invalid pattern: $identifier (without dot or bracket)
                return Err(invalid_expression_error(
                    &self.input,
                    "property access requires '.' (dot) or '[]' (bracket) notation after root '$'",
                    Some(1), // Position of the identifier token
                ));
            }
        }

        let mut selectors = Vec::new();

        while !matches!(self.peek_token(), Some(Token::EOF) | None) {
            let mut selector_parser =
                SelectorParser::new(&mut self.tokens, &self.input, self.position);
            selectors.push(selector_parser.parse_selector()?);
        }

        Ok(selectors)
    }

    /// Tokenize the input expression
    pub(crate) fn tokenize(&mut self) -> JsonPathResult<()> {
        use super::{characters, numbers, operators, strings};

        let chars: Vec<char> = self.input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                ' ' | '\t' | '\n' | '\r' => {
                    // Skip whitespace
                }
                '\'' | '"' => {
                    i = strings::parse_string_literal(self, &chars, i)?;
                }
                c if c.is_ascii_digit() || c == '-' => {
                    i = numbers::parse_number_literal(self, &chars, i)?;
                }
                '=' | '!' | '<' | '>' | '&' | '|' => {
                    i = operators::parse_operator(self, &chars, i)?;
                }
                _ => {
                    i = characters::parse_character_token(self, &chars, i)?;
                }
            }
            i += 1;
        }

        self.tokens.push_back(Token::EOF);
        Ok(())
    }

    /// Peek at next token without consuming
    #[inline]
    pub(crate) fn peek_token(&self) -> Option<&Token> {
        self.tokens.front()
    }
}
