use std::char;

#[derive(Clone)]
pub enum TokenKind {
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

    // sugars
    If,
}

pub fn match_one_char_token(char: char) -> Option<TokenKind> {
    match char {
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

pub fn match_two_char_token(str: &str) -> Option<TokenKind> {
    match str {
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

impl ToString for TokenKind {
    fn to_string(&self) -> String {
        match self {
            TokenKind::Newline => "Newline".to_string(),

            TokenKind::Identifier(val) => format!("Identifier ({})", val),
            TokenKind::Equal => "Equal".to_string(),
            TokenKind::Colon => "Colon".to_string(),
            TokenKind::Arrow => "Arrow".to_string(),
            TokenKind::FatArrow => "FatArrow".to_string(),
            TokenKind::Comma => "Comma".to_string(),

            TokenKind::LeftParenthesis => "LeftParenthesis".to_string(),
            TokenKind::RightParenthesis => "RightParenthesis".to_string(),
            TokenKind::LeftBrace => "LeftBrace".to_string(),
            TokenKind::RightBrace => "RightBrace".to_string(),

            TokenKind::Int32Literal(val) => format!("Int32 ({})", val),
            TokenKind::Float32Literal(val) => format!("Float32 ({})", val),
            // TokenKind::StringLiteral(val) => format!("String ({})", val),
            TokenKind::Plus => "Plus".to_string(),
            TokenKind::Minus => "Minus".to_string(),
            TokenKind::Star => "Star".to_string(),
            TokenKind::Slash => "Slash".to_string(),
            TokenKind::Caret => "Caret".to_string(),
            TokenKind::Concat => "Concat".to_string(),
            TokenKind::LeftPipe => "LeftPipe".to_string(),
            TokenKind::RightPipe => "RightPipe".to_string(),
            TokenKind::LeftComposition => "LeftComposition".to_string(),
            TokenKind::RightComposition => "RightComposition".to_string(),

            TokenKind::If => "If".to_string(),
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
