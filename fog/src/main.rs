use std::env;
use std::fs;
use std::path;

use crate::error::*;
use crate::interpreter::*;
use crate::lexer::token::*;
use crate::lexer::*;
use crate::parser::parser::*;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::parser::resolved_expr::ResolvedStatement::*;
use crate::parser::*;

mod error;
mod interpreter;
mod lexer;
mod parser;
mod util;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- arguments and paths ---
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).expect("usage: ./fog <path>");

    let arg_print_tokens = args.contains(&"--print-tokens".to_string());
    let arg_emit_ast = args.contains(&"--emit-ast".to_string());

    // --- read file ---
    let src = &fs::read_to_string(path)?;

    // --- compilation ---
    // -- lexing
    let (tokens, lexer_errors) = tokenize(src);

    if arg_print_tokens {
        print_tokens(&tokens);
    }

    print_errors("lexer", &lexer_errors);

    // -- AST parsing
    let (ast, parser_errors) = parse_program(&tokens);

    if arg_emit_ast {
        let puml = emit_ast_puml(&ast, path);
        let output_path = path::Path::new(path).with_extension("puml");
        let _ = fs::write(output_path, puml);
    }

    print_errors("parser", &parser_errors);

    // interpreting

    interpret(ast);

    Ok(())
}

// --- Lexer ---

fn print_tokens(tokens: &Vec<Token>) {
    for token in tokens.as_slice() {
        println!(
            " {: >4}:{: >4} | {}",
            token.line,
            token.column,
            token.kind.to_string()
        )
    }
}

fn print_errors(label: &str, errors: &[FogError]) {
    for error in errors {
        match error.span.as_ref() {
            Some(span) => println!(
                "{label} error ({}:{}): {}",
                span.line, span.column, error.message
            ),
            None => println!("{label} error: {}", error.message),
        }
    }
}

// --- AST ---

fn emit_ast_puml(statements: &Vec<parser::resolved_expr::ResolvedStatement>, path: &str) -> String {
    let mut out = String::new();
    let mut id = 0i32;
    out.push_str(&format!("@startuml AST of {}\n", path));
    out.push_str(&format!("title AST of {}\n", path));
    out.push_str("skinparam nodesep 30\n");
    out.push_str("skinparam ranksep 20\n");

    let prog_id = new_node(&mut out, &mut id, "Program", "#e5e5e5");

    for stmt in statements {
        let stmt_id = emit_ast_puml_statement(&mut out, &mut id, &stmt);
        edge(&mut out, prog_id, stmt_id);
    }

    out.push_str("@enduml\n");

    out
}

const COLOR_STATEMENT: &str = "#a0f5e5";
const COLOR_IDENTIFIER: &str = "#fff07a";
const COLOR_LITERAL: &str = "#ffaaaa";
const COLOR_LAMBDA: &str = "#e0a8ff";
const COLOR_FUNC_APPL: &str = "#8ecfff";
const COLOR_TUPLE: &str = "#ffcc88";

fn new_node(out: &mut String, id: &mut i32, label: &str, color: &str) -> i32 {
    let this_id = *id;
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

fn emit_ast_puml_statement(
    out: &mut String,
    id: &mut i32,
    stmt: &parser::resolved_expr::ResolvedStatement,
) -> i32 {
    match stmt {
        TypeAnnotation { name, expr, .. } => {
            emit_ast_puml_named_stmt(out, id, "TypeAnnotation", name, expr)
        }
        Declaration { name, expr, .. } => {
            emit_ast_puml_named_stmt(out, id, "Declaration", name, expr)
        }
    }
}

fn emit_ast_puml_named_stmt(
    out: &mut String,
    id: &mut i32,
    label: &str,
    name: &str,
    expr: &ResolvedExpr,
) -> i32 {
    let node_id = new_node(out, id, label, COLOR_STATEMENT);
    let ident_id = new_node(out, id, name, COLOR_IDENTIFIER);
    let expr_id = emit_ast_puml_expr(out, id, expr);

    edge(out, node_id, ident_id);
    edge(out, node_id, expr_id);

    node_id
}

fn emit_ast_puml_expr(
    out: &mut String,
    id: &mut i32,
    expr: &parser::resolved_expr::ResolvedExpr,
) -> i32 {
    match expr {
        parser::resolved_expr::ResolvedExpr::Identifier { name } => {
            new_node(out, id, &format!("Identifier ({})", name), COLOR_IDENTIFIER)
        }

        parser::resolved_expr::ResolvedExpr::Int32Literal { value } => {
            new_node(out, id, &format!("Int32 {}", value), COLOR_LITERAL)
        }
        parser::resolved_expr::ResolvedExpr::Float32Literal { value } => {
            new_node(out, id, &format!("Float32 ({})", value), COLOR_LITERAL)
        }

        parser::resolved_expr::ResolvedExpr::Lambda {
            param_name,
            param_type,
            body,
        } => {
            let lambda_id = new_node(out, id, "Lambda", COLOR_LAMBDA);
            let param_id =
                emit_ast_puml_named_stmt(out, id, "TypeAnnotation", param_name, param_type);
            let body_id = emit_ast_puml_expr(out, id, body);

            edge(out, lambda_id, param_id);
            edge(out, lambda_id, body_id);
            lambda_id
        }

        parser::resolved_expr::ResolvedExpr::Tuple { exprs } => {
            let tuple_id = new_node(out, id, "Tuple", COLOR_TUPLE);
            for expr in exprs {
                let expr_id = emit_ast_puml_expr(out, id, expr);
                edge(out, tuple_id, expr_id);
            }
            tuple_id
        }

        parser::resolved_expr::ResolvedExpr::FuncAppl { fn_name, args } => {
            let appl_id =
                new_node(out, id, &format!("FuncAppl ({})", fn_name), COLOR_FUNC_APPL);
            for arg in args {
                let arg_id = emit_ast_puml_expr(out, id, arg);
                edge(out, appl_id, arg_id);
            }
            appl_id
        }
    }
}
