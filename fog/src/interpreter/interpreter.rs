use std::rc::Rc;

use crate::anf::anf::ANFStatement;
use crate::error::FogResult;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval_value::eval_scope;
use crate::interpreter::value::Value;
use crate::interpreter::variable::ValueVariable;
use crate::runtime_error;

fn create_top_env() -> Environment<'static> {
    let mut env = Environment::new(None);

    let var_add_int32 = ValueVariable::with_value(
        "addInt32",
        Value::NativeFunction {
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
    );

    let var_subtract_int32 = ValueVariable::with_value(
        "subtractInt32",
        Value::NativeFunction {
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
    );

    vec![var_add_int32, var_subtract_int32]
        .iter()
        .for_each(|var| {
            env.variables.insert(var.name.clone(), var.clone());
        });

    env
}

pub fn interpret(anfs: &Vec<ANFStatement>) -> FogResult<()> {
    // Top-level expressions are not allowed.
    for anf in anfs {
        if let ANFStatement::Value(_) = anf {
            return Err(runtime_error!(
                // Some(*span), // TODO span
                None,
                "cannot have final operand as a top-level statement"
            ));
        }
    }

    let mut top_env = create_top_env();
    eval_scope(anfs, &mut top_env)?;

    let mut all_vars: Vec<ValueVariable> = top_env.variables.values().cloned().collect();
    all_vars.sort_by(|a, b| a.name.cmp(&b.name));

    println!();
    for var in all_vars {
        println!(
            "{} = {}",
            var.name,
            match &*var.value.borrow() {
                Some(value) => value.to_string(),
                None => "[undefined]".to_string(),
            }
        );
    }
    println!();

    Ok(())
}
