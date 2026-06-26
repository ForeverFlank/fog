use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use crate::error::Span;

pub enum ResolvedStatement {
    TypeAnnotation(String, ResolvedExpr, Span),
    Declaration(String, ResolvedExpr, Span),
}

pub enum ResolvedExpr {
    Identifier(String),

    Int32Literal(i32),
    Float32Literal(f32),

    Lambda(String, Box<ResolvedExpr>, Rc<ResolvedExpr>),

    Tuple(Vec<ResolvedExpr>),

    FuncAppl(String, Vec<ResolvedExpr>),
}

impl ResolvedExpr {
    fn fmt_parenthesized(f: &mut fmt::Formatter<'_>, expr: &ResolvedExpr) -> fmt::Result {
        let s: String = expr.to_string();

        if s.contains(' ') {
            write!(f, "({s})")
        } else {
            write!(f, "{s}")
        }
    }
}

impl Display for ResolvedExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolvedExpr::Identifier(name) => write!(f, "{name}"),

            ResolvedExpr::Int32Literal(value) => write!(f, "{value}"),
            ResolvedExpr::Float32Literal(value) => write!(f, "{value}"),

            ResolvedExpr::Tuple(exprs) => {
                let contents: String = exprs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ");

                write!(f, "({contents})")
            }

            ResolvedExpr::Lambda(param_name, _, body) => {
                write!(f, "{param_name} => {body}")
            }

            ResolvedExpr::FuncAppl(fn_name, args) => {
                write!(f, "{fn_name}")?;

                for arg in args {
                    write!(f, " ")?;
                    Self::fmt_parenthesized(f, arg)?;
                }

                Ok(())
            }
        }
    }
}
