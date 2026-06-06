// --- tokens ---

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

    IntLiteral(i64),
    FloatLiteral(f64),
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

impl Clone for TokenKind {
    fn clone(&self) -> Self {
        match self {
            TokenKind::Newline => TokenKind::Newline,
            TokenKind::Identifier(name) => TokenKind::Identifier(name.to_string()),
            TokenKind::Equal => TokenKind::Equal,
            TokenKind::Colon => TokenKind::Colon,
            TokenKind::Arrow => TokenKind::Arrow,
            TokenKind::FatArrow => TokenKind::FatArrow,
            TokenKind::Comma => TokenKind::Comma,
            TokenKind::LeftParenthesis => TokenKind::LeftParenthesis,
            TokenKind::RightParenthesis => TokenKind::RightParenthesis,
            TokenKind::LeftBrace => TokenKind::LeftBrace,
            TokenKind::RightBrace => TokenKind::RightBrace,
            TokenKind::IntLiteral(value) => TokenKind::IntLiteral(*value),
            TokenKind::FloatLiteral(value) => TokenKind::FloatLiteral(*value),
            TokenKind::Plus => TokenKind::Plus,
            TokenKind::Minus => TokenKind::Minus,
            TokenKind::Star => TokenKind::Star,
            TokenKind::Slash => TokenKind::Slash,
            TokenKind::Caret => TokenKind::Caret,
            TokenKind::Concat => TokenKind::Concat,
            TokenKind::LeftPipe => TokenKind::LeftPipe,
            TokenKind::RightPipe => TokenKind::RightPipe,
            TokenKind::LeftComposition => TokenKind::LeftComposition,
            TokenKind::RightComposition => TokenKind::RightComposition,
            TokenKind::If => TokenKind::If,
        }
    }
}

pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Clone for Token {
    fn clone(&self) -> Self {
        Token {
            kind: self.kind.clone(),
            line: self.line,
            column: self.column,
        }
    }
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

        let token_type: TokenKind = match word.as_str() {
            "if" => TokenKind::If,
            _ => TokenKind::Identifier(word),
        };

        Some(Ok(Token {
            kind: token_type,
            line: start_line,
            column: start_column,
        }))
    }

    fn parse_number(
        &mut self,
        start_line: usize,
        start_column: usize,
    ) -> Option<Result<Token, LexerError>> {
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
            match num.parse::<f64>() {
                Ok(v) => TokenKind::FloatLiteral(v),
                Err(_) => {
                    return Some(Err(LexerError {
                        message: "Float parse error",
                        line: start_line,
                        column: start_column,
                    }));
                }
            }
        } else {
            match num.parse::<i64>() {
                Ok(v) => TokenKind::IntLiteral(v),
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

        let sym: String = self.chars[self.pos..self.pos + 2].iter().collect();

        let token_type: TokenKind = match sym.as_str() {
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
            kind: token_type,
            line: start_line,
            column: start_column,
        }))
    }

    fn parse_one_char_symbol(
        &mut self,
        start_line: usize,
        start_column: usize,
    ) -> Option<Result<Token, LexerError>> {
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
            line: start_line,
            column: start_column,
        }))
    }

    fn parse_newline(
        &mut self,
        start_line: usize,
        start_column: usize,
    ) -> Option<Result<Token, LexerError>> {
        let ch: char = self.peek()?;

        if ch != '\n' {
            return None;
        }

        self.next();

        Some(Ok(Token {
            kind: TokenKind::Newline,
            line: start_line,
            column: start_column,
        }))
    }
}
