//! Bracket notation selector parsing
//!
//! Handles parsing of bracket-notation selectors including array indices,
//! string properties, filters, wildcards, and union selectors.

use super::core::SelectorParser;
use crate::jsonpath::{
    ast::JsonSelector,
    error::{JsonPathResult, invalid_expression_error},
    filter_parser::FilterParser,
    tokens::Token,
};

/// Parse bracket-notation selector ([index], [start:end], [?expression])
pub fn parse_bracket_selector(parser: &mut SelectorParser) -> JsonPathResult<JsonSelector> {
    match parser.peek_token() {
        Some(Token::Star) => {
            parser.consume_token();
            parser.expect_token(Token::RightBracket)?;
            Ok(JsonSelector::Wildcard)
        }
        Some(Token::Question) => {
            parser.consume_token();
            let mut filter_parser = FilterParser::new(parser.tokens, parser.input, parser.position);
            let expression = filter_parser.parse_filter_expression()?;
            parser.expect_token(Token::RightBracket)?;
            Ok(JsonSelector::Filter { expression })
        }
        Some(Token::String(s)) => {
            let name = s.clone();
            parser.consume_token();

            // Check for comma-separated union selector
            if matches!(parser.peek_token(), Some(Token::Comma)) {
                parse_string_union_selector(parser, name)
            } else {
                parser.expect_token(Token::RightBracket)?;
                Ok(JsonSelector::Child {
                    name,
                    exact_match: true,
                })
            }
        }
        Some(Token::Integer(n)) => {
            let index = *n;
            parser.consume_token();
            super::slice::parse_index_or_slice(parser, index)
        }
        Some(Token::Colon) => super::slice::parse_slice_from_colon(parser),
        Some(Token::At) => Err(invalid_expression_error(
            parser.input,
            "current node identifier '@' is only valid within filter expressions [?...]",
            Some(parser.position),
        )),
        _ => Err(invalid_expression_error(
            parser.input,
            "expected index, slice, filter, string, or wildcard in brackets",
            Some(parser.position),
        )),
    }
}

/// Parse union selector starting with a string
fn parse_string_union_selector(
    parser: &mut SelectorParser,
    first_name: String,
) -> JsonPathResult<JsonSelector> {
    let mut selectors = vec![JsonSelector::Child {
        name: first_name,
        exact_match: true,
    }];

    while matches!(parser.peek_token(), Some(Token::Comma)) {
        parser.consume_token(); // consume comma

        match parser.peek_token() {
            Some(Token::String(s)) => {
                let name = s.clone();
                parser.consume_token();
                selectors.push(JsonSelector::Child {
                    name,
                    exact_match: true,
                });
            }
            Some(Token::Integer(n)) => {
                let index = *n;
                parser.consume_token();
                selectors.push(JsonSelector::Index {
                    index,
                    from_end: index < 0,
                });
            }
            Some(Token::Star) => {
                parser.consume_token();
                selectors.push(JsonSelector::Wildcard);
            }
            _ => {
                return Err(invalid_expression_error(
                    parser.input,
                    "expected string, integer, or '*' after comma in union selector",
                    Some(parser.position),
                ));
            }
        }
    }

    parser.expect_token(Token::RightBracket)?;
    Ok(JsonSelector::Union { selectors })
}
