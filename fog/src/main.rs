use std::env;
use std::fs;
mod lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path: &String = args.get(1).expect("Usage: ./fog <path>");

    let src: &str = &fs::read_to_string(path).expect("Failed to read source file");

    let (tokens, _) = lexer::tokenize(src);

    for token in tokens {
        let token_type_name: &str = match token.kind {
            lexer::TokenKind::Newline => "NEWLINE",
            lexer::TokenKind::Identifier(val) => &format!("IDENT ({})", val),
            lexer::TokenKind::Equal => "EQUAL",
            lexer::TokenKind::Colon => "COLON",
            lexer::TokenKind::Arrow => "ARROW",
            lexer::TokenKind::LeftParenthesis => "LPAREN",
            lexer::TokenKind::RightParenthesis => "RPAREN",
            lexer::TokenKind::Comma => "COMMA",
            lexer::TokenKind::IntLiteral(val) => &format!("INT ({})", val),
            lexer::TokenKind::FloatLiteral(val) => &format!("FLOAT ({})", val),
            // lexer::TokenKind::StringLiteral(val) => &format!("STRING ({})", val),
            lexer::TokenKind::Plus => "PLUS",
            lexer::TokenKind::Minus => "MINUS",
            lexer::TokenKind::Star => "STAR",
            lexer::TokenKind::Slash => "SLASH",
            lexer::TokenKind::Caret => "CARET",
            lexer::TokenKind::Bar => "BAR",
            lexer::TokenKind::Concat => "CONCAT",
            lexer::TokenKind::LeftPipe => "LPIPE",
            lexer::TokenKind::RightPipe => "RPIPE",
            lexer::TokenKind::LeftComposition => "LCOMPOSE",
            lexer::TokenKind::RightComposition => "RCOMPOSE",
            lexer::TokenKind::If => "IF",
        };

        println!(
            " {: >4}:{: >4} | {}",
            token.line, token.column, token_type_name
        )
    }
}
