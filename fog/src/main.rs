use std::env;
use std::fs;
use std::path;

use crate::ast_nodes::Statement::Declaration;
use crate::ast_nodes::Statement::TypeAnnotation;

mod ast_nodes;
mod ast_parser;
mod interpreter;
mod lexer;

fn main() {
    // --- arguments and paths ---
    let args: Vec<String> = env::args().collect();
    let path: &String = args.get(1).expect("Usage: ./fog <path>");

    let arg_print_tokens = args.contains(&"--print-tokens".to_string());
    let arg_emit_ast = args.contains(&"--emit-ast".to_string());

    // --- read file ---
    let src: &str = &fs::read_to_string(path).expect("Failed to read source file");

    // --- compilation ---
    // -- lexing
    let (tokens, lexer_errors) = lexer::tokenize(src);

    if arg_print_tokens {
        print_tokens(&tokens);
    }

    print_lexer_errors(&lexer_errors);

    // -- AST parsing
    let (ast, ast_parser_errors) = ast_parser::parse_program(tokens);

    if arg_emit_ast {
        let puml: String = emit_ast_puml(&ast, path);
        let output_path: path::PathBuf = path::Path::new(path).with_extension("puml");
        let _ = fs::write(output_path, puml);
    }

    print_ast_parser_errors(&ast_parser_errors);

    // -- interpreting

    interpreter::run(ast);
}

// --- Lexer ---

fn get_token_string(token: &lexer::Token) -> String {
    match &token.kind {
        lexer::TokenKind::Newline => "Newline".to_string(),
        lexer::TokenKind::Identifier(val) => format!("Identifier ({})", val),
        lexer::TokenKind::Equal => "Equal".to_string(),
        lexer::TokenKind::Colon => "Colon".to_string(),
        lexer::TokenKind::Arrow => "Arrow".to_string(),
        lexer::TokenKind::FatArrow => "FatArrow".to_string(),
        lexer::TokenKind::LeftParenthesis => "LeftParenthesis".to_string(),
        lexer::TokenKind::RightParenthesis => "RightParenthesis".to_string(),
        lexer::TokenKind::LeftBrace => "LeftBrace".to_string(),
        lexer::TokenKind::RightBrace => "RightBrace".to_string(),
        lexer::TokenKind::Comma => "Comma".to_string(),
        lexer::TokenKind::Int32Literal(val) => format!("Int ({})", val),
        lexer::TokenKind::Float32Literal(val) => format!("Float ({})", val),
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
    }
}

fn print_tokens(tokens: &Vec<lexer::Token>) {
    for token in tokens.as_slice() {
        println!(
            " {: >4}:{: >4} | {}",
            token.line,
            token.column,
            get_token_string(token)
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
    out.push_str("skinparam nodesep 30\n");
    out.push_str("skinparam ranksep 20\n");

    let prog_id: i32 = new_node(&mut out, &mut id, "Program", "#e5e5e5");

    for stmt in &program.statements {
        let stmt_id: i32 = emit_ast_puml_statement(&mut out, &mut id, stmt);
        edge(&mut out, prog_id, stmt_id);
    }

    out.push_str("@enduml\n");

    out
}

const COLOR_STATEMENT: &str = "#c2fff2";
const COLOR_IDENTIFIER: &str = "#fffdd0";
const COLOR_LITERAL: &str = "#ffcece";
const COLOR_LAMBDA: &str = "#ffc9fc";
const COLOR_FUNC_APPL: &str = "#cbe9ff";

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
            let ta_id: i32 = new_node(out, id, ":", COLOR_STATEMENT);
            let ident_id: i32 = new_node(out, id, &format!("{}", ident.0), COLOR_IDENTIFIER);
            let expr_id: i32 = emit_ast_puml_expr(out, id, expr);

            edge(out, ta_id, ident_id);
            edge(out, ta_id, expr_id);

            ta_id
        }
        Declaration(ident, expr) => {
            let decl_id: i32 = new_node(out, id, "=", COLOR_STATEMENT);
            let ident_id: i32 = new_node(out, id, &format!("{}", ident.0), COLOR_IDENTIFIER);
            let expr_id: i32 = emit_ast_puml_expr(out, id, expr);

            edge(out, decl_id, ident_id);
            edge(out, decl_id, expr_id);

            decl_id
        }
    }
}

fn emit_ast_puml_expr(out: &mut String, id: &mut i32, expr: &ast_nodes::Expr) -> i32 {
    match expr {
        ast_nodes::Expr::Int32Literal(val) => new_node(out, id, &format!("{}", val), COLOR_LITERAL),
        ast_nodes::Expr::Float32Literal(val) => {
            new_node(out, id, &format!("{}", val), COLOR_LITERAL)
        }
        ast_nodes::Expr::StringLiteral(val) => {
            new_node(out, id, &format!("{}", val), COLOR_LITERAL)
        }
        ast_nodes::Expr::Identifier(name) => {
            new_node(out, id, &format!("{}", name), COLOR_IDENTIFIER)
        }
        ast_nodes::Expr::Lambda {
            parameter_name,
            body,
        } => {
            let lambda_id: i32 = new_node(out, id, "λ", COLOR_LAMBDA);
            let param_id: i32 = new_node(out, id, &format!("{}", parameter_name), COLOR_IDENTIFIER);
            let body_id: i32 = emit_ast_puml_expr(out, id, &body);

            edge(out, lambda_id, param_id);
            edge(out, lambda_id, body_id);
            lambda_id
        }
        ast_nodes::Expr::FuncAppl {
            function_name,
            arguments,
        } => {
            let appl_id: i32 = new_node(out, id, &format!("{}", function_name), COLOR_FUNC_APPL);
            for arg in arguments {
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
            "Error: {} at {} at {}:{}",
            error.message,
            get_token_string(&error.token),
            error.token.line,
            error.token.column
        )
    }
}
