use crate::ast::nodes::Expr;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval::eval_type_expr;
use crate::interpreter::interpreter::InterpreterError;
use crate::interpreter::value::Value;

// --- type ---

#[derive(Clone, Eq, Hash)]
pub enum Type {
    Kind,
    Type,
    Int32,
    Float32,
    Function(Box<Type>, Box<Type>),
    Sum(Vec<DataConstructor>),
    Product(Vec<Type>),
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
            Type::Sum(ctors) => ctors.iter().fold(String::new(), |acc, r#type| {
                acc + " + " + &r#type.to_string()
            }),
            Type::Product(types) => types.iter().fold(String::new(), |acc, r#type| {
                acc + " * " + &r#type.to_string()
            }),
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

pub fn get_value_type(value: &Value, env: &Environment) -> Result<Type, InterpreterError> {
    match value {
        Value::Type(_) => Ok(Type::Kind),

        Value::Int32(_) => Ok(Type::Int32),
        Value::Float32(_) => Ok(Type::Float32),

        Value::Function {
            param_type, body, ..
        } => Ok(Type::Function(
            Box::new(param_type.clone()),
            get_expr_type(&body, env)?.into(),
        )),

        Value::NativeFunction {
            param_type,
            return_type,
            ..
        } => Ok(Type::Function(
            Box::new(param_type.clone()),
            Box::new(return_type.clone()),
        )),
    }
}

pub fn get_expr_type(expr: &Expr, env: &Environment) -> Result<Type, InterpreterError> {
    match expr {
        Expr::Identifier(name) => Ok(env.get_var(&name)?.r#type),

        Expr::Int32Literal(_) => Ok(env.get_var("Int32")?.r#type),
        Expr::Float32Literal(_) => Ok(env.get_var("Float32")?.r#type),

        Expr::Lambda {
            param_type, body, ..
        } => Ok(Type::Function(
            eval_type_expr(&param_type, env)?.into(),
            get_expr_type(&body, env)?.into(),
        )),

        Expr::FuncAppl { function, args } => {
            let Type::Function(_, return_type) = env.get_var(&function)?.r#type else {
                return Err(InterpreterError::from_string(format!(
                    "`{}` is not a function",
                    function
                )));
            };

            let mut curr_return_type: Type = *return_type;

            for _ in args {
                curr_return_type = match curr_return_type {
                    Type::Function(_, r) => *r,
                    _ => return Err(InterpreterError::from_str("expected function type")),
                }
            }

            Ok(curr_return_type)
        }
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
