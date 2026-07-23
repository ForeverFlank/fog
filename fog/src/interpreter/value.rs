use std::fmt;
use std::rc::Rc;

use crate::anf::anf::ANFValue;
use crate::error::FogResult;
use crate::interpreter::environment::Environment;
use crate::util::format_joined;

#[derive(Clone)]
pub enum Value {
    Int32(i32),
    Float32(f32),
    Char(char),
    String(String),

    Function {
        param: String,
        body: Rc<ANFValue>,
        captured_env: Box<Environment<'static>>,
    },
    Tuple(Vec<Value>),
    Constructor {
        tag: String,
        values: Vec<Value>,
    },

    NativeFunction(Rc<dyn Fn(Value) -> FogResult<Value>>),
    IO(Rc<dyn Fn() -> FogResult<Value>>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int32(value) => write!(f, "{}", value),
            Value::Float32(value) => write!(f, "{}", value),
            Value::Char(value) => write!(f, "{:?}", value),
            Value::String(value) => write!(f, "{:?}", value),

            Value::Function { param, body, .. } => write!(f, "{param} => {body}"),

            Value::Tuple(values) => write!(f, "({})", format_joined(values, ", ")),

            Value::Constructor { tag, values, .. } => {
                if values.is_empty() {
                    write!(f, "{}", tag)
                } else {
                    write!(f, "{} {}", tag, format_joined(values, " "))
                }
            }

            Value::NativeFunction(..) => write!(f, "[native function]"),
            Value::IO(..) => write!(f, "[IO]"),
        }
    }
}
