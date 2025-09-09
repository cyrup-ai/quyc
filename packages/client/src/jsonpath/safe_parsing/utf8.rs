//! UTF-8 validation and recovery utilities
//!
//! Provides robust UTF-8 handling with multiple recovery strategies
//! and specialized JSONPath string processing capabilities.

use crate::jsonpath::error::{JsonPathResult, invalid_expression_error};

/// UTF-8 validation and recovery utilities
pub struct Utf8Handler;

impl Utf8Handler {
    /// Validate UTF-8 string with recovery options
    ///
    /// Provides multiple strategies for handling invalid UTF-8:
    /// - Strict: Return error on any invalid sequences
    /// - Replace: Replace invalid sequences with replacement character
    /// - Ignore: Skip invalid sequences entirely
    #[inline]
    pub fn validate_utf8_with_recovery(
        input: &[u8],
        strategy: Utf8RecoveryStrategy,
    ) -> JsonPathResult<String> {
        match strategy {
            Utf8RecoveryStrategy::Strict => std::str::from_utf8(input)
                .map(|s| s.to_string())
                .map_err(|e| {
                    invalid_expression_error(
                        "",
                        &format!("invalid UTF-8 sequence at byte {}", e.valid_up_to()),
                        Some(e.valid_up_to()),
                    )
                }),

            Utf8RecoveryStrategy::Replace => Ok(String::from_utf8_lossy(input).into_owned()),

            Utf8RecoveryStrategy::Ignore => {
                let mut result = String::new();
                let mut pos = 0;

                while pos < input.len() {
                    match std::str::from_utf8(&input[pos..]) {
                        Ok(valid_str) => {
                            result.push_str(valid_str);
                            break;
                        }
                        Err(e) => {
                            // Add valid portion
                            if e.valid_up_to() > 0 {
                                let valid_portion =
                                    std::str::from_utf8(&input[pos..pos + e.valid_up_to()])
                                        .map_err(|_| {
                                            invalid_expression_error(
                                                "",
                                                "internal UTF-8 validation error",
                                                Some(pos),
                                            )
                                        })?;
                                result.push_str(valid_portion);
                            }

                            // Skip invalid sequence
                            pos += e.valid_up_to() + 1;
                        }
                    }
                }

                Ok(result)
            }
        }
    }

    /// Validate JSONPath string with escape sequence handling
    ///
    /// Specifically handles UTF-8 validation in the context of JSONPath
    /// string literals, including proper handling of escape sequences.
    #[inline]
    pub fn validate_jsonpath_string(input: &str, allow_escapes: bool) -> JsonPathResult<String> {
        if !allow_escapes {
            // Production validation for unescaped strings
            return Ok(input.to_string());
        }

        let mut result = String::new();
        let mut chars = input.chars();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('\\') => result.push('\\'),
                    Some('\"') => result.push('\"'),
                    Some('\'') => result.push('\''),
                    Some('/') => result.push('/'),
                    Some('b') => result.push('\u{0008}'),
                    Some('f') => result.push('\u{000C}'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('u') => {
                        // Unicode escape sequence \uXXXX
                        let hex_chars: String = chars.by_ref().take(4).collect();
                        if hex_chars.len() != 4 {
                            return Err(invalid_expression_error(
                                input,
                                "incomplete Unicode escape sequence",
                                None,
                            ));
                        }

                        let code_point = u32::from_str_radix(&hex_chars, 16).map_err(|_| {
                            invalid_expression_error(
                                input,
                                &format!("invalid Unicode escape sequence: \\u{}", hex_chars),
                                None,
                            )
                        })?;

                        if let Some(unicode_char) = std::char::from_u32(code_point) {
                            result.push(unicode_char);
                        } else {
                            return Err(invalid_expression_error(
                                input,
                                &format!("invalid Unicode code point: U+{:04X}", code_point),
                                None,
                            ));
                        }
                    }
                    Some(invalid) => {
                        return Err(invalid_expression_error(
                            input,
                            &format!("invalid escape sequence: \\{}", invalid),
                            None,
                        ));
                    }
                    None => {
                        return Err(invalid_expression_error(
                            input,
                            "unterminated escape sequence",
                            None,
                        ));
                    }
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Detect and handle byte order marks (BOMs)
    ///
    /// Handles UTF-8 BOM and other encoding markers that might appear
    /// in JSONPath expressions loaded from files.
    #[inline]
    pub fn handle_bom(input: &[u8]) -> &[u8] {
        // UTF-8 BOM: EF BB BF
        if input.len() >= 3 && input[0] == 0xEF && input[1] == 0xBB && input[2] == 0xBF {
            return &input[3..];
        }

        input
    }
}

/// Strategy for handling invalid UTF-8 sequences
#[derive(Debug, Clone, Copy)]
pub enum Utf8RecoveryStrategy {
    /// Return error on any invalid UTF-8
    Strict,
    /// Replace invalid sequences with Unicode replacement character
    Replace,
    /// Skip invalid sequences entirely
    Ignore,
}
