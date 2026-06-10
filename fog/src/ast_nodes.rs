use std::rc::Rc;

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
    Identifier(String),
    Lambda {
        parameter_name: String,
        body: Rc<Expr>,
    },
    FuncAppl {
        function_name: String,
        arguments: Vec<Box<Expr>>,
    },
    Int32Literal(i32),
    Float32Literal(f32),
    StringLiteral(String),
}

impl ToString for Expr {
    fn to_string(&self) -> String {
        match self {
            Expr::Identifier(name) => name.clone(),
            Expr::Lambda {
                parameter_name,
                body,
            } => {
                format!("{} => {}", parameter_name, body.to_string())
            }
            Expr::FuncAppl {
                function_name,
                arguments,
            } => {
                let args: String = arguments
                    .iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", function_name, args)
            }
            Expr::Int32Literal(value) => value.to_string(),
            Expr::Float32Literal(value) => value.to_string(),
            Expr::StringLiteral(value) => format!("\"{}\"", value),
        }
    }
}
