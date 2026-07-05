use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use crate::error::Span;
use crate::parser::Literal;
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
        pattern: ResolvedDeclPattern,
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

            ResolvedStatement::Declaration { pattern, expr, .. } => {
                write!(f, "{} = {}", pattern, expr)
            }

            ResolvedStatement::Expression { expr, .. } => {
                write!(f, "{}", expr)
            }
        }
    }
}

// --- patterns ---

// -- declaration statement

#[derive(Clone)]
pub enum ResolvedDeclPattern {
    Identifier {
        name: String,
        span: Span,
    },
    Tuple {
        items: Vec<ResolvedTupleDeclPattern>,
        span: Span,
    },
    FunctionClause {
        name: String,
        items: Vec<ResolvedMatchArmPattern>,
        span: Span,
    },
}

impl Display for ResolvedDeclPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolvedDeclPattern::Identifier { name, .. } => {
                write!(f, "{name}")
            }

            ResolvedDeclPattern::Tuple { items, .. } => {
                write!(f, "{}", format_joined(items, ", "))
            }

            ResolvedDeclPattern::FunctionClause { items, .. } => {
                for (i, expr) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    fmt_parenthesized(f, expr)?;
                }

                Ok(())
            }
        }
    }
}

// -- tuple declaration patterns

#[derive(Clone)]
pub enum ResolvedTupleDeclPattern {
    Identifier {
        name: String,
        span: Span,
    },
    Tuple {
        items: Vec<ResolvedTupleDeclPattern>,
        span: Span,
    },
}

impl Display for ResolvedTupleDeclPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolvedTupleDeclPattern::Identifier { name, .. } => {
                write!(f, "{name}")
            }

            ResolvedTupleDeclPattern::Tuple { items, .. } => {
                write!(f, "{}", format_joined(items, ", "))
            }
        }
    }
}

// -- match arm patterns

#[derive(Clone)]
pub enum ResolvedMatchArmPattern {
    Literal {
        literal: Literal,
        span: Span,
    },
    Tuple {
        items: Vec<ResolvedMatchArmPattern>,
        span: Span,
    },
    Identifier {
        name: String,
        span: Span,
    },
    DataConstructor {
        name: String,
        args: Vec<ResolvedMatchArmPattern>,
        span: Span,
    },
}

impl Display for ResolvedMatchArmPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolvedMatchArmPattern::Literal { literal, .. } => {
                write!(f, "{literal}")
            }

            ResolvedMatchArmPattern::Tuple { items, .. } => {
                write!(f, "({})", format_joined(items, ", "))
            }

            ResolvedMatchArmPattern::Identifier { name, .. } => {
                write!(f, "{name}")
            }

            ResolvedMatchArmPattern::DataConstructor { name, args, .. } => {
                write!(f, "{name}")?;
                for item in args {
                    write!(f, " ")?;
                    fmt_parenthesized(f, item)?;
                }
                Ok(())
            }
        }
    }
}

// --- expressions ---

#[derive(Clone)]
pub enum ResolvedExpr {
    Block {
        statements: Vec<ResolvedStatement>,
        span: Span,
    },

    Identifier {
        name: String,
        span: Span,
    },

    Literal {
        literal: Literal,
        span: Span,
    },

    Lambda {
        param_name: String,
        param_type: Box<ResolvedExpr>,
        body: Rc<ResolvedExpr>,
        span: Span,
    },

    Tuple {
        items: Vec<ResolvedExpr>,
        span: Span,
    },

    FunctionAppl {
        callee: Box<ResolvedExpr>,
        arg: Box<ResolvedExpr>,
        span: Span,
    },

    Match {
        scrutinee: Box<ResolvedExpr>,
        match_arms: Vec<ResolvedMatchArm>,
        span: Span,
    },
}

impl ResolvedExpr {
    pub fn span(&self) -> Span {
        match self {
            ResolvedExpr::Block { span, .. }
            | ResolvedExpr::Identifier { span, .. }
            | ResolvedExpr::Literal { span, .. }
            | ResolvedExpr::Lambda { span, .. }
            | ResolvedExpr::Tuple { span, .. }
            | ResolvedExpr::FunctionAppl { span, .. }
            | ResolvedExpr::Match { span, .. } => *span,
        }
    }
}

impl Display for ResolvedExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolvedExpr::Block { statements, .. } => {
                write!(f, "{{\n")?;
                for stmt in statements {
                    write!(f, "    {}\n", stmt)?;
                }
                write!(f, "}}")
            }

            ResolvedExpr::Identifier { name, .. } => write!(f, "{name}"),

            ResolvedExpr::Literal { literal, .. } => write!(f, "{literal}"),

            ResolvedExpr::Tuple { items, .. } => write!(f, "({})", format_joined(items, ", ")),

            ResolvedExpr::Lambda {
                param_name, body, ..
            } => {
                write!(f, "{param_name} => {body}")
            }

            ResolvedExpr::FunctionAppl { callee, arg, .. } => {
                fmt_parenthesized(f, callee.as_ref())?;
                write!(f, " ")?;
                fmt_parenthesized(f, arg.as_ref())
            }

            ResolvedExpr::Match {
                scrutinee,
                match_arms,
                ..
            } => {
                write!(f, "match {scrutinee} {{\n")?;
                for arm in match_arms {
                    write!(f, "    {} => {}\n", arm.pattern, arm.value_expr)?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[derive(Clone)]
pub struct ResolvedMatchArm {
    pub pattern: ResolvedMatchArmPattern,
    pub value_expr: ResolvedExpr,
}

impl Display for ResolvedMatchArm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} => {}\n", self.pattern, self.value_expr)
    }
}
