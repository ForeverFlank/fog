use std::fmt;
use std::fmt::Display;

use crate::error::Span;

// --- statement ---

pub enum ParsedStatement {
    TypeAnnotation {
        name: String,
        expr: ParsedExpr,
        span: Span,
    },
    Declaration {
        name: String,
        expr: ParsedExpr,
        span: Span,
    },
}

// --- operator kind ---

#[derive(Clone, PartialEq, Eq, Hash)]
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

#[derive(Clone)]
pub enum ParsedExpr {
    Identifier {
        name: String,
    },
    Op {
        kind: OpKind,
    },

    Int32Literal {
        value: i32,
    },
    Float32Literal {
        value: f32,
    },

    Lambda {
        param_name: String,
        param_type: Box<ParsedExpr>,
        body: Box<ParsedExpr>,
    },

    Tuple {
        exprs: Vec<ParsedExpr>,
    },

    Collection {
        exprs: Vec<ParsedExpr>,
    },
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
            ParsedExpr::Identifier { name } => write!(f, "{name}"),
            ParsedExpr::Op { kind } => write!(f, "{kind}"),

            ParsedExpr::Int32Literal { value } => write!(f, "{value}"),
            ParsedExpr::Float32Literal { value } => write!(f, "{value}"),

            ParsedExpr::Tuple { exprs } => {
                let contents: String = exprs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ");

                write!(f, "({contents})")
            }

            ParsedExpr::Lambda {
                param_name, body, ..
            } => {
                write!(f, "{param_name} => {body}")
            }

            ParsedExpr::Collection { exprs } => {
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    Self::fmt_parenthesized(f, expr)?;
                }

                Ok(())
            }
        }
    }
}
