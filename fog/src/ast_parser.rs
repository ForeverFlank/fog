use crate::{ast_nodes::Program, lexer::Token};

pub fn parse_program(tokens: Vec<Token>) -> Program {
    Program { items: vec![] }
}
