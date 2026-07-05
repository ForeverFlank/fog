use crate::parser::Literal;

enum ANFExpr {
    Atomic(AtomicExpr),
}

enum AtomicExpr {
    Literal(Literal),
    Identifier(String),
    Lambda(String, ANFExpr),
}
