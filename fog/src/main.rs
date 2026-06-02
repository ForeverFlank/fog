use std::env;
use std::fs;
mod ast_nodes;
mod ast_parser;
mod lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path: &String = args.get(1).expect("Usage: ./fog <path>");

    let src: &str = &fs::read_to_string(path).expect("Failed to read source file");

    let (tokens, lexerErrors) = lexer::tokenize(src);

    for token in tokens {
        let token_type_name: String = match token.kind {
            lexer::TokenKind::Newline => "Newline".to_string(),
            lexer::TokenKind::Identifier(val) => format!("Identifier ({})", val),
            lexer::TokenKind::Equal => "Equal".to_string(),
            lexer::TokenKind::Colon => "Colon".to_string(),
            lexer::TokenKind::Arrow => "Arrow".to_string(),
            lexer::TokenKind::LeftParenthesis => "LeftParenthesis".to_string(),
            lexer::TokenKind::RightParenthesis => "RightParenthesis".to_string(),
            lexer::TokenKind::LeftBrace => "LeftBrace".to_string(),
            lexer::TokenKind::RightBrace => "RightBrace".to_string(),
            lexer::TokenKind::Comma => "Comma".to_string(),
            lexer::TokenKind::IntLiteral(val) => format!("Int ({})", val),
            lexer::TokenKind::FloatLiteral(val) => format!("Float ({})", val),
            // lexer::TokenKind::StringLiteral(val) => format!("String ({})", val),
            lexer::TokenKind::Plus => "Plus".to_string(),
            lexer::TokenKind::Minus => "Minus".to_string(),
            lexer::TokenKind::Star => "Star".to_string(),
            lexer::TokenKind::Slash => "Slash".to_string(),
            lexer::TokenKind::Caret => "Caret".to_string(),
            lexer::TokenKind::Concat => "Concat".to_string(),
            lexer::TokenKind::LeftPipe => "LeftPipe".to_string(),
            lexer::TokenKind::RightPipe => "RightPipe".to_string(),
            lexer::TokenKind::LeftComposition => "LeftComposition".to_string(),
            lexer::TokenKind::RightComposition => "RightComposition".to_string(),
            lexer::TokenKind::If => "If".to_string(),
        };

        println!(
            " {: >4}:{: >4} | {}",
            token.line, token.column, token_type_name
        )
    }
}
