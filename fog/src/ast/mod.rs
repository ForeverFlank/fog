use crate::ASTParser;
use crate::Token;
use crate::ast::nodes::Program;
use crate::error::FogError;

pub mod nodes;
pub mod parser;

pub fn parse_program(tokens: &Vec<Token>) -> (Box<Program>, Vec<FogError>) {
    ASTParser::parse_program(&tokens)
}
