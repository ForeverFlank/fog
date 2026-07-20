use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;

use crate::parser::core_expr::CoreAtomicTypeExpr;
use crate::static_check::kind::Kind;
use crate::util::fmt_parenthesized;
use crate::util::format_joined;

// --- type ---

#[derive(Clone, Eq)]
pub enum Type {
    Int32,
    Float32,
    Char,
    String,
    IOUnit, // HACK super temporary hack; to be replaced with actual IO monad!

    Function(Box<Type>, Box<Type>),
    Product(Vec<Type>),
    Named(String, Vec<Type>), // sum type and stuff

    // parametric polymorphism
    Variable(String),

    TypeConstructor(String, Box<Type>),
}

impl Type {
    pub fn function(param_type: Type, return_type: Type) -> Type {
        Type::Function(param_type.into(), return_type.into())
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        eq_type(self, other, &mut HashMap::new())
    }
}

fn eq_type(type_1: &Type, type_2: &Type, var_type_map: &mut HashMap<String, String>) -> bool {
    match (type_1, type_2) {
        (Type::Int32, Type::Int32) => true,
        (Type::Float32, Type::Float32) => true,
        (Type::Char, Type::Char) => true,
        (Type::String, Type::String) => true,
        (Type::IOUnit, Type::IOUnit) => true,

        (Type::Function(p1, r1), Type::Function(p2, r2)) => {
            eq_type(p1, p2, var_type_map) && eq_type(r1, r2, var_type_map)
        }

        (Type::Product(types_1), Type::Product(types_2)) if types_1.len() == types_2.len() => {
            types_1
                .iter()
                .zip(types_2)
                .all(|(t1, t2)| eq_type(t1, t2, var_type_map))
        }

        (Type::Named(name_1, args_1), Type::Named(name_2, args_2)) => {
            let are_names_equal = name_1 == name_2;

            let are_args_equal = args_1.len() == args_2.len()
                && args_1
                    .iter()
                    .zip(args_2)
                    .all(|(t1, t2)| eq_type(t1, t2, var_type_map));

            are_names_equal && are_args_equal
        }

        (Type::Variable(name_1), Type::Variable(name_2)) => {
            match var_type_map.entry(name_1.to_string()) {
                Entry::Occupied(entry) => entry.get() == name_2,
                Entry::Vacant(entry) => {
                    entry.insert(name_2.to_string());
                    true
                }
            }
        }

        (Type::TypeConstructor(param_1, type_1), Type::TypeConstructor(param_2, type_2)) => {
            let are_params_substitutable = match var_type_map.entry(param_1.to_string()) {
                Entry::Occupied(entry) => entry.get() == param_2,
                Entry::Vacant(entry) => {
                    entry.insert(param_2.to_string());
                    true
                }
            };

            let are_types_equal = eq_type(type_1, type_2, var_type_map);

            are_params_substitutable && are_types_equal
        }

        _ => false,
    }
}

impl Type {
    pub fn substitute_var(&self, name: &str, r#type: &Type) -> Type {
        match self {
            Type::Variable(name_2) if name_2 == name => r#type.clone(),

            Type::Function(param_type, return_type) => Type::function(
                param_type.substitute_var(name, r#type),
                return_type.substitute_var(name, r#type),
            ),

            Type::Product(types) => Type::Product(
                types
                    .iter()
                    .map(|t| t.substitute_var(name, r#type))
                    .collect(),
            ),

            _ => self.clone(),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int32 => write!(f, "Int32"),
            Type::Float32 => write!(f, "Float32"),
            Type::Char => write!(f, "Char"),
            Type::String => write!(f, "String"),
            Type::IOUnit => write!(f, "IO"),

            Type::Function(param_type, return_type) => {
                fmt_parenthesized(f, param_type)?;
                write!(f, " -> {}", return_type)
            }

            Type::Product(types) => {
                if types.is_empty() {
                    write!(f, "Unit")
                } else {
                    write!(f, "{}", format_joined(types, " * "))
                }
            }

            Type::Named(name, args) => {
                write!(f, "{}", name)?;

                for arg in args {
                    write!(f, " ")?;
                    fmt_parenthesized(f, arg)?;
                }

                Ok(())
            }

            Type::Variable(name) => write!(f, "{}", name),

            Type::TypeConstructor(param, body) => {
                write!(f, "{} => {}", param, body)?;

                Ok(())
            }
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
        Type::TypeConstructor(_, _) => Kind::Function(Kind::Type.into(), Kind::Type.into()),
        _ => Kind::Type,
    }
}
