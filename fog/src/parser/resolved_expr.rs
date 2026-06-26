use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use crate::error::Span;

pub enum ResolvedStatement {
    TypeAnnotation {
        name: String,
        expr: ResolvedExpr,
        span: Span,
    },
    Declaration {
        name: String,
        expr: ResolvedExpr,
        span: Span,
    },
}

pub enum ResolvedExpr {
    Identifier {
        name: String,
    },

    Int32Literal {
        value: i32,
    },
    Float32Literal {
        value: f32,
    },

    Lambda {
        param_name: String,
        param_type: Box<ResolvedExpr>,
        body: Rc<ResolvedExpr>,
    },

    Tuple {
        exprs: Vec<ResolvedExpr>,
    },

    FuncAppl {
        fn_name: String,
        args: Vec<ResolvedExpr>,
    },
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
            ResolvedExpr::Identifier { name } => write!(f, "{name}"),

            ResolvedExpr::Int32Literal { value } => write!(f, "{value}"),
            ResolvedExpr::Float32Literal { value } => write!(f, "{value}"),

            ResolvedExpr::Tuple { exprs } => {
                let contents: String = exprs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ");

                write!(f, "({contents})")
            }

            ResolvedExpr::Lambda {
                param_name, body, ..
            } => {
                write!(f, "{param_name} => {body}")
            }

            ResolvedExpr::FuncAppl { fn_name, args } => {
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
