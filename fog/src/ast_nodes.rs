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
    Identifier(Identifier),
    Lambda(Lambda),
    FuncAppl(FuncAppl),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
}

pub struct Lambda {
    pub parameter: Identifier,
    pub body: Box<Expr>,
}

pub struct FuncAppl {
    pub function: Identifier,
    pub arguments: Vec<Box<Expr>>,
}
