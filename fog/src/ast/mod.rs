use crate::ASTParser;
use crate::ASTParserError;
use crate::Token;
use crate::ast::nodes::Program;

pub mod nodes;
pub mod parser;

pub fn parse_program(tokens: &Vec<Token>) -> (Box<Program>, Vec<ASTParserError>) {
    ASTParser::parse_program(&tokens)
}
