// --- tokens ---

#[derive(Clone)]
pub enum TokenKind {
    Newline,

    Identifier(String),
    Equal,
    Colon,
    Arrow,
    FatArrow,
    Comma,
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,

    Int32Literal(i32),
    Float32Literal(f32),
    // StringLiteral(String),
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

    If,
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
            TokenKind::LeftParenthesis => "LeftParenthesis".to_string(),
            TokenKind::RightParenthesis => "RightParenthesis".to_string(),
            TokenKind::LeftBrace => "LeftBrace".to_string(),
            TokenKind::RightBrace => "RightBrace".to_string(),
            TokenKind::Comma => "Comma".to_string(),
            TokenKind::Int32Literal(val) => format!("Int ({})", val),
            TokenKind::Float32Literal(val) => format!("Float ({})", val),
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

// --- lexer ---

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    paren_depth: i32,
    brace_depth: i32,
}

pub struct LexerError {
    pub message: &'static str,
    pub line: usize,
    pub column: usize,
}

pub fn tokenize(src: &str) -> (Vec<Token>, Vec<LexerError>) {
    Lexer::tokenize(src)
}

impl Lexer {
    fn new(src: &str) -> Self {
        Lexer {
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            paren_depth: 0,
            brace_depth: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) {
        if let Some(ch) = self.peek() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.pos += 1;
    }

    pub fn tokenize(src: &str) -> (Vec<Token>, Vec<LexerError>) {
        let mut lexer: Lexer = Lexer::new(src);
        let mut tokens: Vec<Token> = Vec::new();
        let mut errors: Vec<LexerError> = Vec::new();

        while lexer.pos < lexer.chars.len() {
            if lexer.skip_comment() {
                continue;
            }

            let start_line: usize = lexer.line;
            let start_column: usize = lexer.column;

            let result: Option<Result<Token, LexerError>> = lexer
                .parse_newline(start_line, start_column)
                .or_else(|| lexer.parse_word(start_line, start_column))
                .or_else(|| lexer.parse_number(start_line, start_column))
                .or_else(|| lexer.parse_two_char_symbol(start_line, start_column))
                .or_else(|| lexer.parse_one_char_symbol(start_line, start_column));

            match result {
                Some(Ok(token)) => tokens.push(token),
                Some(Err(error)) => errors.push(error),
                None => lexer.next(),
            }
        }

        (tokens, errors)
    }

    fn skip_comment(&mut self) -> bool {
        if self.pos + 1 >= self.chars.len() {
            return false;
        }

        if self.chars[self.pos] == '-' && self.chars[self.pos + 1] == '-' {
            self.next();
            self.next();

            while let Some(ch) = self.peek() {
                if ch == '\n' {
                    break;
                }
                self.next();
            }

            return true;
        }

        false
    }

    fn parse_word(
        &mut self,
        start_line: usize,
        start_column: usize,
    ) -> Option<Result<Token, LexerError>> {
        let pos: usize = self.pos;
        let ch: char = self.peek()?;

        if !(ch.is_alphabetic() || ch == '_') {
            return None;
        }

        let mut word: String = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                word.push(ch);
                self.next();
            } else {
                break;
            }
        }

        let kind: TokenKind = match word.as_str() {
            "if" => TokenKind::If,
            _ => TokenKind::Identifier(word),
        };

        Some(Ok(Token {
            kind,
            pos,
            line: start_line,
            column: start_column,
        }))
    }

    fn parse_number(
        &mut self,
        start_line: usize,
        start_column: usize,
    ) -> Option<Result<Token, LexerError>> {
        let pos: usize = self.pos;
        let ch: char = self.peek()?;

        if !ch.is_numeric() {
            return None;
        }

        let mut num: String = String::new();
        let mut decimal: bool = false;

        while let Some(ch) = self.peek() {
            if !(ch.is_numeric() || ch == '.') {
                break;
            }

            num.push(ch);
            self.next();

            if ch == '.' {
                if !decimal {
                    decimal = true;
                } else {
                    return Some(Err(LexerError {
                        message: "Malformed number",
                        line: start_line,
                        column: start_column,
                    }));
                }
            }
        }

        let kind = if decimal {
            match num.parse::<f32>() {
                Ok(v) => TokenKind::Float32Literal(v),
                Err(_) => {
                    return Some(Err(LexerError {
                        message: "Float parse error",
                        line: start_line,
                        column: start_column,
                    }));
                }
            }
        } else {
            match num.parse::<i32>() {
                Ok(v) => TokenKind::Int32Literal(v),
                Err(_) => {
                    return Some(Err(LexerError {
                        message: "Integer parse error",
                        line: start_line,
                        column: start_column,
                    }));
                }
            }
        };

        Some(Ok(Token {
            kind,
            pos,
            line: start_line,
            column: start_column,
        }))
    }

    fn parse_two_char_symbol(
        &mut self,
        start_line: usize,
        start_column: usize,
    ) -> Option<Result<Token, LexerError>> {
        if self.pos + 1 >= self.chars.len() {
            return None;
        }

        let pos: usize = self.pos;
        let sym: String = self.chars[self.pos..self.pos + 2].iter().collect();

        let kind: TokenKind = match sym.as_str() {
            "->" => TokenKind::Arrow,
            "=>" => TokenKind::FatArrow,

            "||" => TokenKind::Concat,
            "<|" => TokenKind::LeftPipe,
            "|>" => TokenKind::RightPipe,
            "<<" => TokenKind::LeftComposition,
            ">>" => TokenKind::RightComposition,

            _ => return None,
        };

        self.next();
        self.next();

        Some(Ok(Token {
            kind,
            pos,
            line: start_line,
            column: start_column,
        }))
    }

    fn parse_one_char_symbol(
        &mut self,
        start_line: usize,
        start_column: usize,
    ) -> Option<Result<Token, LexerError>> {
        let pos: usize = self.pos;
        let sym: char = self.peek()?;

        let token_type: TokenKind = match sym {
            ':' => TokenKind::Colon,
            '=' => TokenKind::Equal,
            ',' => TokenKind::Comma,
            '(' => TokenKind::LeftParenthesis,
            ')' => TokenKind::RightParenthesis,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,

            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '^' => TokenKind::Caret,

            _ => return None,
        };

        self.next();

        if sym == '(' {
            self.paren_depth += 1;
        }
        if sym == ')' {
            self.paren_depth -= 1;
        }
        if sym == '{' {
            self.brace_depth += 1;
        }
        if sym == '}' {
            self.brace_depth -= 1;
        }

        Some(Ok(Token {
            kind: token_type,
            pos,
            line: start_line,
            column: start_column,
        }))
    }

    fn parse_newline(
        &mut self,
        start_line: usize,
        start_column: usize,
    ) -> Option<Result<Token, LexerError>> {
        let pos: usize = self.pos;
        let ch: char = self.peek()?;

        if ch != '\n' {
            return None;
        }

        self.next();

        Some(Ok(Token {
            kind: TokenKind::Newline,
            pos,
            line: start_line,
            column: start_column,
        }))
    }
}
