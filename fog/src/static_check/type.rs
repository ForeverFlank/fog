use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Display;
use std::vec;

use crate::parser::core_expr::CoreAtomicTypeExpr;
use crate::static_check::kind::Kind;
use crate::static_check::type_class::Constraint;
use crate::util::fmt_parenthesized;
use crate::util::format_joined;

// --- monotype ---

#[derive(Clone, Debug, Eq)]
pub enum Monotype {
    Int32,
    Float32,
    Char,
    String,

    IO(Box<Monotype>),

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

        (Monotype::IO(t1), Monotype::IO(t2)) => eq_monotype(t1, t2, counter),

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

impl Display for Monotype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Monotype::Int32 => write!(f, "Int32"),
            Monotype::Float32 => write!(f, "Float32"),
            Monotype::Char => write!(f, "Char"),
            Monotype::String => write!(f, "String"),
            Monotype::Variable(name) => write!(f, "{}", name),

            Monotype::IO(t) => {
                write!(f, "IO ")?;
                fmt_parenthesized(f, t)
            }

            Monotype::Function(param_type, return_type) => {
                fmt_parenthesized(f, param_type)?;
                write!(f, " -> {}", return_type)?;

                Ok(())
            }

            Monotype::Product(types) => {
                if types.is_empty() {
                    write!(f, "Unit")?;
                } else {
                    write!(f, "{}", format_joined(types, " * "))?;
                }

                Ok(())
            }

            Monotype::Named(name, args) => {
                write!(f, "{}", name)?;

                for arg in args {
                    write!(f, " ")?;
                    fmt_parenthesized(f, arg)?;
                }

                Ok(())
            }

            Monotype::TypeConstructor(param, body) => {
                write!(f, "{} => {}", param, body)
            }
        }
    }
}

// --- scheme ---

#[derive(Clone, Debug, Eq)]
pub struct Type {
    pub vars: Vec<String>,
    pub monotype: Monotype,
}

impl Type {
    pub fn mono(monotype: Monotype) -> Type {
        Type {
            vars: vec![],
            monotype,
        }
    }

    pub fn poly(vars: Vec<String>, monotype: Monotype) -> Type {
        Type { vars, monotype }
    }

    pub fn function(param_type: &Type, return_type: &Type) -> Type {
        let mut type_vars = BTreeSet::new();
        type_vars.extend(param_type.vars.clone());
        type_vars.extend(return_type.vars.clone());

        Type {
            vars: type_vars.into_iter().collect(),
            monotype: Monotype::Function(
                param_type.monotype.clone().into(),
                return_type.monotype.clone().into(),
            ),
        }
    }

    pub fn product(types: &Vec<Type>) -> Type {
        let mut type_vars = BTreeSet::new();
        let mut monotypes = Vec::new();

        for t in types {
            type_vars.extend(t.vars.clone());
            monotypes.push(t.monotype.clone());
        }

        Type {
            vars: type_vars.into_iter().collect(),
            monotype: Monotype::Product(monotypes),
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        if self.vars.len() != other.vars.len() {
            return false;
        }

        let mut monotype_1 = self.monotype.clone();
        let mut monotype_2 = other.monotype.clone();

        for (i, (var_1, var_2)) in self.vars.iter().zip(other.vars.iter()).enumerate() {
            let substitute_var = Monotype::Variable(format!("$var{}", i));

            monotype_1 = monotype_1.substitute_var(var_1, &substitute_var);
            monotype_2 = monotype_2.substitute_var(var_2, &substitute_var);
        }

        monotype_1 == monotype_2
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.vars.is_empty() {
            write!(f, "∀")?;

            for var in self.vars.as_slice() {
                write!(f, " {}", var)?;
            }

            write!(f, ". ")?;
        }

        write!(f, "{}", self.monotype)?;

        Ok(())
    }
}

// --- data constructor ---

#[derive(Clone)]
pub struct DataConstructor {
    pub tag: String,
    pub types: Vec<CoreAtomicTypeExpr>,
}

impl Display for DataConstructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)?;

        for r#type in &self.types {
            write!(f, " {}", r#type)?;
        }

        Ok(())
    }
}

// --- functions ---

pub fn kind_of(monotype: &Monotype) -> Kind {
    match monotype {
        Monotype::TypeConstructor(_, t) => {
            Kind::Function(Kind::Type.into(), kind_of(t.as_ref()).into())
        }

        _ => Kind::Type,
    }
}

// --- test ---

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_alpha_equivalence() {
        let a = Type {
            vars: vec!["a".to_string()],
            monotype: Monotype::Variable("a".to_string()),
        };
        let b = Type {
            vars: vec!["b".to_string()],
            monotype: Monotype::Variable("b".to_string()),
        };

        assert_eq!(a, b);
    }
}
