use std::env;
use std::fs;
mod lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path: &String = &args[0];

    let src: String = fs::read_to_string(path).expect("Failed to read source file");

    let preprocessed: String = lexer::preprocess(src);
    let tokens: Vec<lexer::Token> = lexer::tokenize(preprocessed);
}
