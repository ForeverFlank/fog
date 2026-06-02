// --- AST nodes ---

pub struct Program {
    pub items: Vec<Item>,
}

pub enum Item {
    TypeAnnotation(TypeAnnotation),
    Declaration(Declaration),
}

pub struct TypeAnnotation {
    pub identifier: Identifier,
    pub r#type: TypeExpr,
}

pub struct Declaration {}

pub struct Identifier {
    pub name: String,
}

// --- value expressions ---

pub enum ValueExpr {
    Identifier(Identifier),
    Binary(ValueBinaryExpr),
}

pub struct ValueBinaryExpr {
    pub lhs: Box<ValueExpr>,
    pub op: ValueBinaryOp,
    pub rhs: Box<ValueExpr>,
}

pub enum ValueBinaryOp {
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

// --- type expressions ---

pub enum TypeExpr {
    Identifier(Identifier),
    Binary(TypeBinaryExpr),
}

pub struct TypeBinaryExpr {
    pub lhs: Box<ValueExpr>,
    pub op: ValueBinaryOp,
    pub rhs: Box<ValueExpr>,
}

pub enum TypeBinaryOp {
    Plus,
    Star,
    Arrow,
}
