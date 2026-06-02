use crate::lexer::TokenType::Terminator;

pub enum TokenType {
    Terminator,

    Identifier,
    Equal,
    Colon,
    Arrow,
    LeftParenthesis,
    RightParenthesis,

    IntLiteral,
    FloatLiteral,
    StringLiteral,

    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Bar,
    Concat,

    LeftPipe,
    RightPipe,
    LeftComposition,
    RightComposition,

    If,
}

pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub pos: usize,
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    brace_depth: i32,
    paren_depth: i32,
}

pub struct LexerError {
    pub message: &'static str,
    pub pos: usize,
}

impl Lexer {
    fn new(src: &str) -> Lexer {
        Lexer {
            chars: src.chars().collect(),
            pos: 0,
            brace_depth: 0,
            paren_depth: 0,
        }
    }

    fn next(self: &mut Lexer) {
        self.pos += 1;
    }

    fn peek(self: &Lexer) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    pub fn tokenize(src: &str) -> (Vec<Token>, Vec<LexerError>) {
        let mut lexer: Lexer = Lexer::new(src);
        let mut tokens: Vec<Token> = Vec::new();
        let mut errors: Vec<LexerError> = Vec::new();

        while lexer.pos < lexer.chars.len() {
            if let Some(res) = lexer
                .try_parse_word()
                .or_else(|| lexer.try_parse_number())
                .or_else(|| lexer.try_parse_two_char_symbol())
                .or_else(|| lexer.try_parse_one_char_symbol())
            // .or_else(|| lexer.try_terminate(tokens.last()))
            {
                if let Ok(token) = res {
                    tokens.push(token);
                } else if let Err(error) = res {
                    errors.push(error)
                }
            } else {
                lexer.next();
            }
        }

        (tokens, errors)
    }

    fn try_parse_word(self: &mut Lexer) -> Option<Result<Token, LexerError>> {
        let ch: char = self.peek()?;

        if !(ch.is_alphabetic() || ch == '_') {
            return None;
        }

        let begin: usize = self.pos;

        let mut word: String = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                word.push(ch);
                self.next();
            } else {
                break;
            }
        }

        let token_type: TokenType = match word.as_str() {
            "if" => TokenType::If,
            _ => TokenType::Identifier,
        };

        Some(Ok(Token {
            token_type,
            value: word,
            pos: begin,
        }))
    }

    fn try_parse_number(self: &mut Lexer) -> Option<Result<Token, LexerError>> {
        let ch: char = self.peek()?;

        if !ch.is_numeric() {
            return None;
        }

        let begin: usize = self.pos;

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
                        pos: begin,
                    }));
                }
            }
        }

        let token_type: TokenType = if decimal {
            TokenType::FloatLiteral
        } else {
            TokenType::IntLiteral
        };

        Some(Ok(Token {
            token_type,
            value: num,
            pos: begin,
        }))
    }

    fn try_parse_two_char_symbol(self: &mut Lexer) -> Option<Result<Token, LexerError>> {
        if self.pos + 1 >= self.chars.len() {
            return None;
        }

        let sym: String = self.chars[self.pos..self.pos + 2].iter().collect();

        let begin: usize = self.pos;

        let token_type: TokenType = match sym.as_str() {
            "->" => TokenType::Arrow,

            "||" => TokenType::Concat,
            "<|" => TokenType::LeftPipe,
            "|>" => TokenType::RightPipe,
            "<<" => TokenType::LeftComposition,
            ">>" => TokenType::RightComposition,

            _ => return None,
        };

        self.next();
        self.next();

        Some(Ok(Token {
            token_type,
            value: sym.to_string().to_owned(),
            pos: begin,
        }))
    }

    fn try_parse_one_char_symbol(self: &mut Lexer) -> Option<Result<Token, LexerError>> {
        let sym: char = *self.chars.get(self.pos)?;

        let begin: usize = self.pos;

        let token_type: TokenType = match sym {
            ':' => TokenType::Colon,
            '=' => TokenType::Equal,
            '(' => TokenType::LeftParenthesis,
            ')' => TokenType::RightParenthesis,

            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Star,
            '/' => TokenType::Slash,
            '^' => TokenType::Caret,
            '|' => TokenType::Bar,

            _ => return None,
        };

        self.next();

        if sym == '(' {
            self.paren_depth += 1;
        }
        if sym == ')' {
            self.paren_depth -= 1;
        }

        Some(Ok(Token {
            token_type,
            value: sym.to_string().to_owned(),
            pos: begin,
        }))
    }

    // fn try_terminate(&mut self, last_token: Option<&Token>) -> Option<Result<Token, LexerError>> {
    //     let last_token: &Token = last_token?;
    //     let ch: char = self.peek()?;

    //     if ch != '\n'
    //         || self.paren_depth != 0
    //         || !matches!(
    //             last_token.token_type,
    //             TokenType::Plus | TokenType::Minus /* ... */
    //         )
    //     {
    //         return None;
    //     }

    //     let begin: usize = self.pos;
    //     self.next();

    //     Some(Ok(Token {
    //         token_type: Terminator,
    //         value: String::new(),
    //         pos: begin,
    //     }))
    // }
}

pub fn tokenize(src: &str) -> (Vec<Token>, Vec<LexerError>) {
    Lexer::tokenize(src)
}
