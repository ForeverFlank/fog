use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

use crate::interpreter::kind::Kind;
use crate::util::format_joined;

// --- type ---

#[derive(Clone, Eq)]
pub enum Type {
    Function(Box<Type>, Box<Type>),

    // primitive types
    Int32,
    Float32,

    // ADTs
    Product(Vec<Type>),
    Sum(Vec<DataConstructor>),
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

            (Type::Product(types_1), Type::Product(types_2)) => types_1 == types_2,

            (Type::Sum(ctors_1), Type::Sum(ctors_2)) => {
                let s1: HashSet<_> = ctors_1.iter().collect();
                let s2: HashSet<_> = ctors_2.iter().collect();
                s1 == s2
            }

            _ => false,
        }
    }
}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Type::Function(p, r) => {
                p.hash(state);
                r.hash(state);
            }
            Type::Product(types) => types.hash(state),
            Type::Sum(ctors) => {
                let mut sorted: Vec<_> = ctors.iter().collect();
                sorted.sort_by(|a, b| a.tag.cmp(&b.tag));
                sorted.hash(state);
            }
            _ => {}
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

            Type::Product(types) => {
                if types.is_empty() {
                    write!(f, "Unit")
                } else {
                    write!(f, "{}", format_joined(types, " * "))
                }
            }

            Type::Sum(ctors) => write!(f, "{}", format_joined(ctors, " + ")),
        }
    }
}

// --- data constructor ---

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DataConstructor {
    pub tag: String,
    pub types: Vec<Type>,
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

pub fn nest_function_types(field_types: &Vec<Type>, return_type: Type) -> Type {
    field_types.iter().rev().fold(return_type, |ret, ft| {
        Type::Function(ft.clone().into(), ret.into())
    })
}

pub fn kind_of(r#type: &Type) -> Kind {
    match r#type {
        Type::Function(_, _) => Kind::Function(
            Kind::Type.into(),
            Kind::Function(Kind::Type.into(), Kind::Type.into()).into(),
        ),
        _ => Kind::Type,
    }
}
