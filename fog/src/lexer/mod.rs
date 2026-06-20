use crate::error::FogError;

pub mod lexer;
pub mod token;

pub fn tokenize(src: &str) -> (Vec<token::Token>, Vec<FogError>) {
    lexer::Lexer::tokenize(src)
}
