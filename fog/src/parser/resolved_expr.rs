use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use crate::error::Span;

pub struct Program {
    pub statements: Vec<Statement>,
}

pub enum Statement {
    TypeAnnotation(String, ResolvedExpr, Span),
    Declaration(String, ResolvedExpr, Span),
}

pub enum ResolvedExpr {
    Identifier(String),

    Int32Literal(i32),
    Float32Literal(f32),

    Lambda {
        param_name: String,
        param_type: Box<ResolvedExpr>,
        body: Rc<ResolvedExpr>,
    },

    Tuple(Vec<ResolvedExpr>),

    FuncAppl(String, Vec<ResolvedExpr>),
    DataConstructor(String, Vec<ResolvedExpr>),
}

impl ResolvedExpr {
    fn fmt_parenthesized(expr: &ResolvedExpr, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "({contents})")
            }

            ResolvedExpr::Lambda {
                param_name: param,
                body,
                ..
            } => {
                write!(f, "{param} => {body}")
            }

            ResolvedExpr::FuncAppl(fn_name, args) => {
                write!(f, "{fn_name}")?;

                for arg in args {
                    write!(f, " ")?;
                    Self::fmt_parenthesized(arg, f)?;
                }

                Ok(())
            }

            ResolvedExpr::DataConstructor(name, args) => {
                write!(f, "{name}")?;

                for arg in args {
                    write!(f, " ")?;
                    Self::fmt_parenthesized(arg, f)?;
                }

                Ok(())
            }
        }
    }
}
