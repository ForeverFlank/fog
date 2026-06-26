use crate::Parser;
use crate::Token;
use crate::error::FogError;
use crate::parser::resolved_expr::ResolvedStatement;
use crate::parser::resolver::Resolver;

mod parsed_expr;
pub mod parser;
pub mod resolved_expr;
pub mod resolver;

pub fn parse_program(tokens: &Vec<Token>) -> (Vec<ResolvedStatement>, Vec<FogError>) {
    let (parsed_stmts, parser_errors) = Parser::parse(&tokens);
    let (resolved_stmts, resolver_errors) = Resolver::resolve(parsed_stmts);

    let all_errors: Vec<FogError> = [&parser_errors[..], &resolver_errors[..]].concat();

    (resolved_stmts, all_errors)
}
