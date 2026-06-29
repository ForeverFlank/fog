use std::fmt;
use std::rc::Rc;

use crate::error::FogResult;
use crate::interpreter::environment::Environment;
use crate::interpreter::r#type::Type;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::util::format_joined;

#[derive(Clone)]
pub enum Value {
    Int32(i32),
    Float32(f32),

    Function {
        param_name: String,
        param_type: Type,
        return_type: Type,
        body: Rc<ResolvedExpr>,
        captured_env: Box<Environment<'static>>,
    },
    NativeFunction {
        param_type: Type,
        return_type: Type,
        function: Rc<dyn Fn(Value) -> FogResult<Value>>,
    },

    Tuple(Vec<Value>),

    Constructor {
        tag: String,
        values: Vec<Value>,
        r#type: Type,
    },
}

pub fn value_type_of(value: &Value) -> Type {
    match value {
        Value::Int32(_) => Type::Int32,
        Value::Float32(_) => Type::Float32,

        Value::Function {
            param_type,
            return_type,
            ..
        } => Type::function(param_type.clone(), return_type.clone()),

        Value::NativeFunction {
            param_type,
            return_type,
            ..
        } => Type::function(param_type.clone(), return_type.clone()),

        Value::Tuple(values) => Type::Product(values.iter().map(value_type_of).collect()),

        Value::Constructor { r#type, .. } => r#type.clone(),
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int32(value) => write!(f, "{}", value),
            Value::Float32(value) => write!(f, "{}", value),

            Value::Function {
                param_name, body, ..
            } => write!(f, "{} => {}", param_name, body),

            Value::NativeFunction { .. } => write!(f, "[native function]"),

            Value::Tuple(values) => write!(f, "({})", format_joined(values, ", ")),

            Value::Constructor { tag, values, .. } => {
                if values.is_empty() {
                    write!(f, "{}", tag)
                } else {
                    write!(f, "{} {}", tag, format_joined(values, " "))
                }
            }
        }
    }
}
