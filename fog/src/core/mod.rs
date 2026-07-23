use std::rc::Rc;

use crate::interpreter;
use crate::interpreter::eval_value::eval_function_appl;
use crate::interpreter::value::Value;
use crate::runtime_error;
use crate::static_check;
use crate::static_check::r#type::Monotype;
use crate::static_check::r#type::Type;

// TODO: refactor to consts

struct BuiltInVariable {
    name: String,
    r#type: Type,
    value: Value,
}

fn get_builtin_variables() -> Vec<BuiltInVariable> {
    let unit_type = Monotype::Product(vec![]);

    let var_add_int32 = BuiltInVariable {
        name: "addInt32".to_string(),
        r#type: Type::mono(Monotype::function(
            Monotype::Int32,
            Monotype::function(Monotype::Int32, Monotype::Int32),
        )),
        value: Value::NativeFunction(Rc::new(|a: Value| match a {
            Value::Int32(lhs) => Ok(Value::NativeFunction(Rc::new(move |b: Value| match b {
                Value::Int32(rhs) => Ok(Value::Int32(lhs + rhs)),
                _ => Err(runtime_error!(None, "right operand is not an Int32")),
            }))),
            _ => Err(runtime_error!(None, "left operand is not an Int32")),
        })),
    };

    let var_subtract_int32 = BuiltInVariable {
        name: "subtractInt32".to_string(),
        r#type: Type::mono(Monotype::function(
            Monotype::Int32,
            Monotype::function(Monotype::Int32, Monotype::Int32),
        )),
        value: Value::NativeFunction(Rc::new(|a: Value| match a {
            Value::Int32(lhs) => Ok(Value::NativeFunction(Rc::new(move |b: Value| match b {
                Value::Int32(rhs) => Ok(Value::Int32(lhs - rhs)),
                _ => Err(runtime_error!(None, "right operand is not an Int32")),
            }))),
            _ => Err(runtime_error!(None, "left operand is not an Int32")),
        })),
    };

    // HACK this will be implemented in prelude
    let var_to_string = BuiltInVariable {
        name: "toString".to_string(),
        r#type: Type::mono(Monotype::function(Monotype::Int32, Monotype::String)),
        value: Value::NativeFunction(Rc::new(|val: Value| match val {
            Value::Int32(val) => Ok(Value::String(format!("{val}"))),
            _ => Err(runtime_error!(None, "argument is not a String")),
        })),
    };

    // HACK this will be implemented in prelude
    let var_concat_string = BuiltInVariable {
        name: "concatString".to_string(),
        r#type: Type::mono(Monotype::function(
            Monotype::String,
            Monotype::function(Monotype::String, Monotype::String),
        )),
        value: Value::NativeFunction(Rc::new(|a: Value| match a {
            Value::String(lhs) => Ok(Value::NativeFunction(Rc::new(move |b: Value| match b {
                Value::String(rhs) => Ok(Value::String(lhs.clone() + &rhs)),
                _ => Err(runtime_error!(None, "right operand is not a String")),
            }))),
            _ => Err(runtime_error!(None, "left operand is not a String")),
        })),
    };

    // --- IO ---

    // impure IO run function
    // IO a -> a
    let unsafe_run_io = Value::NativeFunction(Rc::new(|str: Value| match str {
        Value::IO(io) => io(),
        _ => Err(runtime_error!(None, "argument is not an IO")),
    }));

    let var_return = BuiltInVariable {
        name: "return".to_string(),
        // a -> IO a
        r#type: Type::poly(
            vec!["a".to_string()],
            Monotype::function(
                Monotype::Variable("a".to_string()),
                Monotype::IO(Monotype::Variable("a".to_string()).into()),
            ),
        ),
        value: Value::NativeFunction(Rc::new(|value: Value| {
            Ok(Value::IO(Rc::new(move || Ok(value.clone()))))
        })),
    };

    let var_join = BuiltInVariable {
        name: "join".to_string(),
        // IO (IO a) -> IO a
        r#type: Type::poly(
            vec!["a".to_string()],
            Monotype::function(
                Monotype::IO(Monotype::IO(Monotype::Variable("a".to_string()).into()).into()),
                Monotype::IO(Monotype::Variable("a".to_string()).into()),
            ),
        ),
        value: Value::NativeFunction(Rc::new(|value: Value| match value {
            Value::IO(io) => match io() {
                Ok(Value::IO(io)) => io(),
                _ => Err(runtime_error!(None, "argument is not an IO")),
            },
            _ => Err(runtime_error!(None, "argument is not an IO")),
        })),
    };

    let var_fmap = BuiltInVariable {
        name: "fmap".to_string(),
        // (a -> b) -> IO a -> IO b
        r#type: Type::poly(
            vec!["a".to_string(), "b".to_string()],
            Monotype::function(
                Monotype::function(
                    Monotype::Variable("a".to_string()),
                    Monotype::Variable("b".to_string()),
                ),
                Monotype::function(
                    Monotype::IO(Monotype::Variable("a".to_string()).into()),
                    Monotype::IO(Monotype::Variable("b".to_string()).into()),
                ),
            ),
        ),
        value: Value::NativeFunction(Rc::new(move |func: Value| match func {
            Value::Function { .. } => {
                let unsafe_run_io = unsafe_run_io.clone();

                Ok(Value::NativeFunction(Rc::new(
                    move |action: Value| match action {
                        Value::IO(_) => Ok(eval_function_appl(
                            func.clone(),
                            eval_function_appl(unsafe_run_io.clone(), action)?,
                        )?),

                        _ => Err(runtime_error!(None, "argument is not an IO")),
                    },
                )))
            }

            _ => Err(runtime_error!(None, "argument is not a function")),
        })),
    };

    let var_print_line = BuiltInVariable {
        name: "printLine".to_string(),
        // String -> IO Unit
        r#type: Type::mono(Monotype::function(
            Monotype::String,
            Monotype::IO(unit_type.clone().into()),
        )),
        value: Value::NativeFunction(Rc::new(|value: Value| match value {
            Value::String(str) => Ok(Value::IO(Rc::new(move || {
                println!("{str}");
                Ok(Value::Tuple(vec![]))
            }))),
            _ => Err(runtime_error!(None, "argument is not a String")),
        })),
    };

    vec![
        var_add_int32,
        var_subtract_int32,
        var_to_string,
        var_concat_string,
        // IO
        var_return,
        var_join,
        var_print_line,
    ]
}

pub fn get_static_check_types() -> Vec<static_check::variable::TypeVariable> {
    let type_int32 = static_check::variable::TypeVariable {
        name: "Int32".to_string(),
        r#type: Some(Monotype::Int32),
        kind: static_check::kind::Kind::Type,
    };

    let type_string = static_check::variable::TypeVariable {
        name: "String".to_string(),
        r#type: Some(Monotype::String),
        kind: static_check::kind::Kind::Type,
    };

    let type_unit = static_check::variable::TypeVariable {
        name: "Unit".to_string(),
        r#type: Some(Monotype::Product(Vec::new())),
        kind: static_check::kind::Kind::Type,
    };

    let type_io = static_check::variable::TypeVariable {
        name: "IO".to_string(),
        r#type: Some(Monotype::TypeConstructor(
            "a".to_string(),
            Monotype::IO(Monotype::Variable("a".to_string()).into()).into(),
        )),
        kind: static_check::kind::Kind::Type,
    };

    vec![type_int32, type_string, type_unit, type_io]
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
