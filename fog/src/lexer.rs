pub enum TokenType {
    Terminator,

    Identifier,
    Equal,
    Colon,
    Arrow,
    LeftParenthesis,
    RightParenthesis,

    Plus,
    Minus,
    Star,
    Slash,
    Caret,

    If,
}

pub struct Token {
    pub value: String,
    pub token_type: TokenType,
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

    fn peek(self: &Lexer) -> char {
        self.chars[self.pos]
    }

    pub fn tokenize(src: &str) -> Vec<Token> {
        let mut lexer: Lexer = Lexer::new(src);
        let mut res: Vec<Token> = Vec::new();

        while lexer.pos < lexer.chars.len() {
            if let Some(token) = lexer.try_parse_word() {
                res.push(token);
                continue;
            }

            if let Some(token) = lexer.try_parse_one_char_symbol() {
                res.push(token);
                continue;
            }

            if let Some(token) = lexer.try_parse_two_char_symbol() {
                res.push(token);
                continue;
            }

            lexer.next();
        }

        res
    }

    fn try_parse_word(self: &mut Lexer) -> Option<Token> {
        let mut ch: char = self.chars[self.pos];

        if !(ch.is_alphabetic() || ch == '_') {
            return None;
        }

        let begin: usize = self.pos;

        let mut word: String = String::new();

        while ch.is_alphabetic() || ch == '_' {
            word.push(ch);
            self.next();
            ch = self.peek();
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

    fn try_parse_one_char_symbol(self: &mut Lexer) -> Option<Token> {
        let sym: char = self.chars[self.pos];

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
            value: sym.to_string().to_owned(),
            token_type,
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

            _ => return None,
        };

        self.next();

        Some(Token {
            value: sym.to_string().to_owned(),
            token_type,
            pos: begin,
        })
    }
}
