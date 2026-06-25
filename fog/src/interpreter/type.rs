use std::collections::HashSet;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval::eval_type_expr;
use crate::interpreter::r#type::Type::Product;
use crate::interpreter::value::Value;
use crate::parser::parsed_expr::ParsedExpr;

// --- type ---

#[derive(Clone, Eq, Hash)]
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
    // data constructor
    // DataConstructor(DataConstructor),
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
                let s1: HashSet<&DataConstructor> = ctors_1.iter().collect();
                let s2: HashSet<&DataConstructor> = ctors_2.iter().collect();

                s1 == s2
            }

            _ => false,
        }
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        match self {
            Type::Kind => "Kind".to_string(),
            Type::Type => "Type".to_string(),
            Type::Function(param_type, return_type) => {
                format!(
                    "{} -> {}",
                    (*param_type).to_string(),
                    (*return_type).to_string()
                )
            }

            Type::Int32 => "Int32".to_string(),
            Type::Float32 => "Float32".to_string(),

            Type::Product(types) => {
                if types.is_empty() {
                    "Unit".to_string()
                } else {
                    types
                        .iter()
                        .fold(String::new(), |acc: String, r#type: &Type| {
                            acc + " * " + &r#type.to_string()
                        })
                }
            }

            Type::Sum(ctors) => ctors
                .iter()
                .fold(String::new(), |acc: String, r#type: &DataConstructor| {
                    acc + " + " + &r#type.to_string()
                }),
            // Type::DataConstructor()
        }
    }
}

// --- data constructor ---

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DataConstructor {
    pub variant: String,
    pub types: Vec<Type>,
}

impl ToString for DataConstructor {
    fn to_string(&self) -> String {
        format!(
            "{} {}",
            self.variant,
            self.types
                .iter()
                .map(|t| t.to_string())
                .fold(String::new(), |acc, r#type| acc + " " + &r#type)
        )
    }
}

// --- functions ---

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
                .map(|value: &Value| Ok(value_type_of(value, env, span)?.into()))
                .collect::<Result<Vec<Type>, FogError>>()?,
        )),
    }
}

pub fn expr_type_of(expr: &ParsedExpr, env: &Environment, span: &Span) -> FogResult<Type> {
    match expr {
        ParsedExpr::Identifier(name) => Ok(env.get_var(&name)?.r#type),

        ParsedExpr::Int32Literal(_) => Ok(Type::Int32),
        ParsedExpr::Float32Literal(_) => Ok(Type::Float32),

        ParsedExpr::Lambda {
            param_type, body, ..
        } => Ok(Type::Function(
            eval_type_expr(&param_type, env)?.into(),
            expr_type_of(&body, env, span)?.into(),
        )),

        ParsedExpr::FuncAppl(fn_name, args) => {
            let mut curr_type: Type = env.get_var(&fn_name)?.r#type.clone();

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

        ParsedExpr::Tuple(exprs) => Ok(Product(
            exprs
                .iter()
                .map(|expr: &ParsedExpr| Ok(expr_type_of(expr, env, span)?.into()))
                .collect::<Result<Vec<Type>, FogError>>()?,
        )),

        ParsedExpr::DataConstructor(name, args) => todo!(),

        ParsedExpr::Collection(_) => Err(FogError::runtime(
            "unresolved name collection".to_string(),
            None,
        )),
    }
}
