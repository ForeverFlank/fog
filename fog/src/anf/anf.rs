use crate::parser::Literal;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreMatchArmPattern;

pub enum ANFExpr {
    Atomic(AtomicExpr),
    Declaration(CoreDeclPattern, Box<ANFExpr>),
    FunctionAppl(AtomicExpr, AtomicExpr),
}

pub enum AtomicExpr {
    Literal {
        literal: Literal,
    },
    Identifier {
        name: String,
    },
    Lambda {
        param_name: String,
        body: Box<ANFExpr>,
    },
    Tuple {
        items: Vec<ANFExpr>,
    },
    Match {
        scrutinee: Box<AtomicExpr>,
        arms: Vec<(CoreMatchArmPattern, ANFExpr)>,
    },
}
