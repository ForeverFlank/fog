#[derive(Clone)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone)]
pub enum ErrorKind {
    Lex,
    Parse,
    Runtime,
}

#[derive(Clone)]
pub struct FogError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<Span>,
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
