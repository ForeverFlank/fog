use std::fmt;

use crate::parser::core_expr::CoreAtomicTypeExpr;
use crate::static_check::kind::Kind;
use crate::util::format_joined;

// --- type ---

#[derive(Clone, Eq)]
pub enum Type {
    Function(Box<Type>, Box<Type>),

    // primitive types
    Int32,
    Float32,
    Char,
    String,
    IOUnit, // HACK super temporary hack; to be replaced with actual IO monad!

    // ADTs
    Product(Vec<Type>),
    Sum(String), // nominally-typed
}

impl Type {
    pub fn function(param_type: Type, return_type: Type) -> Type {
        Type::Function(param_type.into(), return_type.into())
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Type::Function(param_type_1, return_type_1),
                Type::Function(param_type_2, return_type_2),
            ) => param_type_1 == param_type_2 && return_type_1 == return_type_2,

            (Type::Int32, Type::Int32) => true,
            (Type::Float32, Type::Float32) => true,
            (Type::Char, Type::Char) => true,
            (Type::String, Type::String) => true,
            (Type::IOUnit, Type::IOUnit) => true,

            (Type::Product(types_1), Type::Product(types_2)) => types_1 == types_2,

            (Type::Sum(name_1), Type::Sum(name_2)) => name_1 == name_2,

            _ => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Function(param_type, return_type) => {
                write!(f, "{} -> {}", param_type, return_type)
            }

            Type::Int32 => write!(f, "Int32"),
            Type::Float32 => write!(f, "Float32"),
            Type::Char => write!(f, "Char"),
            Type::String => write!(f, "String"),
            Type::IOUnit => write!(f, "IO"),

            Type::Product(types) => {
                if types.is_empty() {
                    write!(f, "Unit")
                } else {
                    write!(f, "{}", format_joined(types, " * "))
                }
            }

            Type::Sum(name) => write!(f, "{}", name),
        }
    }
}

// --- data constructor ---

#[derive(Clone)]
pub struct DataConstructor {
    pub tag: String,
    pub types: Vec<CoreAtomicTypeExpr>,
}

impl fmt::Display for DataConstructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)?;

        for r#type in &self.types {
            write!(f, " {}", r#type)?;
        }

        Ok(())
    }
}

// --- functions ---

pub fn kind_of(r#type: &Type) -> Kind {
    match r#type {
        Type::Function(_, _) => Kind::Function(
            Kind::Type.into(),
            Kind::Function(Kind::Type.into(), Kind::Type.into()).into(),
        ),
        _ => Kind::Type,
    }
}
