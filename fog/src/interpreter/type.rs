use crate::error::FogError;
use crate::error::FogResult;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval::eval_type_expr;
use crate::interpreter::value::Value;
use crate::parser::nodes::Expr;

// --- type ---

#[derive(Clone, Eq, Hash)]
pub enum Type {
    Kind,

    Type,
    Function(Box<Type>, Box<Type>),

    // primitive types
    Int32,
    Float32,
    Unit,

    // ADTs
    Product(Vec<Type>),
    Sum(String, Vec<DataConstructor>),
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
            (Type::Function(d1, r1), Type::Function(d2, r2)) => d1 == d2 && r1 == r2,

            (Type::Int32, Type::Int32) => true,
            (Type::Float32, Type::Float32) => true,
            (Type::Unit, Type::Unit) => true,

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
            Type::Unit => "Unit".to_string(),

            Type::Product(types) => types.iter().fold(String::new(), |acc, r#type| {
                acc + " * " + &r#type.to_string()
            }),
            Type::Sum(_, ctors) => ctors.iter().fold(String::new(), |acc, r#type| {
                acc + " + " + &r#type.to_string()
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

pub fn value_type_of(value: &Value, env: &Environment) -> FogResult<Type> {
    match value {
        Value::Int32(_) => Ok(Type::Int32),
        Value::Float32(_) => Ok(Type::Float32),

        Value::Function {
            param_type, body, ..
        } => Ok(Type::function(
            param_type.clone(),
            expr_type_of(&body, env)?,
        )),

        Value::NativeFunction {
            param_type,
            return_type,
            ..
        } => Ok(Type::function(param_type.clone(), return_type.clone())),

        Value::EmptyTuple => Ok(Type::Unit),
    }
}

pub fn expr_type_of(expr: &Expr, env: &Environment) -> FogResult<Type> {
    match expr {
        Expr::Identifier(name) => Ok(env.get_var(&name)?.r#type),

        Expr::Int32Literal(_) => Ok(Type::Int32),
        Expr::Float32Literal(_) => Ok(Type::Float32),

        Expr::Lambda {
            param_type, body, ..
        } => Ok(Type::Function(
            eval_type_expr(&param_type, env)?.into(),
            expr_type_of(&body, env)?.into(),
        )),

        Expr::FuncAppl {
            fn_name: function,
            args,
        } => {
            let Type::Function(_, return_type) = env.get_var(&function)?.r#type else {
                return Err(FogError::runtime(
                    format!("`{}` is not a function", function),
                    None,
                ));
            };

            let mut curr_return_type: Type = *return_type;

            for _ in args {
                curr_return_type = match curr_return_type {
                    Type::Function(_, r) => *r,
                    _ => {
                        return Err(FogError::runtime(
                            "expected function type".to_string(),
                            None,
                        ));
                    }
                }
            }

            Ok(curr_return_type)
        }

        Expr::DataConstructor { type_name, .. } => Ok(env.get_type(type_name)?),

        Expr::NameCollection(_) => Err(FogError::runtime(
            "unresolved name collection".to_string(),
            None,
        )),
    }
}
