use std::rc::Rc;

use crate::ast::nodes::Expr;
use crate::interpreter::environment::Environment;
use crate::interpreter::interpreter::InterpreterError;
use crate::interpreter::r#type::Type;

#[derive(Clone)]
pub enum Value {
    Type(Type),
    Int32(i32),
    Float32(f32),
    Function {
        param: String,
        param_type: Type,
        body: Rc<Expr>,
        captured_env: Box<Environment>,
    },
    NativeFunction {
        param_type: Type,
        return_type: Type,
        function: Rc<dyn Fn(Value) -> Result<Value, InterpreterError>>,
    },
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Type(r#type) => (*r#type).to_string(),
            Value::Int32(value) => value.to_string(),
            Value::Float32(value) => value.to_string(),
            Value::Function { param, body, .. } => {
                format!("{} => {}", param, (*body).to_string())
            }
            Value::NativeFunction { .. } => "[native function]".to_string(),
        }
    }
}
