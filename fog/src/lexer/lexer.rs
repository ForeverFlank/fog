use crate::error::Span;
use crate::error::{FogError, FogResult};
use crate::lex_error;
use crate::lexer::token::*;

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    paren_depth: i32,
    brace_depth: i32,
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

    pub fn tokenize(src: &str) -> (Vec<Token>, Vec<FogError>) {
        let mut lexer = Lexer::new(src);
        let mut tokens: Vec<Token> = Vec::new();
        let mut errors: Vec<FogError> = Vec::new();

        while lexer.pos < lexer.chars.len() {
            if lexer.skip_comment() {
                continue;
            }

            let start_line = lexer.line;
            let start_column = lexer.column;

            let result: Option<FogResult<Token>> = lexer
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

    fn parse_word(&mut self, start_line: usize, start_column: usize) -> Option<FogResult<Token>> {
        let pos = self.pos;
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
            pos,
            line: start_line,
            column: start_column,
        }))
    }

    fn parse_number(&mut self, start_line: usize, start_column: usize) -> Option<FogResult<Token>> {
        let pos = self.pos;
        let span = Span {
            line: start_line,
            column: start_column,
        };

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
                    return Some(Err(lex_error!(
                        Some(Span {
                            line: start_line,
                            column: start_column
                        }),
                        "Integer parse error"
                    )));
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
    ) -> Option<FogResult<Token>> {
        if self.pos + 1 >= self.chars.len() {
            return None;
        }

        let pos = self.pos;
        let sym = self.chars[self.pos..self.pos + 2]
            .iter()
            .collect::<String>();

        let kind = match_two_char_token(&sym)?;

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
    ) -> Option<FogResult<Token>> {
        let pos = self.pos;
        let sym = self.peek()?;

        let token_type = match_one_char_token(sym)?;

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
    ) -> Option<FogResult<Token>> {
        let pos = self.pos;
        let ch = self.peek()?;

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
