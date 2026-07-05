use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Pos;
use crate::error::Span;
use crate::lex_error;
use crate::lexer::token::*;

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    fn new(src: &str) -> Self {
        Lexer {
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn current_pos(&self) -> Pos {
        Pos {
            line: self.line,
            column: self.column,
        }
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

    pub fn tokenize(src: &str) -> (Vec<Token>, Vec<FogError>) {
        let mut lexer = Lexer::new(src);
        let mut tokens: Vec<Token> = Vec::new();
        let mut errors: Vec<FogError> = Vec::new();

        while lexer.pos < lexer.chars.len() {
            if lexer.skip_comment() {
                continue;
            }

            let start = lexer.current_pos();

            let result: Option<FogResult<Token>> = lexer
                .parse_newline(start)
                .or_else(|| lexer.parse_word(start))
                .or_else(|| lexer.parse_number(start))
                .or_else(|| lexer.parse_two_char_symbol(start))
                .or_else(|| lexer.parse_one_char_symbol(start));

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

    fn parse_word(&mut self, start: Pos) -> Option<FogResult<Token>> {
        let ch = self.peek()?;

        if !(ch.is_alphabetic() || ch == '_') {
            return None;
        }

        let mut word = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                word.push(ch);
                self.next();
            } else {
                break;
            }
        }

        let kind = match_keyword(&word).unwrap_or(TokenKind::Identifier(word));

        Some(Ok(Token {
            kind,
            span: Span::new(start, self.current_pos()),
        }))
    }

    fn parse_number(&mut self, start: Pos) -> Option<FogResult<Token>> {
        let span = Span::new(start, start);

        let ch = self.peek()?;

        if !ch.is_numeric() {
            return None;
        }

        let mut num = String::new();
        let mut decimal = false;

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
                    return Some(Err(lex_error!(Some(span), "Malformed number")));
                }
            }
        }

        let span = Span::new(start, self.current_pos());

        let kind = if decimal {
            match num.parse::<f32>() {
                Ok(v) => TokenKind::Float32Literal(v),
                Err(_) => {
                    return Some(Err(lex_error!(Some(span), "Float parse error")));
                }
            }
        } else {
            match num.parse::<i32>() {
                Ok(v) => TokenKind::Int32Literal(v),
                Err(_) => {
                    return Some(Err(lex_error!(Some(span), "Integer parse error")));
                }
            }
        };

        Some(Ok(Token { kind, span }))
    }

    fn parse_two_char_symbol(&mut self, start: Pos) -> Option<FogResult<Token>> {
        if self.pos + 1 >= self.chars.len() {
            return None;
        }

        let sym = self.chars[self.pos..self.pos + 2]
            .iter()
            .collect::<String>();

        let kind = match_two_char_token(&sym)?;

        self.next();
        self.next();

        Some(Ok(Token {
            kind,
            span: Span::new(start, self.current_pos()),
        }))
    }

    fn parse_one_char_symbol(&mut self, start: Pos) -> Option<FogResult<Token>> {
        let sym = self.peek()?;

        let token_type = match_one_char_token(sym)?;

        self.next();

        Some(Ok(Token {
            kind: token_type,
            span: Span::new(start, self.current_pos()),
        }))
    }

    fn parse_newline(&mut self, start: Pos) -> Option<FogResult<Token>> {
        let ch = self.peek()?;

        if ch != '\n' {
            return None;
        }

        self.next();

        Some(Ok(Token {
            kind: TokenKind::Newline,
            span: Span::new(start, self.current_pos()),
        }))
    }
}
