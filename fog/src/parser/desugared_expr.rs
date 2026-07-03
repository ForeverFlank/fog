use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use crate::error::Span;
use crate::parser::Literal;
use crate::util::{fmt_parenthesized, format_joined};

// --- statements ---

#[derive(Clone)]
pub enum DesugaredStatement {
    TypeAnnotation {
        name: String,
        expr: DesugaredExpr,
        span: Span,
    },
    Declaration {
        pattern: DesugaredDeclPattern,
        expr: DesugaredExpr,
        span: Span,
    },
    Expression {
        expr: DesugaredExpr,
        span: Span,
    },
}

impl Display for DesugaredStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DesugaredStatement::TypeAnnotation { name, expr, .. } => {
                write!(f, "{} : {}", name, expr)
            }

            DesugaredStatement::Declaration { pattern, expr, .. } => {
                write!(f, "{} = {}", pattern, expr)
            }

            DesugaredStatement::Expression { expr, .. } => {
                write!(f, "{}", expr)
            }
        }
    }
}

// --- patterns ---

// -- declaration statement

#[derive(Clone)]
pub enum DesugaredDeclPattern {
    Identifier {
        name: String,
        span: Span,
    },
    Tuple {
        items: Vec<DesugaredTupleDeclPattern>,
        span: Span,
    },
}

impl Display for DesugaredDeclPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DesugaredDeclPattern::Identifier { name, .. } => {
                write!(f, "{name}")
            }
            DesugaredDeclPattern::Tuple { items, .. } => {
                write!(f, "{}", format_joined(items, ", "))
            }
        }
    }
}

// -- tuple declaration

#[derive(Clone)]
pub enum DesugaredTupleDeclPattern {
    Identifier {
        name: String,
        span: Span,
    },
    Literal {
        literal: Literal,
        span: Span,
    },
    Tuple {
        items: Vec<DesugaredTupleDeclPattern>,
        span: Span,
    },
}

impl Display for DesugaredTupleDeclPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DesugaredTupleDeclPattern::Identifier { name, .. } => {
                write!(f, "{name}")
            }
            DesugaredTupleDeclPattern::Literal { literal, .. } => {
                write!(f, "{literal}")
            }
            DesugaredTupleDeclPattern::Tuple { items, .. } => {
                write!(f, "{}", format_joined(items, ", "))
            }
        }
    }
}

// --- expressions ---

#[derive(Clone)]
pub enum DesugaredExpr {
    Block {
        statements: Vec<DesugaredStatement>,
        span: Span,
    },

    Identifier {
        name: String,
        span: Span,
    },

    Int32Literal {
        value: i32,
        span: Span,
    },
    Float32Literal {
        value: f32,
        span: Span,
    },

    Lambda {
        param_name: String,
        param_type: Box<DesugaredExpr>,
        body: Rc<DesugaredExpr>,
        span: Span,
    },

    Tuple {
        items: Vec<DesugaredExpr>,
        span: Span,
    },

    FuncAppl {
        fn_name: String,
        args: Vec<DesugaredExpr>,
        span: Span,
    },

    Match {
        expr: Box<DesugaredExpr>,
        match_arms: Vec<DesugaredMatchArm>,
        span: Span,
    },
}

impl DesugaredExpr {
    pub fn span(&self) -> Span {
        match self {
            DesugaredExpr::Block { span, .. }
            | DesugaredExpr::Identifier { span, .. }
            | DesugaredExpr::Int32Literal { span, .. }
            | DesugaredExpr::Float32Literal { span, .. }
            | DesugaredExpr::Lambda { span, .. }
            | DesugaredExpr::Tuple { span, .. }
            | DesugaredExpr::FuncAppl { span, .. }
            | DesugaredExpr::Match { span, .. } => span,
        }
    }
}

#[derive(Clone)]
pub struct DesugaredMatchArm {
    pub pattern: DesugaredExpr,
    pub value_expr: DesugaredExpr,
}

impl Display for DesugaredExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DesugaredExpr::Block { statements, .. } => {
                write!(f, "{{\n")?;
                for stmt in statements {
                    write!(f, "    {}\n", stmt)?;
                }
                write!(f, "}}")
            }

            DesugaredExpr::Identifier { name, .. } => write!(f, "{name}"),

            DesugaredExpr::Int32Literal { value, .. } => write!(f, "{value}"),
            DesugaredExpr::Float32Literal { value, .. } => write!(f, "{value}"),

            DesugaredExpr::Tuple { items, .. } => write!(f, "({})", format_joined(items, ", ")),

            DesugaredExpr::Lambda {
                param_name, body, ..
            } => {
                write!(f, "{param_name} => {body}")
            }

            DesugaredExpr::FuncAppl { fn_name, args, .. } => {
                write!(f, "{fn_name}")?;
                for arg in args {
                    write!(f, " ")?;
                    fmt_parenthesized(f, arg)?;
                }
                Ok(())
            }

            DesugaredExpr::Match {
                expr, match_arms, ..
            } => {
                write!(f, "match {expr} {{\n")?;
                for arm in match_arms {
                    write!(f, "    {} => {}\n", arm.pattern, arm.value_expr)?;
                }
                write!(f, "}}")
            }
        }
    }
}
