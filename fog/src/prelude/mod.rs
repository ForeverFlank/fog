use std::rc::Rc;

use crate::interpreter::value::Value;
use crate::runtime_error;
use crate::static_check::r#type::Type;

struct BuiltInVariable {
    name: String,
    r#type: Type,
    value: Value,
}

// TODO: move stuff from static_check and interpreter to here

fn get_builtin_variables() -> Vec<BuiltInVariable> {
    let var_add_int32 = BuiltInVariable {
        name: "addInt32".to_string(),
        r#type: Type::function(Type::Int32, Type::function(Type::Int32, Type::Int32)),
        value: Value::NativeFunction {
            function: Rc::new(|a: Value| match a {
                Value::Int32(lhs) => Ok(Value::NativeFunction {
                    function: Rc::new(move |b: Value| match b {
                        Value::Int32(rhs) => Ok(Value::Int32(lhs + rhs)),
                        _ => Err(runtime_error!(None, "right operand is not an Int32")),
                    }),
                }),
                _ => Err(runtime_error!(None, "left operand is not an Int32")),
            }),
        },
    };

    let var_subtract_int32 = BuiltInVariable {
        name: "subtractInt32".to_string(),
        r#type: Type::function(Type::Int32, Type::function(Type::Int32, Type::Int32)),
        value: Value::NativeFunction {
            function: Rc::new(|a: Value| match a {
                Value::Int32(lhs) => Ok(Value::NativeFunction {
                    function: Rc::new(move |b: Value| match b {
                        Value::Int32(rhs) => Ok(Value::Int32(lhs - rhs)),
                        _ => Err(runtime_error!(None, "right operand is not an Int32")),
                    }),
                }),
                _ => Err(runtime_error!(None, "left operand is not an Int32")),
            }),
        },
    };

    vec![var_add_int32, var_subtract_int32]
}
