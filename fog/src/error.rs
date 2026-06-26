#[derive(Clone)]
pub struct Span {
    pub pos: usize,
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

impl FogError {
    pub fn lex(message: String, span: Option<Span>) -> FogError {
        FogError {
            kind: ErrorKind::Lex,
            message,
            span,
        }
    }

    pub fn parse(message: String, span: Option<Span>) -> FogError {
        FogError {
            kind: ErrorKind::Parse,
            message,
            span,
        }
    }

    pub fn runtime(message: String, span: Option<Span>) -> FogError {
        FogError {
            kind: ErrorKind::Runtime,
            message,
            span,
        }
    }
}

pub type FogResult<T> = Result<T, FogError>;
