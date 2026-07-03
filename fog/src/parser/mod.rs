use crate::Token;
use crate::error::FogError;

mod desugar;
pub mod desugared_expr;
mod parsed_expr;
pub mod parser;
pub mod resolved_expr;
pub mod resolver;

pub fn parse_program(
    tokens: &Vec<Token>,
) -> (Vec<desugared_expr::DesugaredStatement>, Vec<FogError>) {
    let (parsed_stmts, parser_errors) = parser::parse(&tokens);
    let (resolved_stmts, resolver_errors) = resolver::resolve(parsed_stmts);
    let (desugared_stmts, desugar_errors) = desugar::desugar(resolved_stmts);

    let all_errors: Vec<FogError> = [
        &parser_errors[..],
        &resolver_errors[..],
        &desugar_errors[..],
    ]
    .concat();

    (desugared_stmts, all_errors)
}
