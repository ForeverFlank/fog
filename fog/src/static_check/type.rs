use std::fmt;

use crate::parser::core_expr::CoreAtomicTypeExpr;
use crate::static_check::kind::Kind;
use crate::util::fmt_parenthesized;
use crate::util::format_joined;

// --- type ---

#[derive(Clone, Debug, Eq)]
pub enum Type {
    Int32,
    Float32,
    Char,
    String,
    IOUnit, // HACK super temporary hack; to be replaced with actual IO monad!

    Function(Box<Type>, Box<Type>),
    Product(Vec<Type>),
    Named(String, Vec<Type>),

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
        eq_type(self, other, &mut 0)
    }
}

fn eq_type(type_1: &Type, type_2: &Type, counter: &mut i32) -> bool {
    match (type_1, type_2) {
        (Type::Int32, Type::Int32) => true,
        (Type::Float32, Type::Float32) => true,
        (Type::Char, Type::Char) => true,
        (Type::String, Type::String) => true,
        (Type::IOUnit, Type::IOUnit) => true,

        (Type::Function(p1, r1), Type::Function(p2, r2)) => {
            eq_type(p1, p2, counter) && eq_type(r1, r2, counter)
        }

        (Type::Product(types_1), Type::Product(types_2)) if types_1.len() == types_2.len() => {
            types_1
                .iter()
                .zip(types_2.iter())
                .all(|(t1, t2)| eq_type(t1, t2, counter))
        }

        (Type::Named(name_1, args_1), Type::Named(name_2, args_2)) => {
            let are_names_equal = name_1 == name_2;
            let are_args_equal = args_1.len() == args_2.len()
                && args_1
                    .iter()
                    .zip(args_2.iter())
                    .all(|(t1, t2)| eq_type(t1, t2, counter));

            are_names_equal && are_args_equal
        }

        (Type::Variable(name_1), Type::Variable(name_2)) => name_1 == name_2,

        (Type::TypeConstructor(param_1, type_1), Type::TypeConstructor(param_2, type_2)) => {
            let substituted_param = Type::Variable(format!("$param{}", *counter));
            *counter += 1;

            let type_1 = type_1.substitute_var(param_1, &substituted_param);
            let type_2 = type_2.substitute_var(param_2, &substituted_param);

            eq_type(&type_1, &type_2, counter)
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

            Type::Named(type_name, args) => Type::Named(
                type_name.clone(),
                args.iter()
                    .map(|t| t.substitute_var(name, r#type))
                    .collect(),
            ),

            Type::TypeConstructor(param, body) if param != name => {
                Type::TypeConstructor(param.clone(), body.substitute_var(name, r#type).into())
            }

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
            Type::Variable(name) => write!(f, "{}", name),

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

            Type::TypeConstructor(param, body) => write!(f, "{} => {}", param, body),
        }
    }
}

// --- scheme ---

#[derive(Clone, Debug, Eq)]
pub enum Scheme {
    Mono(Type),
    Poly(Vec<String>, Type),
}

impl PartialEq for Scheme {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Scheme::Mono(type_1), Scheme::Mono(type_2)) => type_1 == type_2,

            (Scheme::Poly(vars_1, type_1), Scheme::Poly(vars_2, type_2)) => {
                if vars_1.len() != vars_2.len() {
                    return false;
                }

                let mut type_1 = type_1.clone();
                let mut type_2 = type_2.clone();

                for (i, (var_1, var_2)) in vars_1.iter().zip(vars_2.iter()).enumerate() {
                    let substitute_var = Type::Variable(format!("$var{}", i));

                    type_1 = type_1.substitute_var(var_1, &substitute_var);
                    type_2 = type_2.substitute_var(var_2, &substitute_var);
                }

                type_1 == type_2
            }

            _ => false,
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

// --- test ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpha_equivalence() {
        let left = Scheme::Poly(
            vec!["a".to_string()],
            Type::Variable("a".to_string()).into(),
        );
        let right = Scheme::Poly(
            vec!["b".to_string()],
            Type::Variable("b".to_string()).into(),
        );

        assert_eq!(left, right);
    }
}
