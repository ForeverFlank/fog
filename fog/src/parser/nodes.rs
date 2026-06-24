use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use crate::error::Span;

// --- AST nodes ---

pub struct Program {
    pub statements: Vec<Statement>,
}

pub enum Statement {
    TypeAnnotation(String, Expr, Span),
    Declaration(String, Expr, Span),
}

// --- expressions ---

pub enum Expr {
    Identifier(String),

    Int32Literal(i32),
    Float32Literal(f32),

    Lambda {
        param: String,
        param_type: Box<Expr>,
        body: Rc<Expr>,
    },

    // bunch of names, undecided if it's a
    // function application or a data constructor
    NameCollection(Vec<Box<Expr>>),

    FuncAppl {
        function: String,
        args: Vec<Box<Expr>>,
    },

    DataConstructor(String, Vec<Box<Expr>>),
}

impl Expr {
    fn fmt_parenthesized(expr: &Expr, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = expr.to_string();

        if s.contains(' ') {
            write!(f, "({s})")
        } else {
            write!(f, "{s}")
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Identifier(name) => write!(f, "{name}"),

            Expr::Int32Literal(value) => write!(f, "{value}"),
            Expr::Float32Literal(value) => write!(f, "{value}"),

            Expr::Lambda { param, body, .. } => {
                write!(f, "{param} => {body}")
            }

            Expr::NameCollection(names) => {
                for (i, name) in names.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    Self::fmt_parenthesized(name, f)?;
                }
                Ok(())
            }

            Expr::FuncAppl { function, args } => {
                write!(f, "{function}")?;

                for arg in args {
                    write!(f, " ")?;
                    Self::fmt_parenthesized(arg, f)?;
                }

                Ok(())
            }

            Expr::DataConstructor(name, types) => {
                write!(f, "{name}")?;

                for ty in types {
                    write!(f, " ")?;
                    Self::fmt_parenthesized(ty, f)?;
                }

                Ok(())
            }
        }
    }
}
