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
        body: Rc<ResolvedExpr>,
        captured_env: Box<Environment>,
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

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Int32(value) => value.to_string(),
            Value::Float32(value) => value.to_string(),

            Value::Function {
                param_name, body, ..
            } => {
                format!("{} => {}", param_name, (*body).to_string())
            }

            Value::NativeFunction { .. } => "[native function]".to_string(),

            Value::Tuple(values) => format!("({})", format_joined(values, ", ")),

            Value::Constructor { tag, values, .. } => {
                if values.is_empty() {
                    tag.clone()
                } else {
                    format!("{} {}", tag, format_joined(values, " "))
                }
            }
        }
    }
}
