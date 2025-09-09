//! String literal tokenization with escape sequence handling
//!
//! Handles parsing of quoted string literals including escape sequences,
//! Unicode processing, and UTF-16 surrogate pair handling.

use super::core::ExpressionParser;
use crate::jsonpath::{
    error::{JsonPathResult, invalid_expression_error},
    tokens::Token,
};

/// Parse string literal with quote handling and escape sequences
pub(crate) fn parse_string_literal(
    parser: &mut ExpressionParser,
    chars: &[char],
    mut i: usize,
) -> JsonPathResult<usize> {
    let quote = chars[i];
    i += 1; // Skip opening quote
    let start = i;
    let mut string_value = String::new();

    while i < chars.len() {
        if chars[i] == quote {
            // Found closing quote
            break;
        } else if chars[i] == '\\' && i + 1 < chars.len() {
            // Handle escape sequence
            i += 1; // Skip backslash
            match chars[i] {
                '"' => string_value.push('"'),
                '\'' => string_value.push('\''),
                '\\' => string_value.push('\\'),
                '/' => string_value.push('/'),
                'b' => string_value.push('\u{0008}'), // Backspace
                'f' => string_value.push('\u{000C}'), // Form feed
                'n' => string_value.push('\n'),       // Newline
                'r' => string_value.push('\r'),       // Carriage return
                't' => string_value.push('\t'),       // Tab
                'u' => {
                    i = parse_unicode_escape(parser, chars, i, &mut string_value)?;
                }
                _ => {
                    return Err(invalid_expression_error(
                        &parser.input,
                        "invalid escape sequence",
                        Some(i),
                    ));
                }
            }
        } else {
            // Regular character
            string_value.push(chars[i]);
        }
        i += 1;
    }

    if i >= chars.len() {
        return Err(invalid_expression_error(
            &parser.input,
            "unterminated string literal",
            Some(start),
        ));
    }

    parser.tokens.push_back(Token::String(string_value));
    Ok(i)
}

/// Parse Unicode escape sequence \uXXXX with surrogate pair support
fn parse_unicode_escape(
    parser: &ExpressionParser,
    chars: &[char],
    mut i: usize,
    string_value: &mut String,
) -> JsonPathResult<usize> {
    // Unicode escape sequence \uXXXX
    if i + 4 >= chars.len() {
        return Err(invalid_expression_error(
            &parser.input,
            "incomplete unicode escape sequence",
            Some(i),
        ));
    }
    let hex_digits: String = chars[i + 1..i + 5].iter().collect();
    if let Ok(code_point) = u32::from_str_radix(&hex_digits, 16) {
        // Handle Unicode surrogate pairs (UTF-16)
        if (0xD800..=0xDBFF).contains(&code_point) {
            // High surrogate - look for low surrogate
            if i + 10 < chars.len() && chars[i + 5] == '\\' && chars[i + 6] == 'u' {
                let low_hex: String = chars[i + 7..i + 11].iter().collect();
                if let Ok(low_surrogate) = u32::from_str_radix(&low_hex, 16) {
                    if (0xDC00..=0xDFFF).contains(&low_surrogate) {
                        // Valid surrogate pair - convert to Unicode scalar
                        let high = code_point - 0xD800;
                        let low = low_surrogate - 0xDC00;
                        let unicode_scalar = 0x10000 + (high << 10) + low;
                        if let Some(unicode_char) = char::from_u32(unicode_scalar) {
                            string_value.push(unicode_char);
                            i += 10; // Skip both \uXXXX sequences
                        } else {
                            return Err(invalid_expression_error(
                                &parser.input,
                                "invalid surrogate pair result",
                                Some(i),
                            ));
                        }
                    } else {
                        return Err(invalid_expression_error(
                            &parser.input,
                            "high surrogate not followed by valid low surrogate",
                            Some(i),
                        ));
                    }
                } else {
                    return Err(invalid_expression_error(
                        &parser.input,
                        "invalid low surrogate hex digits",
                        Some(i),
                    ));
                }
            } else {
                return Err(invalid_expression_error(
                    &parser.input,
                    "high surrogate not followed by low surrogate escape sequence",
                    Some(i),
                ));
            }
        } else if (0xDC00..=0xDFFF).contains(&code_point) {
            // Low surrogate without high surrogate is invalid
            return Err(invalid_expression_error(
                &parser.input,
                "low surrogate without preceding high surrogate",
                Some(i),
            ));
        } else if let Some(unicode_char) = char::from_u32(code_point) {
            // Regular Unicode character (not surrogate)
            string_value.push(unicode_char);
            i += 4; // Skip the 4 hex digits
        } else {
            return Err(invalid_expression_error(
                &parser.input,
                "invalid unicode code point",
                Some(i),
            ));
        }
    } else {
        return Err(invalid_expression_error(
            &parser.input,
            "invalid unicode escape sequence",
            Some(i),
        ));
    }
    Ok(i)
}
