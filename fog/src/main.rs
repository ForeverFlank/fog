use std::env;
use std::fs;

use crate::error::*;
use crate::lexer::token::*;
use crate::lexer::*;
use crate::parser::*;

mod anf;
mod core;
mod error;
mod interpreter;
mod lexer;
mod optimizer;
mod parser;
mod static_check;
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

    // -- lex
    let (tokens, lexer_errors) = tokenize(src);
    print_errors(&lexer_errors);

    if arg_print_tokens {
        print_tokens(&tokens);
    }

    // -- parse
    let (top_stmts, parser_errors) = parse_program(&tokens);
    print_errors(&parser_errors);

    // for stmt in top_stmts.as_slice() {
    //     println!("{stmt}");
    // }

    // -- static check
    let static_check_errors = static_check::static_check(&top_stmts);
    print_errors(&static_check_errors);

    // -- ANF normalize

    let (anfs, anf_metadata) = anf::anf_parser::parse_anf(&top_stmts);

    for anf in anfs.as_slice() {
        println!("{anf}");
    }

    // TODO it's an interpreter causing stack overflow??

    // -- optimize

    // let  = optimize(top_stmts);
    // print_errors("optimizer", &parser_errors);

    if !lexer_errors.is_empty() || !parser_errors.is_empty() || !static_check_errors.is_empty()
    /* || !optimizer_errors.is_empty() */
    {
        return Err("syntax error found".into());
    }

    // -- interpret
    let res = interpreter::interpret(&anfs, &anf_metadata);

    if let Err(error) = res {
        match error.span {
            Some(span) => eprintln!(
                "runtime error ({}:{}): {}",
                span.start.line, span.start.column, error.message
            ),
            None => eprintln!("runtime error: {}", error.message),
        }
    }

    Ok(())
}

// --- lexer ---

fn print_tokens(tokens: &Vec<Token>) {
    for token in tokens.as_slice() {
        eprintln!(
            " {: >4}:{: >4} | {}",
            token.span.start.line,
            token.span.start.column,
            token.kind.to_string()
        )
    }
}

fn print_errors(errors: &Vec<FogError>) {
    for error in errors {
        eprintln!("{error}");
    }
}
