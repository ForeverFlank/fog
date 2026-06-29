use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use crate::error::Span;
use crate::util::{fmt_parenthesized, format_joined};

// --- statements ---

#[derive(Clone)]
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
    Expression {
        expr: ResolvedExpr,
        span: Span,
    },
}

impl Display for ResolvedStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolvedStatement::TypeAnnotation { name, expr, .. } => {
                write!(f, "{} : {}", name, expr)
            }

            ResolvedStatement::Declaration { name, expr, .. } => {
                write!(f, "{} = {}", name, expr)
            }

            ResolvedStatement::Expression { expr, .. } => {
                write!(f, "{}", expr)
            }
        }
    }
}

// --- expressions ---

#[derive(Clone)]
pub enum ResolvedExpr {
    Block {
        statements: Vec<ResolvedStatement>,
    },

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
        items: Vec<ResolvedExpr>,
    },

    FuncAppl {
        fn_name: String,
        args: Vec<ResolvedExpr>,
    },

    Match {
        expr: Box<ResolvedExpr>,
        match_arms: Vec<MatchArm>,
    },
}

#[derive(Clone)]
pub struct MatchArm {
    pub pattern: ResolvedExpr,
    pub value_expr: ResolvedExpr,
}

impl Display for ResolvedExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolvedExpr::Block { statements } => {
                write!(f, "{{\n")?;

                for stmt in statements {
                    write!(f, "    {}\n", stmt)?;
                }

                write!(f, "}}")
            }

            ResolvedExpr::Identifier { name } => write!(f, "{name}"),

            ResolvedExpr::Int32Literal { value } => write!(f, "{value}"),
            ResolvedExpr::Float32Literal { value } => write!(f, "{value}"),

            ResolvedExpr::Tuple { items } => write!(f, "({})", format_joined(items, ", ")),

            ResolvedExpr::Lambda {
                param_name, body, ..
            } => {
                write!(f, "{param_name} => {body}")
            }

            ResolvedExpr::FuncAppl { fn_name, args } => {
                write!(f, "{fn_name}")?;

                for arg in args {
                    write!(f, " ")?;
                    fmt_parenthesized(f, arg)?;
                }

                Ok(())
            }

            ResolvedExpr::Match { expr, match_arms } => {
                write!(f, "match {expr} {{\n")?;

                for arm in match_arms {
                    write!(f, "    {} => {}\n", arm.pattern, arm.value_expr)?;
                }

                write!(f, "}}")
            }
        }
    }
}
