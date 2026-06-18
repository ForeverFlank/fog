use std::rc::Rc;

use crate::{
    ast::nodes::Expr,
    interpreter::{environment::Environment, value::Value},
};

#[derive(Clone, Eq, Hash)]
pub enum Type {
    Kind,
    Type,
    Int32,
    Float32,
    Function(Rc<Type>, Rc<Type>),
    Sum(Vec<DataConstructor>),
    Product(Vec<Rc<Type>>),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DataConstructor {
    pub variant: String,
    pub types: Vec<Rc<Type>>,
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Kind, Type::Kind) => true,
            (Type::Type, Type::Type) => true,
            (Type::Int32, Type::Int32) => true,
            (Type::Float32, Type::Float32) => true,
            (Type::Function(d1, r1), Type::Function(d2, r2)) => d1 == r1 && d2 == r2,
            _ => false,
        }
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        match self {
            Type::Kind => "Kind".to_string(),
            Type::Type => "Type".to_string(),
            Type::Int32 => "Int32".to_string(),
            Type::Float32 => "Float32".to_string(),
            Type::Function(param_type, return_type) => {
                format!(
                    "{} -> {}",
                    (*param_type).to_string(),
                    (*return_type).to_string()
                )
            }
            // Type::Constructor(type_constructor) => type_constructor
            //     .types
            //     .iter()
            //     .fold(String::new(), |str, t| str + " " + &t.to_string()),
            Type::Sum(items) => todo!(),
            Type::Product(items) => todo!(),
        }
    }
}

pub fn get_value_type(value: &Value, env: &Environment) -> Type {
    match value {
        Value::Type(_) => Type::Kind,
        Value::Int32(_) => Type::Int32,
        Value::Float32(_) => Type::Float32,
        Value::Function {
            param_type, body, ..
        } => Type::Function(param_type.clone(), get_expr_type(&body, env).into()),
        Value::NativeFunction {
            param_type,
            return_type,
            ..
        } => Type::Function(param_type.clone(), return_type.clone()),
    }
}

pub fn get_expr_type(expr: &Expr, env: &Environment) -> Type {
    match *expr {
        Expr::Int32Literal(_) => *env.get_var("Int32")?.r#type,
        Expr::Float32Literal(_) => *env.get_var("Float32")?.r#type,
        Expr::Identifier(name) => env.get_var(name)?.r#type,
    }
}

pub fn is_value_of_type(value: &Value, r#type: &Type) -> bool {
    match (value, r#type) {
        (Value::Type(_), Type::Type) => true,
        (Value::Int32(_), Type::Int32) => true,
        (Value::Float32(_), Type::Float32) => true,
        (Value::Function { .. }, Type::Function(_, _)) => true,
        _ => false,
    }
}
