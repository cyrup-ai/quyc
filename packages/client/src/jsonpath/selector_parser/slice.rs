//! Slice and index parsing for array selectors
//!
//! Handles parsing of array index and slice notation including
//! RFC 9535 compliant slice expressions with start, end, and step values.

use super::core::SelectorParser;
use crate::jsonpath::{
    ast::JsonSelector,
    error::{JsonPathResult, invalid_expression_error},
    tokens::Token,
};

/// Parse index or slice notation after initial integer
pub fn parse_index_or_slice(
    parser: &mut SelectorParser,
    start: i64,
) -> JsonPathResult<JsonSelector> {
    match parser.peek_token() {
        Some(Token::RightBracket) => {
            parser.consume_token();
            Ok(JsonSelector::Index {
                index: start,
                from_end: start < 0,
            })
        }
        Some(Token::Colon) => parse_slice_from_start(parser, start),
        Some(Token::Comma) => parse_integer_union_selector(parser, start),
        _ => Err(invalid_expression_error(
            parser.input,
            "expected ']', ':', or ',' after index",
            Some(parser.position),
        )),
    }
}

/// Parse union selector starting with integer
fn parse_integer_union_selector(
    parser: &mut SelectorParser,
    first_index: i64,
) -> JsonPathResult<JsonSelector> {
    let mut selectors = vec![JsonSelector::Index {
        index: first_index,
        from_end: first_index < 0,
    }];

    while matches!(parser.peek_token(), Some(Token::Comma)) {
        parser.consume_token(); // consume comma

        match parser.peek_token() {
            Some(Token::Integer(n)) => {
                let index = *n;
                parser.consume_token();
                selectors.push(JsonSelector::Index {
                    index,
                    from_end: index < 0,
                });
            }
            Some(Token::String(s)) => {
                let name = s.clone();
                parser.consume_token();
                selectors.push(JsonSelector::Child {
                    name,
                    exact_match: true,
                });
            }
            Some(Token::Star) => {
                parser.consume_token();
                selectors.push(JsonSelector::Wildcard);
            }
            _ => {
                return Err(invalid_expression_error(
                    parser.input,
                    "expected integer, string, or '*' after comma in union selector",
                    Some(parser.position),
                ));
            }
        }
    }

    parser.expect_token(Token::RightBracket)?;
    Ok(JsonSelector::Union { selectors })
}

/// Parse slice notation starting with integer (e.g., [1:5])
pub fn parse_slice_from_start(
    parser: &mut SelectorParser,
    start: i64,
) -> JsonPathResult<JsonSelector> {
    parser.consume_token(); // consume colon

    // Parse end index
    let end = if let Some(Token::Integer(n)) = parser.peek_token() {
        let n = *n;
        parser.consume_token();
        Some(n)
    } else if matches!(parser.peek_token(), Some(Token::RightBracket)) {
        None // Open-ended slice like [1:]
    } else if matches!(parser.peek_token(), Some(Token::Colon)) {
        None // Empty end in patterns like [1::2]
    } else {
        None
    };

    // Parse optional step
    let step = parse_optional_step(parser)?;

    parser.expect_token(Token::RightBracket)?;
    Ok(JsonSelector::Slice {
        start: Some(start),
        end,
        step,
    })
}

/// Parse slice notation starting with colon (e.g., [:5])
pub fn parse_slice_from_colon(parser: &mut SelectorParser) -> JsonPathResult<JsonSelector> {
    parser.consume_token(); // consume colon

    // Parse end index
    let end = if let Some(Token::Integer(n)) = parser.peek_token() {
        let n = *n;
        parser.consume_token();
        Some(n)
    } else if matches!(parser.peek_token(), Some(Token::Colon)) {
        None // Empty end in patterns like [::2]
    } else {
        None
    };

    // Parse optional step
    let step = parse_optional_step(parser)?;

    parser.expect_token(Token::RightBracket)?;
    Ok(JsonSelector::Slice {
        start: None,
        end,
        step,
    })
}

/// Parse optional step value in slice notation
fn parse_optional_step(parser: &mut SelectorParser) -> JsonPathResult<Option<i64>> {
    if matches!(parser.peek_token(), Some(Token::Colon)) {
        parser.consume_token(); // consume second colon
        // After second colon, step is REQUIRED per RFC 9535
        if let Some(Token::Integer(n)) = parser.peek_token() {
            let n = *n;
            parser.consume_token();
            // RFC 9535: step must not be zero
            if n == 0 {
                return Err(invalid_expression_error(
                    parser.input,
                    "step value cannot be zero in slice expression",
                    Some(parser.position),
                ));
            }
            Ok(Some(n))
        } else {
            Err(invalid_expression_error(
                parser.input,
                "step value required after second colon in slice",
                Some(parser.position),
            ))
        }
    } else {
        Ok(None)
    }
}
