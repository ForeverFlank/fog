use std::fmt;
use std::fmt::Display;

use crate::error::Span;
use crate::util::{format_joined, fmt_parenthesized};

// --- statement ---

#[derive(Clone)]
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
    Expression {
        expr: ParsedExpr,
        span: Span,
    },
}

impl Display for ParsedStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedStatement::TypeAnnotation { name, expr, .. } => {
                write!(f, "{} : {}", name, expr)
            }

            ParsedStatement::Declaration { name, expr, .. } => {
                write!(f, "{} = {}", name, expr)
            }

            ParsedStatement::Expression { expr, .. } => {
                write!(f, "{}", expr)
            }
        }
    }
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
    Block {
        statements: Vec<ParsedStatement>,
    },

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
        items: Vec<ParsedExpr>,
    },

    Collection {
        items: Vec<ParsedExpr>,
    },

    Match {
        expr: Box<ParsedExpr>,
        match_arms: Vec<MatchArm>,
    },
}

#[derive(Clone)]
pub struct MatchArm {
    pub pattern: ParsedExpr,
    pub value_expr: ParsedExpr,
}

impl Display for ParsedExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedExpr::Block { statements } => {
                write!(f, "{{\n")?;

                // TODO: indent?
                for stmt in statements {
                    write!(f, "    {}\n", stmt)?;
                }

                write!(f, "}}")
            }

            ParsedExpr::Identifier { name } => write!(f, "{name}"),
            ParsedExpr::Op { kind } => write!(f, "{kind}"),

            ParsedExpr::Int32Literal { value } => write!(f, "{value}"),
            ParsedExpr::Float32Literal { value } => write!(f, "{value}"),

            ParsedExpr::Tuple { items } => write!(f, "({})", format_joined(items, ", ")),

            ParsedExpr::Lambda {
                param_name, body, ..
            } => {
                write!(f, "{param_name} => {body}")
            }

            ParsedExpr::Collection { items } => {
                for (i, expr) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    fmt_parenthesized(f, expr)?;
                }

                Ok(())
            }

            ParsedExpr::Match { expr, match_arms } => {
                write!(f, "match {expr} {{");

                for arm in match_arms {
                    write!(f, "    {} => {}", arm.pattern, arm.value_expr);
                }

                write!(f, "}}")
            }
        }
    }
}
