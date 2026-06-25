use crate::ASTParser;
use crate::Token;
use crate::error::FogError;
use crate::parser::resolved_expr::Program;

pub mod parsed_expr;
pub mod parser;
pub mod resolved_expr;

pub fn parse_program(tokens: &Vec<Token>) -> (Box<Program>, Vec<FogError>) {
    ASTParser::parse_program(&tokens)
}
