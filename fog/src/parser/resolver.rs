use std::collections::HashMap;

use crate::{
    error::FogError,
    parser::{parsed_expr::ParsedStatement, resolved_expr::ResolvedStatement},
};

pub struct Resolver {
    infix_ops: HashMap<String, InfixOp>,
}

#[derive(Clone)]
pub enum OpAssociativity {
    Left,
    Right,
}

pub struct InfixOp {
    pub name: String,
    pub associativity: OpAssociativity,
    pub precedence: i32,
}

// fn token_op_key(token: &Token) -> Option<&str> {
//     match &token.kind {
//         TokenKind::Plus => Some("+"),
//         TokenKind::Minus => Some("-"),
//         TokenKind::Star => Some("*"),
//         TokenKind::Slash => Some("/"),
//         TokenKind::Arrow => Some("->"),
//         TokenKind::Identifier(name) => Some(name.as_str()),
//         _ => None,
//     }
// }

impl Resolver {
    fn new() -> Self {
        Self {
            infix_ops: [
                ("*", OpAssociativity::Left, 3),
                ("/", OpAssociativity::Left, 3),
                ("+", OpAssociativity::Left, 2),
                ("-", OpAssociativity::Left, 2),
                ("->", OpAssociativity::Right, 1),
            ]
            .iter()
            .map(|(sym, assoc, prec)| {
                (
                    sym.to_string(),
                    InfixOp {
                        name: sym.to_string(),
                        associativity: assoc.clone(),
                        precedence: *prec,
                    },
                )
            })
            .collect(),,
        }
    }

    pub fn resolve(
        parsed_statements: Vec<ParsedStatement>,
    ) -> (Vec<ResolvedStatement>, Vec<FogError>) {
    }
}
