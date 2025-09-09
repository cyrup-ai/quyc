//! Character and identifier tokenization
//!
//! Handles parsing of single-character tokens, identifiers, keywords,
//! and special character sequences like double dots.

use super::core::ExpressionParser;
use crate::jsonpath::{
    error::{JsonPathResult, invalid_expression_error},
    tokens::Token,
};

/// Parse character tokens, identifiers, and keywords
pub(crate) fn parse_character_token(
    parser: &mut ExpressionParser,
    chars: &[char],
    i: usize,
) -> JsonPathResult<usize> {
    match chars[i] {
        '$' => {
            parser.tokens.push_back(Token::Root);
            Ok(i)
        }
        '.' => {
            // Check for double dot (..)
            if i + 1 < chars.len() && chars[i + 1] == '.' {
                // Check for invalid triple dot (...)
                if i + 2 < chars.len() && chars[i + 2] == '.' {
                    return Err(invalid_expression_error(
                        &parser.input,
                        "triple dot '...' is invalid, use '..' for recursive descent",
                        Some(i),
                    ));
                }
                parser.tokens.push_back(Token::DoubleDot);
                Ok(i + 1) // Skip the second dot
            } else {
                parser.tokens.push_back(Token::Dot);
                Ok(i)
            }
        }
        '[' => {
            parser.tokens.push_back(Token::LeftBracket);
            Ok(i)
        }
        ']' => {
            parser.tokens.push_back(Token::RightBracket);
            Ok(i)
        }
        '(' => {
            parser.tokens.push_back(Token::LeftParen);
            Ok(i)
        }
        ')' => {
            parser.tokens.push_back(Token::RightParen);
            Ok(i)
        }
        ',' => {
            parser.tokens.push_back(Token::Comma);
            Ok(i)
        }
        ':' => {
            parser.tokens.push_back(Token::Colon);
            Ok(i)
        }
        '?' => {
            parser.tokens.push_back(Token::Question);
            Ok(i)
        }
        '@' => {
            parser.tokens.push_back(Token::At);
            Ok(i)
        }
        '*' => {
            parser.tokens.push_back(Token::Star);
            Ok(i)
        }
        c if c.is_alphabetic() || c == '_' => parse_identifier(parser, chars, i),
        _ => Err(invalid_expression_error(
            &parser.input,
            &format!("unexpected character '{}'", chars[i]),
            Some(i),
        )),
    }
}

/// Parse identifier or keyword token
fn parse_identifier(
    parser: &mut ExpressionParser,
    chars: &[char],
    mut i: usize,
) -> JsonPathResult<usize> {
    let start = i;
    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
        i += 1;
    }
    let identifier: String = chars[start..i].iter().collect();

    // Check for reserved keywords
    let token = match identifier.as_str() {
        "true" => Token::True,
        "false" => Token::False,
        "null" => Token::Null,
        _ => Token::Identifier(identifier),
    };

    parser.tokens.push_back(token);
    Ok(i.saturating_sub(1)) // Adjust for loop increment
}
