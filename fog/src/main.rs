use std::env;
use std::fs;
mod ast_nodes;
mod ast_parser;
mod lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path: &String = args.get(1).expect("Usage: ./fog <path>");

    let src: &str = &fs::read_to_string(path).expect("Failed to read source file");

    let (tokens, lexer_errors) = lexer::tokenize(src);

    for token in tokens {
        let token_type_name: &str = match token.kind {
            lexer::TokenKind::Newline => "Newline",
            lexer::TokenKind::Identifier(val) => &format!("Identifier ({})", val),
            lexer::TokenKind::Equal => "Equal",
            lexer::TokenKind::Colon => "Colon",
            lexer::TokenKind::Arrow => "Arrow",
            lexer::TokenKind::FatArrow => "FatArrow",
            lexer::TokenKind::LeftParenthesis => "LeftParenthesis",
            lexer::TokenKind::RightParenthesis => "RightParenthesis",
            lexer::TokenKind::LeftBrace => "LeftBrace",
            lexer::TokenKind::RightBrace => "RightBrace",
            lexer::TokenKind::Comma => "Comma",
            lexer::TokenKind::IntLiteral(val) => &format!("Int ({})", val),
            lexer::TokenKind::FloatLiteral(val) => &format!("Float ({})", val),
            // lexer::TokenKind::StringLiteral(val) => format!("String ({})", val),
            lexer::TokenKind::Plus => "Plus",
            lexer::TokenKind::Minus => "Minus",
            lexer::TokenKind::Star => "Star",
            lexer::TokenKind::Slash => "Slash",
            lexer::TokenKind::Caret => "Caret",
            lexer::TokenKind::Concat => "Concat",
            lexer::TokenKind::LeftPipe => "LeftPipe",
            lexer::TokenKind::RightPipe => "RightPipe",
            lexer::TokenKind::LeftComposition => "LeftComposition",
            lexer::TokenKind::RightComposition => "RightComposition",
            lexer::TokenKind::If => "If",
        };

        println!(
            " {: >4}:{: >4} | {}",
            token.line, token.column, token_type_name
        )
    }

    for error in lexer_errors {
        println!(
            "Error: {} at {}:{}",
            error.message, error.line, error.column
        )
    }
}
