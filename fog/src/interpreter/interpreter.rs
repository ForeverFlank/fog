use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval_value::annotate_type;
use crate::interpreter::eval_value::declare;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;
use crate::parser::resolved_expr::ResolvedStatement;

fn create_top_env() -> Environment<'static> {
    let mut env = Environment::new(None);

    // the Type itself
    let t_type = Type::Type;

    // primitive types
    let t_int32 = Type::Int32;
    let t_float32 = Type::Float32;
    let t_unit = Type::Product(Vec::new());

    // function type
    let t_function = Type::Function(
        Box::new(Type::Type),
        Box::new(Type::Function(Box::new(Type::Type), Box::new(Type::Type))),
    );

    // builtin functions
    let var_plus_int32_int32 = Variable {
        name: "_plus_Int32_Int32".to_string(),
        value: Some(Value::NativeFunction {
            param_type: Type::Int32,
            return_type: Type::Function(Box::new(Type::Int32), Box::new(Type::Int32)),
            function: Rc::new(|a: Value| match a {
                Value::Int32(lhs) => Ok(Value::NativeFunction {
                    param_type: Type::Int32,
                    return_type: Type::Int32,
                    function: Rc::new(move |b: Value| match b {
                        Value::Int32(rhs) => Ok(Value::Int32(lhs + rhs)),
                        _ => Err(FogError::runtime(
                            "right operand is not an Int32".to_string(),
                            None,
                        )),
                    }),
                }),
                _ => Err(FogError::runtime(
                    "left operand is not an Int32".to_string(),
                    None,
                )),
            }),
        }),
        r#type: Type::function(Type::Int32, Type::function(Type::Int32, Type::Int32)),
    };

    vec![t_type, t_int32, t_float32, t_unit, t_function]
        .iter()
        .for_each(|r#type| {
            env.types.insert(r#type.to_string(), r#type.clone());
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
                annotate_type(name, expr, &mut top_env, span)?;
            }
            ResolvedStatement::Declaration { name, expr, span } => {
                declare(name, expr, &mut top_env, span)?;
            }
            ResolvedStatement::Expression { span, .. } => {
                return Err(FogError::runtime(
                    "cannot have final operand as a top-level statement".to_string(),
                    Some(span.clone()),
                ));
            }
        }
    }

    let mut all_vars: Vec<Variable> = top_env.variables.values().cloned().collect();
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
