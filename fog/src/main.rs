use std::env;
use std::fs;

use crate::error::*;
use crate::interpreter::*;
use crate::lexer::token::*;
use crate::lexer::*;
use crate::parser::parser::*;
use crate::parser::*;

mod error;
mod interpreter;
mod lexer;
mod parser;
mod util;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- arguments and paths ---
    let args: Vec<String> = env::args().collect();
    let Some(path) = args.get(1) else {
        return Err("usage: fog <path>".into());
    };

    let arg_print_tokens = args.contains(&"--print-tokens".to_string());
    // let arg_emit_ast = args.contains(&"--emit-ast".to_string());

    // --- read file ---
    let src = &fs::read_to_string(path)?;

    // -- lexing
    let (tokens, lexer_errors) = tokenize(src);

    if arg_print_tokens {
        print_tokens(&tokens);
    }

    print_errors("lexer", &lexer_errors);

    // -- parsing
    let (ast, parser_errors) = parse_program(&tokens);

    print_errors("parser", &parser_errors);

    // -- interpreting
    let res = interpret(&ast);

    if let Err(error) = res {
        match error.span {
            Some(span) => println!(
                "runtime error ({}:{}): {}",
                span.line, span.column, error.message
            ),
            None => println!("runtime error: {}", error.message),
        }
    }

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

fn print_errors(label: &str, errors: &Vec<FogError>) {
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
