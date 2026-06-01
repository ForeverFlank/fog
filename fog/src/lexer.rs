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

pub fn tokenize(src: &str) -> Vec<Token> {
    Lexer::tokenize(src)
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(src: &str) -> Lexer {
        Lexer {
            chars: src.chars().collect(),
            pos: 0,
        }
    }

    fn next(self: &mut Lexer) {
        self.pos += 1;
    }

    fn peek(self: &Lexer) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    pub fn tokenize(src: &str) -> Vec<Token> {
        let mut lexer: Lexer = Lexer::new(src);
        let mut res: Vec<Token> = Vec::new();

        while lexer.pos < lexer.chars.len() {
            if let Some(token) = lexer
                .try_parse_word()
                .or_else(|| lexer.try_parse_number())
                .or_else(|| lexer.try_parse_two_char_symbol())
                .or_else(|| lexer.try_parse_one_char_symbol())
            {
                res.push(token);
            } else {
                lexer.next();
            }
        }

        res
    }

    fn try_parse_word(self: &mut Lexer) -> Option<Token> {
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

        Some(Token {
            token_type,
            value: word,
            pos: begin,
        })
    }

    fn try_parse_number(self: &mut Lexer) -> Option<Token> {
        let ch: char = self.peek()?;

        if !ch.is_numeric() {
            return None;
        }

        let begin: usize = self.pos;

        let mut num: String = String::new();
        let mut decimal: bool = false;

        while let Some(ch) = self.peek() {
            if ch.is_numeric() || ch == '.' {
                num.push(ch);
                self.next();

                if ch == '.' {
                    if !decimal {
                        decimal = true;
                    } else {
                        // error
                    }
                }
            } else {
                break;
            }
        }

        let token_type = if decimal {
            TokenType::FloatLiteral
        } else {
            TokenType::IntLiteral
        };

        Some(Token {
            token_type,
            value: num,
            pos: begin,
        })
    }

    fn try_parse_two_char_symbol(self: &mut Lexer) -> Option<Token> {
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

        Some(Token {
            token_type,
            value: sym.to_string().to_owned(),
            pos: begin,
        })
    }

    fn try_parse_one_char_symbol(self: &mut Lexer) -> Option<Token> {
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

            _ => return None,
        };

        self.next();

        Some(Token {
            token_type,
            value: sym.to_string().to_owned(),
            pos: begin,
        })
    }
}
