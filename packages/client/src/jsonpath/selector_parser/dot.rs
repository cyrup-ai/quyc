//! Dot notation selector parsing
//!
//! Handles parsing of dot-notation selectors including property access,
//! recursive descent (..), and wildcard (.*) patterns.

use super::core::SelectorParser;
use crate::jsonpath::{
    ast::JsonSelector,
    error::{JsonPathResult, invalid_expression_error},
    tokens::Token,
};

/// Parse dot-notation selector (.property or ..)
pub fn parse_dot_selector(parser: &mut SelectorParser) -> JsonPathResult<JsonSelector> {
    match parser.peek_token() {
        Some(Token::Dot) => {
            parser.consume_token();
            Ok(JsonSelector::RecursiveDescent)
        }
        Some(Token::Star) => {
            parser.consume_token();
            Ok(JsonSelector::Wildcard)
        }
        Some(Token::Identifier(name)) => {
            let name = name.clone();
            parser.consume_token();
            Ok(JsonSelector::Child {
                name,
                exact_match: true,
            })
        }
        Some(Token::At) => Err(invalid_expression_error(
            parser.input,
            "current node identifier '@' is only valid within filter expressions [?...]",
            Some(parser.position),
        )),
        _ => Err(invalid_expression_error(
            parser.input,
            "expected property name, '..' (recursive descent), or '*' (wildcard) after '.'",
            Some(parser.position),
        )),
    }
}
