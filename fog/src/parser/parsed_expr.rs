use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use crate::error::Span;

// --- statement ---

pub enum ParsedStatement {
    TypeAnnotation(String, ParsedExpr, Span),
    Declaration(String, ParsedExpr, Span),
}

// --- operator kind ---

pub enum OpKind {
    Plus,
    Minus,
    Star,
    Slash,
    Arrow,
}

impl Display for OpKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpKind::Plus => write!(f, "+"),
            OpKind::Minus => write!(f, "-"),
            OpKind::Star => write!(f, "*"),
            OpKind::Slash => write!(f, "/"),
            OpKind::Arrow => write!(f, "->"),
        }
    }
}

// --- parsed expression ---

pub enum ParsedExpr {
    Identifier(String),
    Op(OpKind),

    Int32Literal(i32),
    Float32Literal(f32),

    Lambda(String, Box<ParsedExpr>, Rc<ParsedExpr>),

    Tuple(Vec<ParsedExpr>),

    Collection(Vec<ParsedExpr>),
    FuncAppl(String, Vec<ParsedExpr>),
}

impl ParsedExpr {
    fn fmt_parenthesized(f: &mut fmt::Formatter<'_>, expr: &ParsedExpr) -> fmt::Result {
        let s: String = expr.to_string();

        if s.contains(' ') {
            write!(f, "({s})")
        } else {
            write!(f, "{s}")
        }
    }
}

impl Display for ParsedExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedExpr::Identifier(name) => write!(f, "{name}"),
            ParsedExpr::Op(kind) => write!(f, "{kind}"),

            ParsedExpr::Int32Literal(value) => write!(f, "{value}"),
            ParsedExpr::Float32Literal(value) => write!(f, "{value}"),

            ParsedExpr::Tuple(exprs) => {
                let contents: String = exprs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ");

                write!(f, "({contents})")
            }

            ParsedExpr::Lambda(param_name, _, body) => {
                write!(f, "{param_name} => {body}")
            }

            ParsedExpr::Collection(names) => {
                for (i, name) in names.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    Self::fmt_parenthesized(f, name)?;
                }

                Ok(())
            }

            ParsedExpr::FuncAppl(fn_name, args) => {
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
