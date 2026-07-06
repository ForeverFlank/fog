use std::fmt;
use std::fmt::Display;

use crate::error::Span;
use crate::parser::Literal;
use crate::util::fmt_parenthesized;
use crate::util::format_joined;

// --- statements ---

#[derive(Clone)]
pub enum CoreStatement {
    KindAnnotation {
        name: String,
        expr: CoreKindExpr,
        span: Span,
    },
    TypeDeclaration {
        name: String,
        expr: CoreAtomicTypeExpr,
        span: Span,
    },
    TypeAnnotation {
        name: String,
        expr: CoreTypeExpr,
        span: Span,
    },
    VarDeclaration {
        pattern: CoreDeclPattern,
        expr: CoreExpr,
        span: Span,
    },
    Expression {
        expr: CoreExpr,
        span: Span,
    },
}

impl Display for CoreStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreStatement::KindAnnotation { name, expr, .. } => {
                write!(f, "{} : {}", name, expr)
            }

            CoreStatement::TypeDeclaration { name, expr, .. } => {
                write!(f, "{} = {}", name, expr)
            }

            CoreStatement::TypeAnnotation { name, expr, .. } => {
                write!(f, "{} : {}", name, expr)
            }

            CoreStatement::VarDeclaration { pattern, expr, .. } => {
                write!(f, "{} = {}", pattern, expr)
            }

            CoreStatement::Expression { expr, .. } => {
                write!(f, "{}", expr)
            }
        }
    }
}

// --- patterns ---

// -- declaration statement

#[derive(Clone)]
pub enum CoreDeclPattern {
    Identifier {
        name: String,
        span: Span,
    },
    Tuple {
        items: Vec<CoreTupleDeclPattern>,
        span: Span,
    },
}

impl CoreDeclPattern {
    pub fn all_identifiers(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        match self {
            CoreDeclPattern::Identifier { name, .. } => Box::new(std::iter::once(name.as_str())),

            CoreDeclPattern::Tuple { items, .. } => {
                Box::new(items.iter().flat_map(|item| item.all_identifiers()))
            }
        }
    }
}

impl Display for CoreDeclPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreDeclPattern::Identifier { name, .. } => {
                write!(f, "{name}")
            }

            CoreDeclPattern::Tuple { items, .. } => {
                write!(f, "{}", format_joined(items, ", "))
            }
        }
    }
}

// -- tuple declaration

#[derive(Clone)]
pub enum CoreTupleDeclPattern {
    Identifier {
        name: String,
        span: Span,
    },
    Tuple {
        items: Vec<CoreTupleDeclPattern>,
        span: Span,
    },
}

impl CoreTupleDeclPattern {
    pub fn all_identifiers(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        match self {
            CoreTupleDeclPattern::Identifier { name, .. } => {
                Box::new(std::iter::once(name.as_str()))
            }

            CoreTupleDeclPattern::Tuple { items, .. } => {
                Box::new(items.iter().flat_map(|item| item.all_identifiers()))
            }
        }
    }
}

impl Display for CoreTupleDeclPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreTupleDeclPattern::Identifier { name, .. } => {
                write!(f, "{name}")
            }

            CoreTupleDeclPattern::Tuple { items, .. } => {
                write!(f, "{}", format_joined(items, ", "))
            }
        }
    }
}

// -- match arm pattern

#[derive(Clone)]
pub enum CoreMatchArmPattern {
    Literal {
        literal: Literal,
        span: Span,
    },
    Tuple {
        items: Vec<CoreMatchArmPattern>,
        span: Span,
    },
    Identifier {
        name: String,
        span: Span,
    },
    DataConstructor {
        name: String,
        args: Vec<CoreMatchArmPattern>,
        span: Span,
    },
}

impl CoreMatchArmPattern {
    pub fn span(&self) -> Span {
        match *self {
            CoreMatchArmPattern::Literal { span, .. }
            | CoreMatchArmPattern::Tuple { span, .. }
            | CoreMatchArmPattern::Identifier { span, .. }
            | CoreMatchArmPattern::DataConstructor { span, .. } => span,
        }
    }

    pub fn all_identifiers(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        match self {
            CoreMatchArmPattern::Literal { .. } => Box::new(std::iter::empty()),

            CoreMatchArmPattern::Tuple { items, .. } => {
                Box::new(items.iter().flat_map(|item| item.all_identifiers()))
            }

            CoreMatchArmPattern::Identifier { name, .. } => {
                Box::new(std::iter::once(name.as_str()))
            }

            CoreMatchArmPattern::DataConstructor { args, .. } => {
                Box::new(args.iter().flat_map(|item| item.all_identifiers()))
            }
        }
    }
}

impl Display for CoreMatchArmPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreMatchArmPattern::Literal { literal, .. } => {
                write!(f, "{literal}")
            }

            CoreMatchArmPattern::Tuple { items, .. } => {
                write!(f, "({})", format_joined(items, ", "))
            }

            CoreMatchArmPattern::Identifier { name, .. } => {
                write!(f, "{name}")
            }

            CoreMatchArmPattern::DataConstructor { name, args, .. } => {
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
pub enum CoreKindExpr {
    Type {
        span: Span,
    },
    Constraint {
        span: Span,
    },
    Function {
        param_kind: Box<CoreKindExpr>,
        return_kind: Box<CoreKindExpr>,
        span: Span,
    },
}

impl CoreKindExpr {
    pub fn span(&self) -> Span {
        match self {
            CoreKindExpr::Type { span }
            | CoreKindExpr::Constraint { span }
            | CoreKindExpr::Function { span, .. } => *span,
        }
    }
}

impl Display for CoreKindExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreKindExpr::Type { .. } => {
                write!(f, "Type")
            }

            CoreKindExpr::Constraint { .. } => {
                write!(f, "Constraint")
            }

            CoreKindExpr::Function {
                param_kind,
                return_kind,
                ..
            } => {
                fmt_parenthesized(f, param_kind)?;
                write!(f, " -> ");
                fmt_parenthesized(f, return_kind)
            }
        }
    }
}

#[derive(Clone)]
pub enum CoreTypeExpr {
    Identifier {
        name: String,
        span: Span,
    },
    FunctionAppl {
        callee: Box<CoreAtomicTypeExpr>,
        arg: Box<CoreAtomicTypeExpr>,
        span: Span,
    },
    Product {
        types: Vec<CoreAtomicTypeExpr>,
        span: Span,
    },
    Sum {
        ctors: Vec<CoreDataConstructor>,
        span: Span,
    },
}

impl Display for CoreTypeExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

#[derive(Clone)]
pub enum CoreAtomicTypeExpr {
    Identifier {
        name: String,
        span: Span,
    },
    Product {
        types: Vec<CoreAtomicTypeExpr>,
        span: Span,
    },
    FunctionAppl {
        callee: Box<CoreAtomicTypeExpr>,
        arg: Box<CoreAtomicTypeExpr>,
        span: Span,
    },
}

impl Display for CoreAtomicTypeExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreAtomicTypeExpr::Identifier { name, .. } => {
                write!(f, "{name}")
            }

            CoreAtomicTypeExpr::Product { types, .. } => {
                write!(f, "{}", format_joined(types, " * "))
            }

            CoreAtomicTypeExpr::FunctionAppl { callee, arg, .. } => {
                fmt_parenthesized(f, callee.as_ref())?;
                write!(f, " ")?;
                fmt_parenthesized(f, arg.as_ref())
            }
        }
    }
}

#[derive(Clone)]
pub struct CoreDataConstructor {
    tag: String,
    types: Vec<CoreAtomicTypeExpr>,
}

impl Display for CoreDataConstructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

#[derive(Clone)]
pub enum CoreExpr {
    Block {
        statements: Vec<CoreStatement>,
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
        param_type: Box<CoreExpr>,
        body: Box<CoreExpr>,
        span: Span,
    },
    Tuple {
        items: Vec<CoreExpr>,
        span: Span,
    },
    FunctionAppl {
        callee: Box<CoreExpr>,
        arg: Box<CoreExpr>,
        span: Span,
    },
    Match {
        scrutinee: Box<CoreExpr>,
        arms: Vec<CoreMatchArm>,
        span: Span,
    },
}

impl CoreExpr {
    pub fn span(&self) -> Span {
        match self {
            CoreExpr::Block { span, .. }
            | CoreExpr::Identifier { span, .. }
            | CoreExpr::Literal { span, .. }
            | CoreExpr::Lambda { span, .. }
            | CoreExpr::Tuple { span, .. }
            | CoreExpr::FunctionAppl { span, .. }
            | CoreExpr::Match { span, .. } => *span,
        }
    }

    pub fn uncurry(&self) -> (&CoreExpr, Vec<&CoreExpr>) {
        let mut args = Vec::new();
        let mut head = self;

        while let CoreExpr::FunctionAppl { callee, arg, .. } = head {
            args.push(arg.as_ref());
            head = callee.as_ref();
        }

        args.reverse();

        (head, args)
    }
}

#[derive(Clone)]
pub struct CoreMatchArm {
    pub pattern: CoreMatchArmPattern,
    pub value_expr: CoreExpr,
}

impl Display for CoreExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreExpr::Block { statements, .. } => {
                write!(f, "{{\n")?;
                for stmt in statements {
                    write!(f, "    {}\n", stmt)?;
                }
                write!(f, "}}")
            }

            CoreExpr::Identifier { name, .. } => {
                write!(f, "{name}")
            }

            CoreExpr::Literal { literal, .. } => {
                write!(f, "{literal}")
            }

            CoreExpr::Tuple { items, .. } => {
                write!(f, "({})", format_joined(items, ", "))
            }

            CoreExpr::Lambda {
                param_name, body, ..
            } => {
                write!(f, "{param_name} => {body}")
            }

            CoreExpr::FunctionAppl { callee, arg, .. } => {
                fmt_parenthesized(f, callee.as_ref())?;
                write!(f, " ")?;
                fmt_parenthesized(f, arg.as_ref())
            }

            CoreExpr::Match {
                scrutinee, arms, ..
            } => {
                write!(f, "match {scrutinee} {{\n")?;

                for arm in arms {
                    write!(f, "    {} => {}\n", arm.pattern, arm.value_expr)?;
                }

                write!(f, "}}")
            }
        }
    }
}
