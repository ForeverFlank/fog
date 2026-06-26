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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- arguments and paths ---
    let args: Vec<String> = env::args().collect();
    let path: &String = args.get(1).expect("usage: ./fog <path>");

    let arg_print_tokens: bool = args.contains(&"--print-tokens".to_string());
    let arg_emit_ast: bool = args.contains(&"--emit-ast".to_string());

    // --- read file ---
    let src: &str = &fs::read_to_string(path)?;

    // --- compilation ---
    // -- lexing
    let (tokens, lexer_errors) = tokenize(src);

    if arg_print_tokens {
        print_tokens(&tokens);
    }

    print_lexer_errors(&lexer_errors);

    // -- AST parsing
    let (ast, parser_errors) = parse_program(&tokens);

    if arg_emit_ast {
        let puml: String = emit_ast_puml(&ast, path);
        let output_path: path::PathBuf = path::Path::new(path).with_extension("puml");
        let _ = fs::write(output_path, puml);
    }

    print_parser_errors(&parser_errors);

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

fn print_lexer_errors(errors: &Vec<FogError>) {
    for error in errors {
        let span = error.span.as_ref();
        match span {
            Some(span) => println!(
                "lexer error ({}:{}): {}",
                span.line, span.column, error.message
            ),
            None => println!("lexer error: {}", error.message),
        }
    }
}

// --- AST ---

fn emit_ast_puml(statements: &Vec<parser::resolved_expr::ResolvedStatement>, path: &str) -> String {
    let mut out: String = String::new();
    let mut id: i32 = 0;
    out.push_str(&format!("@startuml AST of {}\n", path));
    out.push_str(&format!("title AST of {}\n", path));
    out.push_str("skinparam nodesep 30\n");
    out.push_str("skinparam ranksep 20\n");

    let prog_id: i32 = new_node(&mut out, &mut id, "Program", "#e5e5e5");

    for stmt in statements {
        let stmt_id: i32 = emit_ast_puml_statement(&mut out, &mut id, &stmt);
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

fn emit_ast_puml_statement(
    out: &mut String,
    id: &mut i32,
    stmt: &parser::resolved_expr::ResolvedStatement,
) -> i32 {
    match stmt {
        TypeAnnotation(name, expr, _) => emit_ast_puml_type_annotation(out, id, name, expr),
        Declaration(name, expr, _) => emit_ast_puml_declaration(out, id, name, expr),
    }
}

fn emit_ast_puml_type_annotation(
    out: &mut String,
    id: &mut i32,
    name: &String,
    expr: &ResolvedExpr,
) -> i32 {
    let ta_id: i32 = new_node(out, id, "TypeAnnotation", COLOR_STATEMENT);
    let ident_id: i32 = new_node(out, id, &format!("{}", name), COLOR_IDENTIFIER);
    let expr_id: i32 = emit_ast_puml_expr(out, id, expr);

    edge(out, ta_id, ident_id);
    edge(out, ta_id, expr_id);

    ta_id
}

fn emit_ast_puml_declaration(
    out: &mut String,
    id: &mut i32,
    name: &String,
    expr: &ResolvedExpr,
) -> i32 {
    let decl_id: i32 = new_node(out, id, "Declaration", COLOR_STATEMENT);
    let ident_id: i32 = new_node(out, id, &format!("{}", name), COLOR_IDENTIFIER);
    let expr_id: i32 = emit_ast_puml_expr(out, id, expr);

    edge(out, decl_id, ident_id);
    edge(out, decl_id, expr_id);

    decl_id
}

fn emit_ast_puml_expr(
    out: &mut String,
    id: &mut i32,
    expr: &parser::resolved_expr::ResolvedExpr,
) -> i32 {
    match expr {
        parser::resolved_expr::ResolvedExpr::Identifier(name) => {
            new_node(out, id, &format!("Identifier ({})", name), COLOR_IDENTIFIER)
        }

        parser::resolved_expr::ResolvedExpr::Int32Literal(val) => {
            new_node(out, id, &format!("Int32 {}", val), COLOR_LITERAL)
        }
        parser::resolved_expr::ResolvedExpr::Float32Literal(val) => {
            new_node(out, id, &format!("Float32 ({})", val), COLOR_LITERAL)
        }

        parser::resolved_expr::ResolvedExpr::Lambda(param_name, param_type, body) => {
            let lambda_id: i32 = new_node(out, id, "Lambda", COLOR_LAMBDA);
            let param_id: i32 = emit_ast_puml_type_annotation(out, id, param_name, param_type);
            let body_id: i32 = emit_ast_puml_expr(out, id, &body);

            edge(out, lambda_id, param_id);
            edge(out, lambda_id, body_id);
            lambda_id
        }

        parser::resolved_expr::ResolvedExpr::Tuple(exprs) => {
            let tuple_id: i32 = new_node(out, id, "Tuple", COLOR_IDENTIFIER);
            for expr in exprs {
                let expr_id: i32 = emit_ast_puml_expr(out, id, expr);
                edge(out, tuple_id, expr_id);
            }
            tuple_id
        }

        parser::resolved_expr::ResolvedExpr::FuncAppl(fn_name, args) => {
            let appl_id: i32 =
                new_node(out, id, &format!("FuncAppl ({})", fn_name), COLOR_FUNC_APPL);
            for arg in args {
                let arg_id: i32 = emit_ast_puml_expr(out, id, arg);
                edge(out, appl_id, arg_id);
            }
            appl_id
        }
    }
}

fn print_parser_errors(errors: &Vec<FogError>) {
    for error in errors {
        let span = error.span.as_ref();
        match span {
            Some(span) => {
                println!(
                    "parser error ({}:{}): {}",
                    span.line, span.column, error.message,
                )
            }
            None => println!("parser error: {}", error.message),
        }
    }
}
