use std::env;
use std::fs;
use std::path;

use crate::ast_nodes::Statement::Declaration;
use crate::ast_nodes::Statement::TypeAnnotation;
mod ast_nodes;
mod ast_parser;
mod lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path: &String = args.get(1).expect("Usage: ./fog <path>");

    let src: &str = &fs::read_to_string(path).expect("Failed to read source file");

    let (tokens, lexer_errors) = lexer::tokenize(src);

    print_tokens(&tokens);
    print_lexer_errors(&lexer_errors);

    let (ast, ast_parser_errors) = ast_parser::parse_program(tokens);

    let puml: String = emit_ast_puml(&ast, path);
    let output_path: path::PathBuf = path::Path::new(path).with_extension("puml");
    let _ = fs::write(output_path, puml);
    print_ast_parser_errors(&ast_parser_errors);
}

// --- Lexer ---

fn print_tokens(tokens: &Vec<lexer::Token>) {
    for token in tokens.as_slice() {
        let token_type_name: &str = match &token.kind {
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
}

fn print_lexer_errors(errors: &Vec<lexer::LexerError>) {
    for error in errors {
        println!(
            "Error: {} at {}:{}",
            error.message, error.line, error.column
        )
    }
}

// --- AST ---

fn emit_ast_puml(program: &ast_nodes::Program, path: &str) -> String {
    let mut out: String = String::new();
    let mut id: i32 = 0;
    out.push_str(&format!("@startuml AST of {}\n", path));
    out.push_str(&format!("title AST of {}\n", path));

    let prog_id: i32 = new_node(&mut out, &mut id, "Program", "#e5e5e5");

    for stmt in &program.statements {
        let stmt_id: i32 = emit_ast_puml_statement(&mut out, &mut id, stmt);
        edge(&mut out, prog_id, stmt_id);
    }

    out.push_str("@enduml\n");

    out
}

const COLOR_STATEMENT: &str = "#aef1eb";
const COLOR_IDENTIFIER: &str = "#fffc9f";
const COLOR_LITERAL: &str = "#ffb0b0";
const COLOR_LAMBDA: &str = "#ade4ff";
const COLOR_FUNC_APPL: &str = "#bcafff";

fn new_node(out: &mut String, id: &mut i32, label: &str, color: &str) -> i32 {
    let this_id: i32 = *id;
    out.push_str(&format!(
        "rectangle \"{}\" as n{} {}\n",
        label, this_id, color
    ));
    *id += 1;
    this_id
}

fn edge(out: &mut String, parent_id: i32, child_id: i32) {
    out.push_str(&format!("n{} <-- n{}\n", parent_id, child_id));
}

fn emit_ast_puml_statement(out: &mut String, id: &mut i32, stmt: &ast_nodes::Statement) -> i32 {
    match stmt {
        TypeAnnotation(ident, expr) => {
            let ta_id: i32 = new_node(out, id, "Type Annotation", COLOR_STATEMENT);
            let ident_id: i32 = new_node(
                out,
                id,
                &format!("Identifier: {}", ident.0),
                COLOR_IDENTIFIER,
            );
            let expr_id: i32 = emit_ast_puml_expr(out, id, expr);

            edge(out, ta_id, ident_id);
            edge(out, ta_id, expr_id);

            ta_id
        }
        Declaration(ident, expr) => {
            let decl_id: i32 = new_node(out, id, "Declaration", COLOR_STATEMENT);
            let ident_id: i32 = new_node(
                out,
                id,
                &format!("Identifier: {}", ident.0),
                COLOR_IDENTIFIER,
            );
            let expr_id: i32 = emit_ast_puml_expr(out, id, expr);

            edge(out, decl_id, ident_id);
            edge(out, decl_id, expr_id);

            decl_id
        }
    }
}

fn emit_ast_puml_expr(out: &mut String, id: &mut i32, expr: &ast_nodes::Expr) -> i32 {
    match expr {
        ast_nodes::Expr::IntLiteral(val) => {
            new_node(out, id, &format!("Int: {}", val), COLOR_LITERAL)
        }
        ast_nodes::Expr::FloatLiteral(val) => {
            new_node(out, id, &format!("Float: {}", val), COLOR_LITERAL)
        }
        ast_nodes::Expr::StringLiteral(val) => {
            new_node(out, id, &format!("String: {}", val), COLOR_LITERAL)
        }
        ast_nodes::Expr::Identifier(ident) => new_node(
            out,
            id,
            &format!("Identifier: {}", ident.0),
            COLOR_IDENTIFIER,
        ),
        ast_nodes::Expr::Lambda(lambda) => {
            let lambda_id: i32 = new_node(out, id, "Lambda", COLOR_LAMBDA);
            let param_id: i32 = new_node(
                out,
                id,
                &format!("Param: {}", lambda.parameter.0),
                COLOR_IDENTIFIER,
            );
            let body_id: i32 = emit_ast_puml_expr(out, id, &lambda.body);

            edge(out, lambda_id, param_id);
            edge(out, lambda_id, body_id);
            lambda_id
        }
        ast_nodes::Expr::FuncAppl(appl) => {
            let appl_id: i32 = new_node(
                out,
                id,
                &format!("FuncAppl: {}", appl.function.0),
                COLOR_FUNC_APPL,
            );
            for arg in &appl.arguments {
                let arg_id = emit_ast_puml_expr(out, id, arg);
                edge(out, appl_id, arg_id);
            }
            appl_id
        }
    }
}

fn print_ast_parser_errors(errors: &Vec<ast_parser::ASTParserError>) {
    for error in errors {
        println!(
            "Error: {} at {}:{}",
            error.message, error.token.line, error.token.column
        )
    }
}
