//! Token definitions for `JSONPath` lexical analysis
//!
//! Defines the token types used in `JSONPath` expression parsing and provides
//! utility functions for token comparison and matching.

/// Tokens for `JSONPath` expression lexical analysis
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Structural tokens
    /// Root identifier token ($)
    Root,
    /// Dot notation token (.)
    Dot,
    /// Double dot (recursive descent) token (..)
    DoubleDot,
    /// Left bracket token ([)
    LeftBracket,
    /// Right bracket token (])
    RightBracket,
    /// Left parenthesis token (()
    LeftParen,
    /// Right parenthesis token ())
    RightParen,
    /// Comma separator token (,)
    Comma,
    /// Colon separator token (:)
    Colon,
    /// Question mark token (?)
    Question,
    /// Current node identifier token (@)
    At,
    /// Wildcard selector token (*)
    Star,

    // Literals
    /// String literal token
    String(String),
    /// Integer literal token
    Integer(i64),
    /// Floating-point number literal token
    Number(f64),
    /// Boolean true literal token
    True,
    /// Boolean false literal token
    False,
    /// Null literal token
    Null,

    // Operators
    /// Equality operator token (==)
    Equal,
    /// Inequality operator token (!=)
    NotEqual,
    /// Less than operator token (<)
    Less,
    /// Less than or equal operator token (<=)
    LessEq,
    /// Greater than operator token (>)
    Greater,
    /// Greater than or equal operator token (>=)
    GreaterEq,
    /// Logical AND operator token (&&)
    LogicalAnd,
    /// Logical OR operator token (||)
    LogicalOr,

    // Identifiers and functions
    /// Property identifier or function name token
    Identifier(String),

    // Special
    /// End of file/input token
    EOF,
}

impl Token {
    /// Check if token is a comparison operator
    #[inline]
    #[must_use] 
    pub fn is_comparison_operator(&self) -> bool {
        matches!(
            self,
            Token::Equal
                | Token::NotEqual
                | Token::Less
                | Token::LessEq
                | Token::Greater
                | Token::GreaterEq
        )
    }

    /// Check if token is a logical operator
    #[inline]
    #[must_use] 
    pub fn is_logical_operator(&self) -> bool {
        matches!(self, Token::LogicalAnd | Token::LogicalOr)
    }

    /// Check if token is a literal value
    #[inline]
    #[must_use] 
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Token::String(_)
                | Token::Integer(_)
                | Token::Number(_)
                | Token::True
                | Token::False
                | Token::Null
        )
    }

    /// Check if token represents a structural element
    #[inline]
    #[must_use] 
    pub fn is_structural(&self) -> bool {
        matches!(
            self,
            Token::Root
                | Token::Dot
                | Token::DoubleDot
                | Token::LeftBracket
                | Token::RightBracket
                | Token::LeftParen
                | Token::RightParen
                | Token::Comma
                | Token::Colon
                | Token::Question
                | Token::At
                | Token::Star
        )
    }

    /// Get string representation for debugging
    #[must_use] 
    pub fn as_debug_str(&self) -> &'static str {
        match self {
            Token::Root => "$",
            Token::Dot => ".",
            Token::DoubleDot => "..",
            Token::LeftBracket => "[",
            Token::RightBracket => "]",
            Token::LeftParen => "(",
            Token::RightParen => ")",
            Token::Comma => ",",
            Token::Colon => ":",
            Token::Question => "?",
            Token::At => "@",
            Token::Star => "*",
            Token::String(_) => "string",
            Token::Integer(_) => "integer",
            Token::Number(_) => "number",
            Token::True => "true",
            Token::False => "false",
            Token::Null => "null",
            Token::Equal => "==",
            Token::NotEqual => "!=",
            Token::Less => "<",
            Token::LessEq => "<=",
            Token::Greater => ">",
            Token::GreaterEq => ">=",
            Token::LogicalAnd => "&&",
            Token::LogicalOr => "||",
            Token::Identifier(_) => "identifier",
            Token::EOF => "EOF",
        }
    }
}

/// Utility functions for token matching and comparison
pub struct TokenMatcher;

impl TokenMatcher {
    /// Check if two tokens match (handles different variants with same discriminant)
    #[inline]
    #[must_use] 
    pub fn tokens_match(actual: &Token, expected: &Token) -> bool {
        match (actual, expected) {
            (Token::RightBracket, Token::RightBracket) => true,
            (Token::RightParen, Token::RightParen) => true,
            (Token::LeftParen, Token::LeftParen) => true,
            (Token::LeftBracket, Token::LeftBracket) => true,
            (Token::Dot, Token::Dot) => true,
            (Token::DoubleDot, Token::DoubleDot) => true,
            (Token::Comma, Token::Comma) => true,
            (Token::Colon, Token::Colon) => true,
            (Token::Question, Token::Question) => true,
            (Token::At, Token::At) => true,
            (Token::Star, Token::Star) => true,
            (Token::Equal, Token::Equal) => true,
            (Token::NotEqual, Token::NotEqual) => true,
            (Token::Less, Token::Less) => true,
            (Token::LessEq, Token::LessEq) => true,
            (Token::Greater, Token::Greater) => true,
            (Token::GreaterEq, Token::GreaterEq) => true,
            (Token::LogicalAnd, Token::LogicalAnd) => true,
            (Token::LogicalOr, Token::LogicalOr) => true,
            (Token::True, Token::True) => true,
            (Token::False, Token::False) => true,
            (Token::Null, Token::Null) => true,
            (Token::Root, Token::Root) => true,
            (Token::EOF, Token::EOF) => true,
            // For tokens with data, we only check the discriminant
            (Token::String(_), Token::String(_)) => true,
            (Token::Integer(_), Token::Integer(_)) => true,
            (Token::Number(_), Token::Number(_)) => true,
            (Token::Identifier(_), Token::Identifier(_)) => true,
            _ => false,
        }
    }

    /// Get precedence level for operators (higher number = higher precedence)
    #[inline]
    #[must_use] 
    pub fn operator_precedence(token: &Token) -> u8 {
        match token {
            Token::LogicalOr => 1,
            Token::LogicalAnd => 2,
            Token::Equal | Token::NotEqual => 3,
            Token::Less | Token::LessEq | Token::Greater | Token::GreaterEq => 4,
            _ => 0,
        }
    }

    /// Check if token can start a primary expression
    #[inline]
    #[must_use] 
    pub fn can_start_primary(token: &Token) -> bool {
        matches!(
            token,
            Token::At
                | Token::String(_)
                | Token::Number(_)
                | Token::Integer(_)
                | Token::True
                | Token::False
                | Token::Null
                | Token::LeftParen
                | Token::Identifier(_)
        )
    }
}
