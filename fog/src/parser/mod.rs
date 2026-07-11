use std::fmt::Display;

use crate::error::FogError;
use crate::lexer::token::Token;
use crate::parser::core_expr::CoreExpr;

pub mod core_expr;
pub mod desugar;
mod parsed_expr;
pub mod parser;
mod resolved_expr;
mod resolver;

#[derive(Clone)]
pub enum Literal {
    Int32(i32),
    Float32(f32),
    Char(char),
    String(String),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Int32(value) => write!(f, "{}", value),
            Literal::Float32(value) => write!(f, "{}", value),
            Literal::Char(value) => write!(f, "{:?}", value),
            Literal::String(value) => write!(f, "{:?}", value),
        }
    }
}

pub fn parse_program(tokens: &Vec<Token>) -> (Vec<core_expr::CoreStatement>, Vec<FogError>) {
    let (parsed_stmts, parser_errors) = parser::parse(&tokens);
    let (resolved_stmts, resolver_errors) = resolver::resolve(parsed_stmts);
    let (mut desugared_stmts, desugar_errors) = desugar::desugar(resolved_stmts);

    let all_errors: Vec<FogError> = [
        &parser_errors[..],
        &resolver_errors[..],
        &desugar_errors[..],
    ]
    .concat();

    // TODO: call main

    // desugared_stmts.push(core_expr::CoreStatement::TypeAnnotation {
    //     name: "main".to_string(),
    //     expr: core_expr::CoreAtomicTypeExpr::Identifier {
    //         name: "IOUnit".to_string(),
    //         span: (),
    //     },
    //     span: (),
    // });

    // desugared_stmts.push(core_expr::CoreStatement::Expression { expr: CoreExpr::, span: () }

    (desugared_stmts, all_errors)
}
