use std::fmt::Display;

use crate::error::Span;
use crate::parser::Literal;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreMatchArmPattern;
use crate::util::fmt_parenthesized;
use crate::util::format_joined;
use crate::util::indent;

#[derive(Clone)]
pub enum ANFExpr {
    Atomic(AtomicExpr),
    Declaration(CoreDeclPattern, Box<ANFExpr>),
    FunctionAppl(AtomicExpr, AtomicExpr),
}

impl Display for ANFExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ANFExpr::Atomic(expr) => {
                write!(f, "{expr}")
            }
            ANFExpr::Declaration(pattern, expr, ..) => {
                write!(f, "{pattern} = {}", *expr)
            }
            ANFExpr::FunctionAppl(callee, arg, ..) => {
                fmt_parenthesized(f, callee)?;
                write!(f, " ")?;
                fmt_parenthesized(f, arg)
            }
        }
    }
}

#[derive(Clone)]
pub enum AtomicExpr {
    Block {
        anfs: Vec<ANFExpr>,
        span: Span,
    },
    Literal {
        literal: Literal,
        span: Span,
    },
    Identifier {
        name: String,
        span: Span,
    },
    Lambda {
        param_name: String,
        body: Box<ANFExpr>,
        span: Span,
    },
    Tuple {
        items: Vec<ANFExpr>,
        span: Span,
    },
    Match {
        scrutinee: Box<AtomicExpr>,
        arms: Vec<(CoreMatchArmPattern, ANFExpr)>,
        span: Span,
    },
}

impl AtomicExpr {
    pub fn span(&self) -> Span {
        match self {
            AtomicExpr::Block { span, .. }
            | AtomicExpr::Literal { span, .. }
            | AtomicExpr::Identifier { span, .. }
            | AtomicExpr::Lambda { span, .. }
            | AtomicExpr::Tuple { span, .. }
            | AtomicExpr::Match { span, .. } => *span,
        }
    }
}

impl Display for AtomicExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomicExpr::Block { anfs, .. } => {
                write!(f, "{{\n")?;

                for anf in anfs {
                    write!(f, "{}\n", indent(&anf.to_string()))?;
                }

                write!(f, "}}")
            }

            AtomicExpr::Literal { literal, .. } => {
                write!(f, "{literal}")
            }

            AtomicExpr::Identifier { name, .. } => {
                write!(f, "{name}")
            }

            AtomicExpr::Lambda {
                param_name, body, ..
            } => {
                write!(f, "{param_name} => {}", *body)
            }

            AtomicExpr::Tuple { items, .. } => {
                write!(f, "({})", format_joined(items, ", "))
            }

            AtomicExpr::Match {
                scrutinee, arms, ..
            } => {
                write!(f, "{} => {{\n", *scrutinee)?;
                for arm in arms {
                    write!(f, "{}\n", indent(&format!("{} => {}", arm.0, arm.1)))?;
                }
                write!(f, "}}")
            }
        }
    }
}
