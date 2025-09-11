//! Normalized path string parsing and validation
//!
//! This module handles parsing normalized path strings into internal representations,
//! validating syntax and ensuring conformance to RFC 9535 requirements.

use super::types::{NormalizedPath, NormalizedPathProcessor, PathSegment};
use crate::jsonpath::error::{JsonPathResult, invalid_expression_error};

impl NormalizedPathProcessor {
    /// Parse a normalized path string into segments
    ///
    /// Validates that the input string conforms to normalized path syntax
    /// and converts it to internal representation.
    #[inline]
    pub fn parse_normalized_path(path: &str) -> JsonPathResult<NormalizedPath> {
        if path == "$" {
            return Ok(NormalizedPath::root());
        }

        if !path.starts_with('$') {
            return Err(invalid_expression_error(
                path,
                "normalized paths must start with $",
                Some(0),
            ));
        }

        let mut segments = vec![PathSegment::Root];
        let remaining = &path[1..];

        if remaining.is_empty() {
            return Ok(NormalizedPath {
                segments,
                normalized_string: path.to_string(),
            });
        }

        let mut chars = remaining.chars().peekable();
        let mut position = 1; // Start after $

        while chars.peek().is_some() {
            if chars.next() != Some('[') {
                return Err(invalid_expression_error(
                    path,
                    "normalized paths must use bracket notation",
                    Some(position),
                ));
            }
            position += 1;

            // Parse the bracket content
            let segment = Self::parse_bracket_content(&mut chars, &mut position, path)?;
            segments.push(segment);

            if chars.next() != Some(']') {
                return Err(invalid_expression_error(
                    path,
                    "expected closing bracket",
                    Some(position),
                ));
            }
            position += 1;
        }

        Ok(NormalizedPath {
            segments,
            normalized_string: path.to_string(),
        })
    }

    /// Parse content within brackets
    #[inline]
    fn parse_bracket_content(
        chars: &mut std::iter::Peekable<std::str::Chars>,
        position: &mut usize,
        full_path: &str,
    ) -> JsonPathResult<PathSegment> {
        let start_pos = *position;

        match chars.peek() {
            Some('\'') => {
                // Single-quoted string (member name)
                chars.next(); // consume opening quote
                *position += 1;

                let mut member_name = String::new();

                while let Some(ch) = chars.next() {
                    *position += 1;

                    if ch == '\'' {
                        // End of string
                        Self::validate_member_name(&member_name)?;
                        return Ok(PathSegment::Member(member_name));
                    } else if ch == '\\' {
                        // Escape sequence
                        match chars.next() {
                            Some('\'') => {
                                member_name.push('\'');
                                *position += 1;
                            }
                            Some('\\') => {
                                member_name.push('\\');
                                *position += 1;
                            }
                            Some(escaped) => {
                                return Err(invalid_expression_error(
                                    full_path,
                                    format!("invalid escape sequence \\{escaped}"),
                                    Some(*position),
                                ));
                            }
                            None => {
                                return Err(invalid_expression_error(
                                    full_path,
                                    "unterminated escape sequence",
                                    Some(*position),
                                ));
                            }
                        }
                    } else {
                        member_name.push(ch);
                    }
                }

                Err(invalid_expression_error(
                    full_path,
                    "unterminated string literal",
                    Some(start_pos),
                ))
            }

            Some(ch) if ch.is_ascii_digit() => {
                // Array index
                let mut index_str = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        index_str.push(ch);
                        chars.next();
                        *position += 1;
                    } else {
                        break;
                    }
                }

                // Validate no leading zeros (except for "0")
                if index_str.len() > 1 && index_str.starts_with('0') {
                    return Err(invalid_expression_error(
                        full_path,
                        "array indices cannot have leading zeros",
                        Some(start_pos),
                    ));
                }

                let index = index_str.parse::<i64>().map_err(|_| {
                    invalid_expression_error(full_path, "invalid array index", Some(start_pos))
                })?;

                if index < 0 {
                    return Err(invalid_expression_error(
                        full_path,
                        "normalized paths require non-negative array indices",
                        Some(start_pos),
                    ));
                }

                Ok(PathSegment::Index(index))
            }

            _ => Err(invalid_expression_error(
                full_path,
                "expected string literal or array index",
                Some(start_pos),
            )),
        }
    }
}
