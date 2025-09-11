//! Number literal tokenization with RFC 9535 compliance
//!
//! Handles parsing of integer and floating-point literals with
//! validation for leading zeros and negative zero restrictions.

use super::core::ExpressionParser;
use crate::jsonpath::{
    error::{JsonPathResult, invalid_expression_error},
    tokens::Token,
};

/// Parse number literal (integer or float) with RFC 9535 validation
pub(crate) fn parse_number_literal(
    parser: &mut ExpressionParser,
    chars: &[char],
    mut i: usize,
) -> JsonPathResult<usize> {
    let start = i;
    let c = chars[i];

    if c == '-' {
        i += 1;
    }

    // RFC 9535: integers cannot have leading zeros (except for "0" itself)
    let digit_start = i;
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }

    // Validate no leading zeros for multi-digit integers
    if i > digit_start + 1 && chars[digit_start] == '0' {
        return Err(invalid_expression_error(
            &parser.input,
            "integers cannot have leading zeros",
            Some(digit_start),
        ));
    }

    // RFC 9535: negative zero "-0" is invalid per grammar: int = "0" / (["−"] (non-zero-digit *DIGIT))
    if start < digit_start && chars[digit_start] == '0' && i == digit_start + 1 {
        return Err(invalid_expression_error(
            &parser.input,
            "negative zero is not allowed",
            Some(start),
        ));
    }

    // Check for decimal point
    let mut is_float = false;
    if i < chars.len() && chars[i] == '.' {
        is_float = true;
        i += 1; // Skip decimal point
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
    }

    let number_str: String = chars[start..i].iter().collect();
    if is_float {
        if let Ok(float_val) = number_str.parse::<f64>() {
            parser.tokens.push_back(Token::Number(float_val));
        } else {
            return Err(invalid_expression_error(
                &parser.input,
                "invalid floating point number format",
                Some(start),
            ));
        }
    } else if let Ok(int_val) = number_str.parse::<i64>() {
        parser.tokens.push_back(Token::Integer(int_val));
    } else {
        return Err(invalid_expression_error(
            &parser.input,
            "invalid integer format",
            Some(start),
        ));
    }
    Ok(i.saturating_sub(1)) // Adjust for loop increment
}
