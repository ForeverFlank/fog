use std::fmt;
use std::fmt::Display;

use crate::error::FogResult;
use crate::error::Span;
use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;
use crate::parse_error;
use crate::parser::Literal;
use crate::parser::core_expr::CoreAtomicTypeExpr;
use crate::parser::core_expr::CoreKindExpr;
use crate::parser::core_expr::CoreTypeExpr;
use crate::util::fmt_parenthesized;
use crate::util::format_joined;
use crate::util::indent;

// --- statement ---

#[derive(Clone)]
pub enum ParsedStatement {
    KindAnnotation {
        name: String,
        expr: CoreKindExpr,
        span: Span,
    },
    TypeDeclaration {
        name: String,
        expr: CoreTypeExpr,
        span: Span,
    },

    TypeAnnotation {
        pattern: ParsedDeclPattern,
        expr: CoreAtomicTypeExpr,
        span: Span,
    },
    VarDeclaration {
        pattern: ParsedDeclPattern,
        expr: ParsedValueExpr,
        // span: Span,
    },

    Expression {
        expr: ParsedValueExpr,
        span: Span,
    },
}

impl Display for ParsedStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedStatement::KindAnnotation { name, expr, .. } => {
                write!(f, "{} : {}", name, expr)
            }

            ParsedStatement::TypeDeclaration { name, expr, .. } => {
                write!(f, "{} = {}", name, expr)
            }

            ParsedStatement::TypeAnnotation { pattern, expr, .. } => {
                write!(f, "{} : {}", pattern, expr)
            }

            ParsedStatement::VarDeclaration { pattern, expr, .. } => {
                write!(f, "{} = {}", pattern, expr)
            }

            ParsedStatement::Expression { expr, .. } => {
                write!(f, "{}", expr)
            }
        }
    }
}

// --- pattern ---

// -- declaration pattern

#[derive(Clone)]
pub enum ParsedDeclPattern {
    Identifier {
        name: String,
        span: Span,
    },
    Literal {
        literal: Literal,
        span: Span,
    },
    Tuple {
        items: Vec<ParsedDeclPattern>,
        span: Span,
    },
    Collection {
        items: Vec<ParsedDeclPattern>,
        span: Span,
    },
}

impl Display for ParsedDeclPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedDeclPattern::Identifier { name, .. } => {
                write!(f, "{name}")
            }

            ParsedDeclPattern::Literal { literal, .. } => {
                write!(f, "{literal}")
            }

            ParsedDeclPattern::Tuple { items, .. } => {
                write!(f, "{}", format_joined(items, ", "))
            }

            ParsedDeclPattern::Collection { items, .. } => {
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

impl OpKind {
    pub fn from_token(token: &Token) -> Option<OpKind> {
        match &token.kind {
            TokenKind::Plus => Some(OpKind::Plus),
            TokenKind::Minus => Some(OpKind::Minus),
            TokenKind::Star => Some(OpKind::Star),
            TokenKind::Slash => Some(OpKind::Slash),
            TokenKind::Arrow => Some(OpKind::Arrow),
            _ => None,
        }
    }
}

// --- expressions ---

#[derive(Clone)]
pub enum ParsedValueExpr {
    Block {
        statements: Vec<ParsedStatement>,
        span: Span,
    },
    Identifier {
        name: String,
        span: Span,
    },
    Op {
        kind: OpKind,
        span: Span,
    },
    Literal {
        literal: Literal,
        span: Span,
    },
    Lambda {
        param_name: String,
        param_type: CoreAtomicTypeExpr,
        body: Box<ParsedValueExpr>,
        span: Span,
    },
    Tuple {
        items: Vec<ParsedValueExpr>,
        span: Span,
    },
    Collection {
        items: Vec<ParsedValueExpr>,
        span: Span,
    },
    Match {
        scrutinee: Box<ParsedValueExpr>,
        match_arms: Vec<ParsedMatchArm>,
        span: Span,
    },
}

impl ParsedValueExpr {
    pub fn span(&self) -> Span {
        match self {
            ParsedValueExpr::Block { span, .. }
            | ParsedValueExpr::Identifier { span, .. }
            | ParsedValueExpr::Op { span, .. }
            | ParsedValueExpr::Literal { span, .. }
            | ParsedValueExpr::Lambda { span, .. }
            | ParsedValueExpr::Tuple { span, .. }
            | ParsedValueExpr::Collection { span, .. }
            | ParsedValueExpr::Match { span, .. } => *span,
        }
    }

    pub fn is_primary_starter(&self) -> bool {
        match self {
            ParsedValueExpr::Identifier { .. }
            | ParsedValueExpr::Literal { .. }
            | ParsedValueExpr::Tuple { .. }
            | ParsedValueExpr::Collection { .. } => true,

            ParsedValueExpr::Op { kind, .. } => matches!(kind, OpKind::Minus),

            _ => false,
        }
    }

    pub fn into_decl_pattern(self) -> FogResult<ParsedDeclPattern> {
        match self {
            ParsedValueExpr::Identifier { name, span } => {
                Ok(ParsedDeclPattern::Identifier { name, span })
            }

            ParsedValueExpr::Literal { literal, span } => {
                Ok(ParsedDeclPattern::Literal { literal, span })
            }

            ParsedValueExpr::Tuple { items, span } => Ok(ParsedDeclPattern::Tuple {
                items: items
                    .into_iter()
                    .map(ParsedValueExpr::into_decl_pattern)
                    .collect::<Result<Vec<_>, _>>()?,
                span,
            }),

            ParsedValueExpr::Collection { items, span } => Ok(ParsedDeclPattern::Collection {
                items: items
                    .into_iter()
                    .map(ParsedValueExpr::into_decl_pattern)
                    .collect::<Result<Vec<_>, _>>()?,
                span,
            }),

            ParsedValueExpr::Block { .. }
            | ParsedValueExpr::Op { .. }
            | ParsedValueExpr::Lambda { .. }
            | ParsedValueExpr::Match { .. } => Err(parse_error!(
                Some(self.span()),
                "invalid declaration pattern"
            )),
        }
    }
}

impl Display for ParsedValueExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedValueExpr::Block { statements, .. } => {
                write!(f, "{{\n")?;
                for stmt in statements {
                    write!(f, "{}\n", indent(&stmt.to_string()))?;
                }
                write!(f, "}}")
            }

            ParsedValueExpr::Identifier { name, .. } => write!(f, "{name}"),
            ParsedValueExpr::Op { kind, .. } => write!(f, "{kind}"),

            ParsedValueExpr::Literal { literal, .. } => write!(f, "{literal}"),

            ParsedValueExpr::Tuple { items, .. } => write!(f, "({})", format_joined(items, ", ")),

            ParsedValueExpr::Lambda {
                param_name, body, ..
            } => {
                write!(f, "{param_name} => {body}")
            }

            ParsedValueExpr::Collection { items: args, .. } => {
                for (i, expr) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    fmt_parenthesized(f, expr)?;
                }
                Ok(())
            }

            ParsedValueExpr::Match {
                scrutinee,
                match_arms,
                ..
            } => {
                write!(f, "match {scrutinee} {{\n")?;
                for arm in match_arms {
                    write!(
                        f,
                        "{}\n",
                        indent(&format!("{} => {}", arm.pattern, arm.value_expr))
                    )?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[derive(Clone)]
pub struct ParsedMatchArm {
    pub pattern: ParsedValueExpr,
    pub value_expr: ParsedValueExpr,
}
