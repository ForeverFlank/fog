use crate::ASTParser;
use crate::Token;
use crate::error::FogError;
use crate::parser::nodes::Program;

pub mod nodes;
pub mod parser;

pub fn parse_program(tokens: &Vec<Token>) -> (Box<Program>, Vec<FogError>) {
    ASTParser::parse_program(&tokens)
}
