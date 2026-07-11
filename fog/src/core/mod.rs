use std::rc::Rc;

use crate::interpreter;
use crate::interpreter::value::Value;
use crate::runtime_error;
use crate::static_check;
use crate::static_check::r#type::Type;

// TODO: refactor to consts

struct BuiltInVariable {
    name: String,
    r#type: Type,
    value: Value,
}

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

    let var_print_line = BuiltInVariable {
        name: "printLine".to_string(),
        r#type: Type::function(Type::String, Type::IOUnit),
        value: Value::NativeFunction {
            function: Rc::new(|str: Value| match str {
                Value::String(str) => Ok({
                    println!("{str}");
                    Value::IOUnitUnit
                }),
                _ => Err(runtime_error!(None, "argument is not a String")),
            }),
        },
    };

    // HACK this will be implemented in prelude
    let var_to_string = BuiltInVariable {
        name: "toString".to_string(),
        r#type: Type::function(Type::Int32, Type::String),
        value: Value::NativeFunction {
            function: Rc::new(|val: Value| match val {
                Value::Int32(val) => Ok(Value::String(format!("{val}"))),
                _ => Err(runtime_error!(None, "argument is not a String")),
            }),
        },
    };

    vec![
        var_add_int32,
        var_subtract_int32,
        var_print_line,
        var_to_string,
    ]
}

pub fn get_static_check_types() -> Vec<static_check::variable::TypeVariable> {
    let type_int32 = static_check::variable::TypeVariable {
        name: "Int32".to_string(),
        r#type: Some(Type::Int32),
        kind: static_check::kind::Kind::Type,
    };

    let type_unit = static_check::variable::TypeVariable {
        name: "Unit".to_string(),
        r#type: Some(Type::Product(Vec::new())),
        kind: static_check::kind::Kind::Type,
    };

    vec![type_int32, type_unit]
}

pub fn get_static_check_variables() -> Vec<static_check::variable::ValueVariable> {
    get_builtin_variables()
        .iter()
        .map(|var| static_check::variable::ValueVariable {
            name: var.name.clone(),
            r#type: var.r#type.clone(),
            declared: true,
        })
        .collect()
}

pub fn get_interpreter_variables() -> Vec<interpreter::variable::ValueVariable> {
    get_builtin_variables()
        .into_iter()
        .map(|var| interpreter::variable::ValueVariable::new(&var.name, var.value))
        .collect()
}
