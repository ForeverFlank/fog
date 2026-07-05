use std::fmt::Display;

use crate::parser::Literal;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreMatchArmPattern;
use crate::util::fmt_parenthesized;
use crate::util::format_joined;

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
            ANFExpr::Declaration(pattern, expr) => {
                write!(f, "{pattern} = {}", *expr)
            }
            ANFExpr::FunctionAppl(callee, arg) => {
                fmt_parenthesized(f, callee)?;
                write!(f, " ")?;
                fmt_parenthesized(f, arg)
            }
        }
    }
}

#[derive(Clone)]
pub enum AtomicExpr {
    Literal {
        literal: Literal,
    },
    Identifier {
        name: String,
    },
    Lambda {
        param_name: String,
        body: Box<ANFExpr>,
    },
    Tuple {
        items: Vec<ANFExpr>,
    },
    Match {
        scrutinee: Box<AtomicExpr>,
        arms: Vec<(CoreMatchArmPattern, ANFExpr)>,
    },
}

impl Display for AtomicExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomicExpr::Literal { literal } => {
                write!(f, "{literal}")
            }

            AtomicExpr::Identifier { name } => {
                write!(f, "{name}")
            }

            AtomicExpr::Lambda { param_name, body } => {
                write!(f, "{param_name} => {}", *body)
            }

            AtomicExpr::Tuple { items } => {
                write!(f, "({})", format_joined(items, ", "))
            }

            AtomicExpr::Match { scrutinee, arms } => {
                write!(f, "{} => {{\n", *scrutinee)?;
                for arm in arms {
                    write!(f, "    {} => {}\n", arm.0, arm.1)?;
                }
                write!(f, "}}")
            }
        }
    }
}
