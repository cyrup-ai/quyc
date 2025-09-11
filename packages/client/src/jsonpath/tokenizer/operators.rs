//! Operator tokenization for comparison and logical operators
//!
//! Handles parsing of multi-character operators like ==, !=, <=, >=, &&, ||
//! with validation for single-character operator restrictions.

use super::core::ExpressionParser;
use crate::jsonpath::{
    error::{JsonPathResult, invalid_expression_error},
    tokens::Token,
};

/// Parse operator tokens (comparison and logical operators)
pub(crate) fn parse_operator(
    parser: &mut ExpressionParser,
    chars: &[char],
    i: usize,
) -> JsonPathResult<usize> {
    match chars[i] {
        '=' => {
            if i + 1 < chars.len() && chars[i + 1] == '=' {
                parser.tokens.push_back(Token::Equal);
                Ok(i + 1) // Skip next =
            } else {
                Err(invalid_expression_error(
                    &parser.input,
                    "single '=' not supported, use '==' for equality",
                    Some(i),
                ))
            }
        }
        '!' => {
            if i + 1 < chars.len() && chars[i + 1] == '=' {
                parser.tokens.push_back(Token::NotEqual);
                Ok(i + 1) // Skip next =
            } else {
                Err(invalid_expression_error(
                    &parser.input,
                    "single '!' not supported, use '!=' for inequality",
                    Some(i),
                ))
            }
        }
        '<' => {
            if i + 1 < chars.len() && chars[i + 1] == '=' {
                parser.tokens.push_back(Token::LessEq);
                Ok(i + 1) // Skip next =
            } else {
                parser.tokens.push_back(Token::Less);
                Ok(i)
            }
        }
        '>' => {
            if i + 1 < chars.len() && chars[i + 1] == '=' {
                parser.tokens.push_back(Token::GreaterEq);
                Ok(i + 1) // Skip next =
            } else {
                parser.tokens.push_back(Token::Greater);
                Ok(i)
            }
        }
        '&' => {
            if i + 1 < chars.len() && chars[i + 1] == '&' {
                parser.tokens.push_back(Token::LogicalAnd);
                Ok(i + 1) // Skip next &
            } else {
                Err(invalid_expression_error(
                    &parser.input,
                    "single '&' not supported, use '&&' for logical AND",
                    Some(i),
                ))
            }
        }
        '|' => {
            if i + 1 < chars.len() && chars[i + 1] == '|' {
                parser.tokens.push_back(Token::LogicalOr);
                Ok(i + 1) // Skip next |
            } else {
                Err(invalid_expression_error(
                    &parser.input,
                    "single '|' not supported, use '||' for logical OR",
                    Some(i),
                ))
            }
        }
        _ => Err(invalid_expression_error(
            &parser.input,
            format!("unexpected operator character '{}'", chars[i]),
            Some(i),
        )),
    }
}
