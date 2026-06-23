use std::rc::Rc;

use crate::error::Span;

// --- AST nodes ---

pub struct Program {
    pub statements: Vec<Statement>,
}

pub enum Statement {
    TypeAnnotation(String, Expr, Span),
    Declaration(String, Expr, Span),
}

// --- expressions ---

pub enum Expr {
    Identifier(String),
    Lambda {
        param: String,
        param_type: Box<Expr>,
        body: Rc<Expr>,
    },
    FuncAppl {
        function: String,
        args: Vec<Box<Expr>>,
    },
    Int32Literal(i32),
    Float32Literal(f32),
    // StringLiteral(String),
}

impl ToString for Expr {
    fn to_string(&self) -> String {
        match self {
            Expr::Identifier(name) => name.clone(),
            Expr::Lambda { param, body, .. } => {
                format!("{} => {}", param, body.to_string())
            }
            Expr::FuncAppl {
                function: function_name,
                args: arguments,
            } => {
                let args: String = arguments
                    .iter()
                    .map(|arg| {
                        let str: String = arg.to_string();
                        if str.contains(' ') {
                            format!("({})", str)
                        } else {
                            str
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ");
                format!("{} {}", function_name, args)
            }
            Expr::Int32Literal(value) => value.to_string(),
            Expr::Float32Literal(value) => value.to_string(),
            // Expr::StringLiteral(value) => format!("\"{}\"", value),
        }
    }
}
