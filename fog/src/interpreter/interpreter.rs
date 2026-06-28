use std::rc::Rc;

use crate::error::FogResult;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval_statement::annotate;
use crate::interpreter::eval_statement::declare;
use crate::interpreter::kind::Kind;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::TypeVariable;
use crate::interpreter::variable::ValueVariable;
use crate::parser::resolved_expr::ResolvedStatement;
use crate::runtime_error;

fn create_top_env() -> Environment<'static> {
    let mut env = Environment::new(None);

    // // the Type itself
    // let k_type = Kind::Type;

    // // function kind
    // let k_function = Kind::Function(
    //     Kind::Type.into(),
    //     Kind::Function(Kind::Type.into(), Kind::Type.into()).into(),
    // );

    // primitive types
    let t_int32 = TypeVariable {
        name: "Int32".to_string(),
        r#type: Type::Int32.into(),
        kind: Kind::Type,
    };

    let t_float32 = TypeVariable {
        name: "Float32".to_string(),
        r#type: Type::Float32.into(),
        kind: Kind::Type,
    };

    let t_unit = TypeVariable {
        name: "Unit".to_string(),
        r#type: Type::Product(Vec::new()).into(),
        kind: Kind::Type,
    };

    // builtin functions
    let var_plus_int32_int32 = ValueVariable {
        name: "_plus_Int32_Int32".to_string(),
        value: Some(Value::NativeFunction {
            param_type: Type::Int32,
            return_type: Type::Function(Type::Int32.into(), Type::Int32.into()),
            function: Rc::new(|a: Value| match a {
                Value::Int32(lhs) => Ok(Value::NativeFunction {
                    param_type: Type::Int32,
                    return_type: Type::Int32,
                    function: Rc::new(move |b: Value| match b {
                        Value::Int32(rhs) => Ok(Value::Int32(lhs + rhs)),
                        _ => Err(runtime_error!(None, "right operand is not an Int32")),
                    }),
                }),
                _ => Err(runtime_error!(None, "left operand is not an Int32")),
            }),
        }),
        r#type: Type::function(Type::Int32, Type::function(Type::Int32, Type::Int32)),
    };

    vec![t_int32, t_float32, t_unit]
        .iter()
        .for_each(|type_var| {
            env.types.insert(type_var.name.clone(), type_var.clone());
        });

    vec![var_plus_int32_int32].iter().for_each(|var| {
        env.variables.insert(var.name.clone(), var.clone());
    });

    env
}

pub fn interpret(statements: &Vec<ResolvedStatement>) -> FogResult<()> {
    let mut top_env = create_top_env();

    for stmt in statements {
        match stmt {
            ResolvedStatement::TypeAnnotation { name, expr, span } => {
                annotate(name, expr, &mut top_env, span)?;
            }
            ResolvedStatement::Declaration { name, expr, span } => {
                declare(name, expr, &mut top_env, span)?;
            }
            ResolvedStatement::Expression { span, .. } => {
                return Err(runtime_error!(
                    Some(span.clone()),
                    "cannot have final operand as a top-level statement"
                ));
            }
        }
    }

    let mut all_vars: Vec<ValueVariable> = top_env.variables.values().cloned().collect();
    all_vars.sort_by(|a, b| a.name.cmp(&b.name));

    println!();
    for var in all_vars {
        println!(
            "{} : {} = {}",
            var.name,
            var.r#type.to_string(),
            match var.value {
                Some(value) => value.to_string(),
                None => "?".to_string(),
            }
        );
    }
    println!();

    Ok(())
}
