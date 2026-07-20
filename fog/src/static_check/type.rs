use std::fmt;
use std::fmt::Display;

use crate::parser::core_expr::CoreAtomicTypeExpr;
use crate::static_check::kind::Kind;
use crate::util::fmt_parenthesized;
use crate::util::format_joined;

// --- monotype ---

#[derive(Clone, Debug, Eq)]
pub enum Monotype {
    Int32,
    Float32,
    Char,
    String,
    IOUnit, // HACK super temporary hack; to be replaced with actual IO monad!

    Function(Box<Monotype>, Box<Monotype>),
    Product(Vec<Monotype>),
    Named(String, Vec<Monotype>),

    Variable(String),
    TypeConstructor(String, Box<Monotype>),
}

impl Monotype {
    pub fn function(param_type: Monotype, return_type: Monotype) -> Monotype {
        Monotype::Function(param_type.into(), return_type.into())
    }
}

impl PartialEq for Monotype {
    fn eq(&self, other: &Self) -> bool {
        eq_monotype(self, other, &mut 0)
    }
}

fn eq_monotype(type_1: &Monotype, type_2: &Monotype, counter: &mut i32) -> bool {
    match (type_1, type_2) {
        (Monotype::Int32, Monotype::Int32) => true,
        (Monotype::Float32, Monotype::Float32) => true,
        (Monotype::Char, Monotype::Char) => true,
        (Monotype::String, Monotype::String) => true,
        (Monotype::IOUnit, Monotype::IOUnit) => true,

        (Monotype::Function(p1, r1), Monotype::Function(p2, r2)) => {
            eq_monotype(p1, p2, counter) && eq_monotype(r1, r2, counter)
        }

        (Monotype::Product(types_1), Monotype::Product(types_2))
            if types_1.len() == types_2.len() =>
        {
            types_1
                .iter()
                .zip(types_2.iter())
                .all(|(t1, t2)| eq_monotype(t1, t2, counter))
        }

        (Monotype::Named(name_1, args_1), Monotype::Named(name_2, args_2)) => {
            let are_names_equal = name_1 == name_2;
            let are_args_equal = args_1.len() == args_2.len()
                && args_1
                    .iter()
                    .zip(args_2.iter())
                    .all(|(t1, t2)| eq_monotype(t1, t2, counter));

            are_names_equal && are_args_equal
        }

        (Monotype::Variable(name_1), Monotype::Variable(name_2)) => name_1 == name_2,

        (
            Monotype::TypeConstructor(param_1, type_1),
            Monotype::TypeConstructor(param_2, type_2),
        ) => {
            let substituted_param = Monotype::Variable(format!("$param{}", *counter));
            *counter += 1;

            let type_1 = type_1.substitute_var(param_1, &substituted_param);
            let type_2 = type_2.substitute_var(param_2, &substituted_param);

            eq_monotype(&type_1, &type_2, counter)
        }

        _ => false,
    }
}

impl Monotype {
    pub fn substitute_var(&self, name: &str, r#type: &Monotype) -> Monotype {
        match self {
            Monotype::Variable(name_2) if name_2 == name => r#type.clone(),

            Monotype::Function(param_type, return_type) => Monotype::function(
                param_type.substitute_var(name, r#type),
                return_type.substitute_var(name, r#type),
            ),

            Monotype::Product(types) => Monotype::Product(
                types
                    .iter()
                    .map(|t| t.substitute_var(name, r#type))
                    .collect(),
            ),

            Monotype::Named(type_name, args) => Monotype::Named(
                type_name.clone(),
                args.iter()
                    .map(|t| t.substitute_var(name, r#type))
                    .collect(),
            ),

            Monotype::TypeConstructor(param, body) if param != name => {
                Monotype::TypeConstructor(param.clone(), body.substitute_var(name, r#type).into())
            }

            _ => self.clone(),
        }
    }
}

impl fmt::Display for Monotype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Monotype::Int32 => write!(f, "Int32"),
            Monotype::Float32 => write!(f, "Float32"),
            Monotype::Char => write!(f, "Char"),
            Monotype::String => write!(f, "String"),
            Monotype::IOUnit => write!(f, "IO"),
            Monotype::Variable(name) => write!(f, "{}", name),

            Monotype::Function(param_type, return_type) => {
                fmt_parenthesized(f, param_type)?;
                write!(f, " -> {}", return_type)
            }

            Monotype::Product(types) => {
                if types.is_empty() {
                    write!(f, "Unit")
                } else {
                    write!(f, "{}", format_joined(types, " * "))
                }
            }

            Monotype::Named(name, args) => {
                write!(f, "{}", name)?;

                for arg in args {
                    write!(f, " ")?;
                    fmt_parenthesized(f, arg)?;
                }

                Ok(())
            }

            Monotype::TypeConstructor(param, body) => write!(f, "{} => {}", param, body),
        }
    }
}

// --- type ---

#[derive(Clone, Debug, Eq)]
pub enum Type {
    Mono(Monotype),
    Poly(Vec<String>, Monotype),
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Mono(type_1), Type::Mono(type_2)) => type_1 == type_2,

            (Type::Poly(vars_1, type_1), Type::Poly(vars_2, type_2)) => {
                if vars_1.len() != vars_2.len() {
                    return false;
                }

                let mut type_1 = type_1.clone();
                let mut type_2 = type_2.clone();

                for (i, (var_1, var_2)) in vars_1.iter().zip(vars_2.iter()).enumerate() {
                    let substitute_var = Monotype::Variable(format!("$var{}", i));

                    type_1 = type_1.substitute_var(var_1, &substitute_var);
                    type_2 = type_2.substitute_var(var_2, &substitute_var);
                }

                type_1 == type_2
            }

            _ => false,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Mono(r#type) | Type::Poly(_, r#type) => r#type.fmt(f),
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

pub fn kind_of(r#type: &Monotype) -> Kind {
    match r#type {
        Monotype::TypeConstructor(_, _) => Kind::Function(Kind::Type.into(), Kind::Type.into()),
        _ => Kind::Type,
    }
}

// --- test ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpha_equivalence() {
        let left = Type::Poly(
            vec!["a".to_string()],
            Monotype::Variable("a".to_string()).into(),
        );
        let right = Type::Poly(
            vec!["b".to_string()],
            Monotype::Variable("b".to_string()).into(),
        );

        assert_eq!(left, right);
    }
}
