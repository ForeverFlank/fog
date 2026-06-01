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
}

pub struct Token {
    pub token_type: TokenType,
}

pub fn preprocess(src: String) -> String {
    return src;
}

pub fn tokenize(preprocessed: String) -> Vec<Token> {
    let chars: Vec<char> = preprocessed.chars().collect();
    let pos: usize = 0;
    let mut res: Vec<Token> = Vec::new();

    return res;
}
