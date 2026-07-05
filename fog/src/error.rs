use std::fmt;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone)]
pub enum ErrorKind {
    Lex,
    Parse,
    TypeCheck,
    Runtime,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Lex => write!(f, "lexer"),
            ErrorKind::Parse => write!(f, "parser"),
            ErrorKind::TypeCheck => write!(f, "static check"),
            ErrorKind::Runtime => write!(f, "runtime"),
        }
    }
}

#[derive(Clone)]
pub struct FogError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<Span>,
}

impl fmt::Display for FogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.span {
            Some(span) => write!(
                f,
                "{} error ({}:{}): {}",
                self.kind, span.line, span.column, self.message
            ),
            None => write!(f, "{} error: {}", self.kind, self.message),
        }
    }
}

#[macro_export]
macro_rules! lex_error {
    ($span:expr, $($arg:tt)*) => {
        $crate::error::FogError {
            kind: $crate::error::ErrorKind::Lex,
            message: format!($($arg)*),
            span: $span,
        }
    };
}

#[macro_export]
macro_rules! parse_error {
    ($span:expr, $($arg:tt)*) => {
        $crate::error::FogError {
            kind: $crate::error::ErrorKind::Parse,
            message: format!($($arg)*),
            span: $span,
        }
    };
}

#[macro_export]
macro_rules! type_check_error {
    ($span:expr, $($arg:tt)*) => {
        $crate::error::FogError {
            kind: $crate::error::ErrorKind::TypeCheck,
            message: format!($($arg)*),
            span: $span,
        }
    };
}

#[macro_export]
macro_rules! runtime_error {
    ($span:expr, $($arg:tt)*) => {
        $crate::error::FogError {
            kind: $crate::error::ErrorKind::Runtime,
            message: format!($($arg)*),
            span: $span,
        }
    };
}

pub type FogResult<T> = Result<T, FogError>;
