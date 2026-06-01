use std::env;
use std::fs;
mod lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path: &String = args.get(1).expect("Usage: ./fog <path>");

    let src: &str = &fs::read_to_string(path).expect("Failed to read source file");

    let tokens: Vec<lexer::Token> = lexer::tokenize(src);

    for token in tokens {
        let token_type_name: &str = match token.token_type {
            lexer::TokenType::Terminator => "TERMINATOR",
            lexer::TokenType::Identifier => "IDENTIFIER",
            lexer::TokenType::Equal => "EQUAL",
            lexer::TokenType::Colon => "COLON",
            lexer::TokenType::Arrow => "ARROW",
            lexer::TokenType::LeftParenthesis => "LPAREN",
            lexer::TokenType::RightParenthesis => "RPAREN",
            lexer::TokenType::IntLiteral => "INT_LITERAL",
            lexer::TokenType::FloatLiteral => "FLOAT_LITERAL",
            lexer::TokenType::StringLiteral => "STRING_LITERAL",
            lexer::TokenType::Plus => "PLUS",
            lexer::TokenType::Minus => "MINUS",
            lexer::TokenType::Star => "STAR",
            lexer::TokenType::Slash => "SLASH",
            lexer::TokenType::Caret => "CARET",
            lexer::TokenType::Concat => "CONCAT",
            lexer::TokenType::LeftPipe => "LPIPE",
            lexer::TokenType::RightPipe => "RPIPE",
            lexer::TokenType::LeftComposition => "LCOMPOSE",
            lexer::TokenType::RightComposition => "RCOMPOSE",
            lexer::TokenType::If => "IF",
        };

        println!(
            "{: >15} | {: >3} | {}",
            token_type_name, token.pos, token.value
        )
    }
}
