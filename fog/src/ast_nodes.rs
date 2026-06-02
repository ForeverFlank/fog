// --- AST nodes ---

pub struct Program {
    pub items: Vec<Statement>,
}

pub enum Statement {
    TypeAnnotation(TypeAnnotation),
    Declaration(Declaration),
}

pub struct TypeAnnotation {
    pub identifier: Identifier,
    pub typeExpr: Expr,
}

pub struct Declaration {
    pub identifier: Identifier,
    pub expr: Expr,
}

pub struct Identifier {
    pub name: String,
}

// --- expressions ---

pub enum Expr {
    Identifier(Identifier),
    Binary(BinaryExpr),
}

pub struct BinaryExpr {
    pub lhs: Box<Expr>,
    pub op: BinaryOp,
    pub rhs: Box<Expr>,
}

pub enum BinaryOp {
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Concat,
    LeftPipe,
    RightPipe,
    LeftComposition,
    RightComposition,
}
