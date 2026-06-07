// --- AST nodes ---

pub struct Program {
    pub statements: Vec<Statement>,
}

pub enum Statement {
    TypeAnnotation(Identifier, Expr),
    Declaration(Identifier, Expr),
}

pub struct Identifier(pub String);

impl Identifier {
    pub fn new(name: &str) -> Self {
        Identifier(name.to_string())
    }
}

// --- expressions ---

pub enum Expr {
    Identifier(Identifier), // TODO just use string lol
    Lambda(Lambda),
    FuncAppl(FuncAppl),
    Int32Literal(i32),
    Float32Literal(f32),
    StringLiteral(String),
}

impl ToString for Expr {
    fn to_string(&self) -> String {
        match self {
            Expr::Identifier(ident) => ident.0.clone(),
            Expr::Lambda(Lambda { parameter, body }) => {
                format!("{} => {}", parameter.0, body.to_string())
            }
            Expr::FuncAppl(FuncAppl {
                function,
                arguments,
            }) => {
                let args = arguments
                    .iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", function.0, args)
            }
            Expr::Int32Literal(value) => value.to_string(),
            Expr::Float32Literal(value) => value.to_string(),
            Expr::StringLiteral(value) => format!("\"{}\"", value),
        }
    }
}

pub struct Lambda {
    pub parameter: Identifier,
    pub body: Box<Expr>,
}

pub struct FuncAppl {
    pub function: Identifier,
    pub arguments: Vec<Box<Expr>>,
}
