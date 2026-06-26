use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval_type::eval_type_annotation_expr;
use crate::interpreter::r#type::Type::Product;
use crate::interpreter::value::Value;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::util::format_joined;

// --- type ---

#[derive(Clone, Eq)]
pub enum Type {
    Kind,
    Type,
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
        Type::Function(Box::new(param_type), Box::new(return_type))
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Kind, Type::Kind) => true,
            (Type::Type, Type::Type) => true,

            (Type::Int32, Type::Int32) => true,
            (Type::Float32, Type::Float32) => true,

            (
                Type::Function(param_type_1, return_type_1),
                Type::Function(param_type_2, return_type_2),
            ) => param_type_1 == param_type_2 && return_type_1 == return_type_2,

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

impl ToString for Type {
    fn to_string(&self) -> String {
        match self {
            Type::Kind => "Kind".to_string(),
            Type::Type => "Type".to_string(),
            Type::Function(param_type, return_type) => {
                format!("{} -> {}", param_type.to_string(), return_type.to_string())
            }

            Type::Int32 => "Int32".to_string(),
            Type::Float32 => "Float32".to_string(),

            Type::Product(types) => {
                if types.is_empty() {
                    "Unit".to_string()
                } else {
                    format_joined(types, " * ")
                }
            }

            Type::Sum(ctors) => format_joined(ctors, " + "),
        }
    }
}

// --- data constructor ---

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DataConstructor {
    pub tag: String,
    pub types: Vec<Type>,
}

impl std::fmt::Display for DataConstructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tag)?;

        for r#type in &self.types {
            write!(f, " {}", r#type.to_string())?;
        }

        Ok(())
    }
}

// --- functions ---

pub fn nest_function_types(field_types: &[Type], return_type: Type) -> Type {
    field_types.iter().rev().fold(return_type, |ret, ft| {
        Type::Function(Box::new(ft.clone()), Box::new(ret))
    })
}

pub fn value_type_of(value: &Value, env: &Environment, span: &Span) -> FogResult<Type> {
    match value {
        Value::Int32(_) => Ok(Type::Int32),
        Value::Float32(_) => Ok(Type::Float32),

        Value::Function {
            param_type, body, ..
        } => Ok(Type::function(
            param_type.clone(),
            expr_type_of(&body, env, span)?,
        )),

        Value::NativeFunction {
            param_type,
            return_type,
            ..
        } => Ok(Type::function(param_type.clone(), return_type.clone())),

        Value::Tuple(values) => Ok(Type::Product(
            values
                .iter()
                .map(|value| value_type_of(value, env, span))
                .collect::<Result<Vec<Type>, FogError>>()?,
        )),

        Value::Constructor { r#type, .. } => Ok(r#type.clone()),
    }
}

pub fn expr_type_of(expr: &ResolvedExpr, env: &Environment, span: &Span) -> FogResult<Type> {
    match expr {
        ResolvedExpr::Identifier { name } => Ok(env.get_var(name, span)?.r#type),

        ResolvedExpr::Int32Literal { .. } => Ok(Type::Int32),
        ResolvedExpr::Float32Literal { .. } => Ok(Type::Float32),

        ResolvedExpr::Lambda {
            param_type, body, ..
        } => Ok(Type::Function(
            eval_type_annotation_expr(param_type, env, span)?.into(),
            expr_type_of(body, env, span)?.into(),
        )),

        ResolvedExpr::FuncAppl { fn_name, args } => {
            let mut curr_type = env.get_var(fn_name, span)?.r#type.clone();

            for _ in args {
                curr_type = match curr_type {
                    Type::Function(_, return_type) => *return_type,
                    _ => {
                        return Err(FogError::runtime(
                            format!("{} is not a function type", curr_type.to_string()),
                            Some(span.clone()),
                        ));
                    }
                };
            }

            Ok(curr_type)
        }

        ResolvedExpr::Tuple { items } => Ok(Product(
            items
                .iter()
                .map(|expr| expr_type_of(expr, env, span))
                .collect::<Result<Vec<Type>, FogError>>()?,
        )),
    }
}
