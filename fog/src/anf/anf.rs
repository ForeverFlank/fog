use core::fmt;
use std::fmt::Display;
use std::vec;

use crate::error::Span;
use crate::parser::Literal;
use crate::parser::core_expr::CoreMatchArmPattern;
use crate::util::fmt_parenthesized;
use crate::util::format_joined;
use crate::util::indent;

#[derive(Clone)]
pub enum ANFExpr {
    Atomic(AtomicExpr),
    Declaration(ANFDeclPattern, ANFDeclExpr),
    FunctionAppl(AtomicExpr, AtomicExpr),
}

impl ANFExpr {
    pub fn all_ids(&self) -> Vec<usize> {
        match self {
            ANFExpr::Atomic(expr) => expr.all_ids(),

            ANFExpr::Declaration(_, expr) => expr.all_ids(),

            ANFExpr::FunctionAppl(callee, arg) => appl_all_ids(callee, arg),
        }
    }
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
            ANFExpr::FunctionAppl(callee, arg, ..) => fmt_appl(f, callee, arg),
        }
    }
}

#[derive(Clone)]
pub enum ANFDeclPattern {
    Single(ANFVar),
    Tuple(Vec<ANFDeclPattern>),
}

impl ANFDeclPattern {
    pub fn all_ids(&self) -> Vec<usize> {
        match self {
            ANFDeclPattern::Single(var) => match var.id {
                Some(id) => vec![id],
                None => vec![],
            },

            ANFDeclPattern::Tuple(vars) => vars.iter().flat_map(|var| var.all_ids()).collect(),
        }
    }
}

impl Display for ANFDeclPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ANFDeclPattern::Single(var) => {
                write!(f, "{var}")
            }

            ANFDeclPattern::Tuple(vars) => {
                write!(f, "({})", format_joined(vars, ", "))
            }
        }
    }
}

#[derive(Clone)]
pub enum ANFDeclExpr {
    Atomic(AtomicExpr),
    FunctionAppl(AtomicExpr, AtomicExpr),
}

impl ANFDeclExpr {
    pub fn all_ids(&self) -> Vec<usize> {
        match self {
            ANFDeclExpr::Atomic(expr) => expr.all_ids(),

            ANFDeclExpr::FunctionAppl(callee, arg) => appl_all_ids(callee, arg),
        }
    }
}

impl Display for ANFDeclExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ANFDeclExpr::Atomic(expr) => write!(f, "{expr}"),

            ANFDeclExpr::FunctionAppl(callee, arg) => fmt_appl(f, callee, arg),
        }
    }
}

fn appl_all_ids(callee: &AtomicExpr, arg: &AtomicExpr) -> Vec<usize> {
    let mut ids = callee.all_ids();
    ids.extend(arg.all_ids());
    ids
}

fn fmt_appl(f: &mut fmt::Formatter<'_>, callee: &AtomicExpr, arg: &AtomicExpr) -> fmt::Result {
    fmt_parenthesized(f, callee)?;
    write!(f, " ")?;
    fmt_parenthesized(f, arg)
}

impl From<ANFExpr> for ANFDeclExpr {
    fn from(expr: ANFExpr) -> Self {
        match expr {
            ANFExpr::Atomic(atomic) => ANFDeclExpr::Atomic(atomic),
            ANFExpr::FunctionAppl(callee, arg) => ANFDeclExpr::FunctionAppl(callee, arg),
            ANFExpr::Declaration(..) => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct ANFVar {
    pub id: Option<usize>,
    pub name: String,
}

impl Display for ANFVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = self.id {
            write!(f, "t{}", id)
        } else {
            write!(f, "{}", self.name)
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
    Var {
        var: ANFVar,
        span: Span,
    },
    Lambda {
        param: ANFVar,
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
            | AtomicExpr::Var { span, .. }
            | AtomicExpr::Lambda { span, .. }
            | AtomicExpr::Tuple { span, .. }
            | AtomicExpr::Match { span, .. } => *span,
        }
    }

    pub fn all_ids(&self) -> Vec<usize> {
        match self {
            AtomicExpr::Block { anfs, .. } => anfs.iter().flat_map(ANFExpr::all_ids).collect(),

            AtomicExpr::Literal { .. } => vec![],

            AtomicExpr::Var { var, .. } => match var.id {
                Some(id) => vec![id],
                None => vec![],
            },

            AtomicExpr::Lambda { body, .. } => body.all_ids(),

            AtomicExpr::Tuple { items, .. } => items.iter().flat_map(ANFExpr::all_ids).collect(),

            AtomicExpr::Match {
                scrutinee, arms, ..
            } => {
                let mut ids = scrutinee.all_ids();
                ids.extend(arms.iter().flat_map(|(_, arm)| arm.all_ids()));
                ids
            }
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

            AtomicExpr::Var { var, .. } => {
                write!(f, "{var}")
            }

            AtomicExpr::Lambda {
                param: param_name,
                body,
                ..
            } => {
                write!(f, "{param_name} => {}", *body)
            }

            AtomicExpr::Tuple { items, .. } => {
                write!(f, "({})", format_joined(items, ", "))
            }

            AtomicExpr::Match {
                scrutinee, arms, ..
            } => {
                write!(f, "match {} {{\n", *scrutinee)?;
                for arm in arms {
                    write!(f, "{}\n", indent(&format!("{} => {}", arm.0, arm.1)))?;
                }
                write!(f, "}}")
            }
        }
    }
}
