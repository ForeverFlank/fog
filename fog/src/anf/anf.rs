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
pub enum ANFStatement {
    Declaration(ANFDeclPattern, ANFValue),
    Value(ANFValue),
}

impl ANFStatement {
    pub fn all_ids(&self) -> Vec<usize> {
        match self {
            ANFStatement::Declaration(_, value) => value.all_ids(),
            ANFStatement::Value(value) => value.all_ids(),
        }
    }
}

impl Display for ANFStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ANFStatement::Declaration(pattern, value) => {
                write!(f, "{pattern} = {value}")
            }

            ANFStatement::Value(value) => {
                write!(f, "{value}")
            }
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
pub enum ANFValue {
    Atomic(ANFAtomic),
    FunctionAppl(ANFAtomic, ANFAtomic),
}

impl ANFValue {
    pub fn all_ids(&self) -> Vec<usize> {
        match self {
            ANFValue::Atomic(expr) => expr.all_ids(),

            ANFValue::FunctionAppl(callee, arg) => appl_all_ids(callee, arg),
        }
    }
}

impl Display for ANFValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ANFValue::Atomic(expr) => write!(f, "{expr}"),

            ANFValue::FunctionAppl(callee, arg) => {
                fmt_parenthesized(f, callee)?;
                write!(f, " ")?;
                fmt_parenthesized(f, arg)
            }
        }
    }
}

fn appl_all_ids(callee: &ANFAtomic, arg: &ANFAtomic) -> Vec<usize> {
    let mut ids = callee.all_ids();
    ids.extend(arg.all_ids());
    ids
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
pub enum ANFAtomic {
    Block {
        anfs: Vec<ANFStatement>,
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
        body: Box<ANFValue>,
        span: Span,
    },
    Tuple {
        items: Vec<ANFValue>,
        span: Span,
    },
    Match {
        scrutinee: Box<ANFAtomic>,
        arms: Vec<(CoreMatchArmPattern, ANFValue)>,
        span: Span,
    },
}

impl ANFAtomic {
    pub fn span(&self) -> Span {
        match self {
            ANFAtomic::Block { span, .. }
            | ANFAtomic::Literal { span, .. }
            | ANFAtomic::Var { span, .. }
            | ANFAtomic::Lambda { span, .. }
            | ANFAtomic::Tuple { span, .. }
            | ANFAtomic::Match { span, .. } => *span,
        }
    }

    pub fn all_ids(&self) -> Vec<usize> {
        match self {
            ANFAtomic::Block { anfs, .. } => anfs.iter().flat_map(ANFStatement::all_ids).collect(),

            ANFAtomic::Literal { .. } => vec![],

            ANFAtomic::Var { var, .. } => match var.id {
                Some(id) => vec![id],
                None => vec![],
            },

            ANFAtomic::Lambda { body, .. } => body.all_ids(),

            ANFAtomic::Tuple { items, .. } => items.iter().flat_map(ANFValue::all_ids).collect(),

            ANFAtomic::Match {
                scrutinee, arms, ..
            } => {
                let mut ids = scrutinee.all_ids();
                ids.extend(arms.iter().flat_map(|(_, arm)| arm.all_ids()));
                ids
            }
        }
    }
}

impl Display for ANFAtomic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ANFAtomic::Block { anfs, .. } => {
                write!(f, "{{\n")?;

                for anf in anfs {
                    write!(f, "{}\n", indent(&anf.to_string()))?;
                }

                write!(f, "}}")
            }

            ANFAtomic::Literal { literal, .. } => {
                write!(f, "{literal}")
            }

            ANFAtomic::Var { var, .. } => {
                write!(f, "{var}")
            }

            ANFAtomic::Lambda {
                param: param_name,
                body,
                ..
            } => {
                write!(f, "{param_name} => {}", *body)
            }

            ANFAtomic::Tuple { items, .. } => {
                write!(f, "({})", format_joined(items, ", "))
            }

            ANFAtomic::Match {
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
