//! Unified byte processing trait implementation
//!
//! Consolidates duplicate byte processing from core.rs and processor/core.rs

use crate::jsonpath::error::{JsonPathError, JsonPathResult};
use crate::jsonpath::buffer::StreamBuffer;

/// Result of processing a JSON byte
#[derive(Debug, Clone, PartialEq)]
pub enum JsonProcessResult {
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Comma,
    Colon,
    Whitespace,
}

/// Unified byte processing trait
pub trait JsonByteProcessor {
    fn read_next_byte(&mut self) -> JsonPathResult<Option<u8>>;
    fn process_json_byte(&mut self, byte: u8) -> JsonPathResult<JsonProcessResult>;
    fn skip_whitespace(&mut self) -> JsonPathResult<()>;
    fn read_string(&mut self) -> JsonPathResult<String>;
    fn read_number(&mut self) -> JsonPathResult<f64>;
}

/// Shared byte processor implementation
pub struct SharedByteProcessor<'a> {
    buffer: &'a mut StreamBuffer,
    position: usize,
    bytes_consumed: usize,
    depth: usize,
}

impl<'a> SharedByteProcessor<'a> {
    pub fn new(buffer: &'a mut StreamBuffer, position: usize) -> Self {
        Self {
            buffer,
            position,
            bytes_consumed: 0,
            depth: 0,
        }
    }
    
    pub fn position(&self) -> usize {
        self.position
    }
    
    pub fn bytes_consumed(&self) -> usize {
        self.bytes_consumed
    }
}

impl<'a> JsonByteProcessor for SharedByteProcessor<'a> {
    fn read_next_byte(&mut self) -> JsonPathResult<Option<u8>> {
        if self.position >= self.buffer.len() {
            return Ok(None);
        }
        
        let byte = self.buffer.get_byte_at(self.position)
            .ok_or_else(|| JsonPathError::buffer_underflow())?;
        self.position += 1;
        self.bytes_consumed += 1;
        Ok(Some(byte))
    }
    
    fn process_json_byte(&mut self, byte: u8) -> JsonPathResult<JsonProcessResult> {
        use JsonProcessResult::*;
        
        match byte {
            b'{' => {
                self.depth += 1;
                Ok(ObjectStart)
            }
            b'}' => {
                self.depth = self.depth.saturating_sub(1);
                Ok(ObjectEnd)
            }
            b'[' => {
                self.depth += 1;
                Ok(ArrayStart)
            }
            b']' => {
                self.depth = self.depth.saturating_sub(1);
                Ok(ArrayEnd)
            }
            b'"' => {
                let string = self.read_string()?;
                Ok(String(string))
            }
            b't' | b'f' => {
                let bool_val = self.read_boolean(byte)?;
                Ok(Boolean(bool_val))
            }
            b'n' => {
                self.read_null()?;
                Ok(Null)
            }
            b'-' | b'0'..=b'9' => {
                let number = self.read_number_from_first_byte(byte)?;
                Ok(Number(number))
            }
            b',' => Ok(Comma),
            b':' => Ok(Colon),
            b' ' | b'\t' | b'\n' | b'\r' => {
                self.skip_whitespace()?;
                Ok(Whitespace)
            }
            _ => Err(JsonPathError::unexpected_byte(byte as char))
        }
    }
    
    fn skip_whitespace(&mut self) -> JsonPathResult<()> {
        while let Some(byte) = self.read_next_byte()? {
            match byte {
                b' ' | b'\t' | b'\n' | b'\r' => {},
                _ => {
                    // Put back non-whitespace byte
                    self.position -= 1;
                    self.bytes_consumed -= 1;
                    break;
                }
            }
        }
        Ok(())
    }
    
    fn read_string(&mut self) -> JsonPathResult<String> {
        let mut string = Vec::new();
        let mut escaped = false;
        
        while let Some(byte) = self.read_next_byte()? {
            if escaped {
                string.push(match byte {
                    b'n' => b'\n',
                    b'r' => b'\r', 
                    b't' => b'\t',
                    b'\\' => b'\\',
                    b'"' => b'"',
                    b'/' => b'/',
                    _ => byte,
                });
                escaped = false;
            } else {
                match byte {
                    b'\\' => escaped = true,
                    b'"' => break,
                    _ => string.push(byte),
                }
            }
        }
        
        String::from_utf8(string)
            .map_err(|_| JsonPathError::invalid_utf8())
    }
    
    fn read_number(&mut self) -> JsonPathResult<f64> {
        if let Some(first) = self.read_next_byte()? {
            self.read_number_from_first_byte(first)
        } else {
            Err(JsonPathError::unexpected_end_of_input())
        }
    }
}

// Private helper methods
impl<'a> SharedByteProcessor<'a> {
    fn read_boolean(&mut self, first: u8) -> JsonPathResult<bool> {
        match first {
            b't' => {
                self.expect_bytes(b"rue")?;
                Ok(true)
            }
            b'f' => {
                self.expect_bytes(b"alse")?;
                Ok(false) 
            }
            _ => Err(JsonPathError::unexpected_byte(first as char))
        }
    }
    
    fn read_null(&mut self) -> JsonPathResult<()> {
        self.expect_bytes(b"ull")?;
        Ok(())
    }
    
    fn expect_bytes(&mut self, expected: &[u8]) -> JsonPathResult<()> {
        for &expected_byte in expected {
            match self.read_next_byte()? {
                Some(byte) if byte == expected_byte => {},
                Some(byte) => return Err(JsonPathError::unexpected_byte(byte as char)),
                None => return Err(JsonPathError::unexpected_end_of_input()),
            }
        }
        Ok(())
    }
    
    fn read_number_from_first_byte(&mut self, first: u8) -> JsonPathResult<f64> {
        let mut number_str = vec![first];
        
        while let Some(byte) = self.peek_next_byte()? {
            match byte {
                b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-' => {
                    self.read_next_byte()?;
                    number_str.push(byte);
                }
                _ => break,
            }
        }
        
        let s = String::from_utf8(number_str)
            .map_err(|_| JsonPathError::invalid_utf8())?;
        s.parse()
            .map_err(|_| JsonPathError::invalid_number())
    }
    
    fn peek_next_byte(&mut self) -> JsonPathResult<Option<u8>> {
        if self.position >= self.buffer.len() {
            return Ok(None);
        }
        
        Ok(self.buffer.get_byte_at(self.position))
    }
}