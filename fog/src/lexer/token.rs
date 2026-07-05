use std::char;
use std::fmt;

use crate::error::Span;

#[derive(Clone)]
pub enum TokenKind {
    Eof,
    Newline,

    // statements
    Identifier(String),
    Equal,
    Colon,
    Arrow,
    FatArrow,
    Comma,

    // parentheses
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,

    // literals
    Int32Literal(i32),
    Float32Literal(f32),
    // StringLiteral(String),

    // operators
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Concat,
    LeftPipe,
    RightPipe,
    LeftComposition,
    RightComposition,

    // keywords
    Match,
    // If,
}

pub fn match_one_char_token(ch: char) -> Option<TokenKind> {
    match ch {
        ':' => Some(TokenKind::Colon),
        '=' => Some(TokenKind::Equal),
        ',' => Some(TokenKind::Comma),
        '(' => Some(TokenKind::LeftParenthesis),
        ')' => Some(TokenKind::RightParenthesis),
        '{' => Some(TokenKind::LeftBrace),
        '}' => Some(TokenKind::RightBrace),

        '+' => Some(TokenKind::Plus),
        '-' => Some(TokenKind::Minus),
        '*' => Some(TokenKind::Star),
        '/' => Some(TokenKind::Slash),
        '^' => Some(TokenKind::Caret),

        _ => None,
    }
}

pub fn match_two_char_token(s: &str) -> Option<TokenKind> {
    match s {
        "->" => Some(TokenKind::Arrow),
        "=>" => Some(TokenKind::FatArrow),

        "||" => Some(TokenKind::Concat),
        "<|" => Some(TokenKind::LeftPipe),
        "|>" => Some(TokenKind::RightPipe),
        "<<" => Some(TokenKind::LeftComposition),
        ">>" => Some(TokenKind::RightComposition),

        _ => None,
    }
}

pub fn match_keyword(str: &str) -> Option<TokenKind> {
    match str {
        "match" => Some(TokenKind::Match),

        _ => None,
    }
}

impl TokenKind {
    pub fn is_primary_starter(&self) -> bool {
        match self {
            TokenKind::Identifier(_)
            | TokenKind::Int32Literal(_)
            | TokenKind::Float32Literal(_)
            | TokenKind::LeftParenthesis
            | TokenKind::Minus => true,
            _ => false,
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Eof => write!(f, "Eof"),
            TokenKind::Newline => write!(f, "Newline"),

            TokenKind::Identifier(val) => write!(f, "Identifier ({})", val),
            TokenKind::Equal => write!(f, "Equal"),
            TokenKind::Colon => write!(f, "Colon"),
            TokenKind::Arrow => write!(f, "Arrow"),
            TokenKind::FatArrow => write!(f, "FatArrow"),
            TokenKind::Comma => write!(f, "Comma"),

            TokenKind::LeftParenthesis => write!(f, "LeftParenthesis"),
            TokenKind::RightParenthesis => write!(f, "RightParenthesis"),
            TokenKind::LeftBrace => write!(f, "LeftBrace"),
            TokenKind::RightBrace => write!(f, "RightBrace"),

            TokenKind::Int32Literal(val) => write!(f, "Int32 ({})", val),
            TokenKind::Float32Literal(val) => write!(f, "Float32 ({})", val),
            // TokenKind::StringLiteral(val) => write!(f, "String ({})", val),
            TokenKind::Plus => write!(f, "Plus"),
            TokenKind::Minus => write!(f, "Minus"),
            TokenKind::Star => write!(f, "Star"),
            TokenKind::Slash => write!(f, "Slash"),
            TokenKind::Caret => write!(f, "Caret"),
            TokenKind::Concat => write!(f, "Concat"),
            TokenKind::LeftPipe => write!(f, "LeftPipe"),
            TokenKind::RightPipe => write!(f, "RightPipe"),
            TokenKind::LeftComposition => write!(f, "LeftComposition"),
            TokenKind::RightComposition => write!(f, "RightComposition"),

            TokenKind::Match => write!(f, "Match"),
            // TokenKind::If => write!(f, "If"),
        }
    }
}

#[derive(Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: usize,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn span(&self) -> Span {
        Span {
            line: self.line,
            column: self.column,
        }
    }
}
