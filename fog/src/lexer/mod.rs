pub mod lexer;
pub mod token;

pub fn tokenize(src: &str) -> (Vec<token::Token>, Vec<lexer::LexerError>) {
    lexer::Lexer::tokenize(src)
}
