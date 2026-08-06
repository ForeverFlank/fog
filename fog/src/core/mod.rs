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
    let var_add_int32 = BuiltInVariable {
        name: "addInt32".to_string(),
        r#type: Type::mono(Monotype::function(
            Monotype::Int32,
            Monotype::function(Monotype::Int32, Monotype::Int32),
        )),
        value: Value::NativeFunction(Rc::new(|a: Value| match a {
            Value::Int32(lhs) => Ok(Value::NativeFunction(Rc::new(move |b: Value| match b {
                Value::Int32(rhs) => Ok(Value::Int32(lhs + rhs)),
                _ => Err(runtime_error!(
                    None,
                    "`addInt32`: right operand is not an Int32"
                )),
            }))),
            _ => Err(runtime_error!(
                None,
                "`addInt32`: left operand is not an Int32"
            )),
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
                _ => Err(runtime_error!(
                    None,
                    "`subtractInt32`: right operand is not an Int32"
                )),
            }))),
            _ => Err(runtime_error!(
                None,
                "`subtractInt32`: left operand is not an Int32"
            )),
        })),
    };

    // HACK this will be implemented in prelude
    let var_to_string = BuiltInVariable {
        name: "toString".to_string(),
        r#type: Type::mono(Monotype::function(Monotype::Int32, Monotype::String)),
        value: Value::NativeFunction(Rc::new(|val: Value| match val {
            Value::Int32(val) => Ok(Value::String(format!("{val}"))),
            _ => Err(runtime_error!(
                None,
                "`toString`: argument cannot be converted to String"
            )),
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
                _ => Err(runtime_error!(
                    None,
                    "`concatString`: right operand is not a String"
                )),
            }))),
            _ => Err(runtime_error!(
                None,
                "`concatString`: left operand is not a String"
            )),
        })),
    };

    // --- IO ---

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
            Value::IO(io) => Ok(Value::IO(Rc::new(move || match io()? {
                Value::IO(inner) => inner(),
                _ => Err(runtime_error!(
                    None,
                    "`join`: argument did not produce an IO"
                )),
            }))),

            _ => Err(runtime_error!(None, "`join`: argument is not an IO")),
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
        value: Value::NativeFunction(Rc::new(|func: Value| match func {
            Value::Function { .. } | Value::NativeFunction(_) => Ok(Value::NativeFunction(
                Rc::new(move |action: Value| match action {
                    Value::IO(io) => Ok(Value::IO(Rc::new({
                        let func = func.clone();
                        move || {
                            let value = io()?;
                            eval_function_appl(func.clone(), value)
                        }
                    }))),

                    _ => Err(runtime_error!(None, "`fmap`: argument is not an IO")),
                }),
            )),

            _ => Err(runtime_error!(None, "`fmap`: argument is not a function")),
        })),
    };

    let var_bind = BuiltInVariable {
        name: "bind".to_string(),
        // IO a -> (a -> IO b) -> IO b
        r#type: Type::poly(
            vec!["a".to_string(), "b".to_string()],
            Monotype::function(
                Monotype::IO(Monotype::Variable("a".to_string()).into()),
                Monotype::function(
                    Monotype::function(
                        Monotype::Variable("a".to_string()),
                        Monotype::IO(Monotype::Variable("b".to_string()).into()),
                    ),
                    Monotype::IO(Monotype::Variable("b".to_string()).into()),
                ),
            ),
        ),
        value: Value::NativeFunction(Rc::new(|action: Value| match action {
            Value::IO(io) => Ok(Value::NativeFunction(Rc::new(
                move |func: Value| match func {
                    Value::Function { .. } | Value::NativeFunction(_) => Ok(Value::IO(Rc::new({
                        let io = io.clone();
                        let func = func.clone();

                        move || {
                            let value = io()?;
                            let next = eval_function_appl(func.clone(), value)?;

                            match next {
                                Value::IO(next_io) => next_io(),
                                _ => {
                                    Err(runtime_error!(None, "`bind`: function must return an IO"))
                                }
                            }
                        }
                    }))),

                    _ => Err(runtime_error!(
                        None,
                        "`bind`: second argument is not a function"
                    )),
                },
            ))),

            _ => Err(runtime_error!(None, "`bind`: first argument is not an IO")),
        })),
    };

    let var_then = BuiltInVariable {
        name: "then".to_string(),
        // IO a -> IO b -> IO b
        r#type: Type::poly(
            vec!["a".to_string(), "b".to_string()],
            Monotype::function(
                Monotype::IO(Monotype::Variable("a".to_string()).into()),
                Monotype::function(
                    Monotype::IO(Monotype::Variable("b".to_string()).into()),
                    Monotype::IO(Monotype::Variable("b".to_string()).into()),
                ),
            ),
        ),
        value: Value::NativeFunction(Rc::new(|io_a: Value| match io_a {
            Value::IO(io_a) => Ok(Value::NativeFunction(Rc::new(
                move |io_b: Value| match io_b {
                    Value::IO(io_b) => {
                        let io_a = io_a.clone();

                        Ok(Value::IO(Rc::new(move || {
                            io_a()?;
                            io_b()
                        })))
                    }

                    _ => Err(runtime_error!(
                        None,
                        "`then`: second argument `{}` is not an IO",
                        io_b
                    )),
                },
            ))),

            _ => Err(runtime_error!(
                None,
                "`then`: first argument `{}` is not an IO",
                io_a
            )),
        })),
    };

    let var_print = BuiltInVariable {
        name: "print".to_string(),
        // String -> IO Unit
        r#type: Type::mono(Monotype::function(
            Monotype::String,
            Monotype::IO(Monotype::Product(vec![]).into()),
        )),
        value: Value::NativeFunction(Rc::new(|value: Value| match value {
            Value::String(str) => Ok(Value::IO(Rc::new(move || {
                print!("{}", str);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                Ok(Value::Tuple(vec![]))
            }))),
            _ => Err(runtime_error!(None, "`print`: argument is not a String")),
        })),
    };

    let var_print_line = BuiltInVariable {
        name: "printLine".to_string(),
        // String -> IO Unit
        r#type: Type::mono(Monotype::function(
            Monotype::String,
            Monotype::IO(Monotype::Product(vec![]).into()),
        )),
        value: Value::NativeFunction(Rc::new(|value: Value| match value {
            Value::String(str) => Ok(Value::IO(Rc::new(move || {
                println!("{}", str);

                Ok(Value::Tuple(vec![]))
            }))),
            _ => Err(runtime_error!(
                None,
                "`printLine`: argument is not a String"
            )),
        })),
    };

    let var_read_line = BuiltInVariable {
        name: "readLine".to_string(),
        // IO String
        r#type: Type::mono(Monotype::IO(Monotype::String.into())),
        value: Value::IO(Rc::new(|| {
            let mut line = String::new();

            std::io::stdin().read_line(&mut line).unwrap();

            if let Some('\n') = line.chars().next_back() {
                line.pop();
            }
            if let Some('\r') = line.chars().next_back() {
                line.pop();
            }

            Ok(Value::String(line))
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
        var_fmap,
        var_bind,
        var_then,
        var_print,
        var_print_line,
        var_read_line,
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
