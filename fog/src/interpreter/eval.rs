use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::r#type::DataConstructor;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;
use crate::parser::resolved_expr::ResolvedExpr;

pub enum EvalContext {
    TypeAnnotation,
    Declaration,
}

pub fn eval_value_expr(expr: &ResolvedExpr, env: &Environment, span: &Span) -> FogResult<Value> {
    match expr {
        // variable
        ResolvedExpr::Identifier(name) => env.get_var(&name)?.value.ok_or_else(|| {
            FogError::runtime(
                format!("undeclared variable `{}`", name),
                Some(span.clone()),
            )
        }),

        // literals
        ResolvedExpr::Int32Literal(value) => Ok(Value::Int32(*value)),
        ResolvedExpr::Float32Literal(value) => Ok(Value::Float32(*value)),

        // AST lambda -> interpreter function
        ResolvedExpr::Lambda(param_name, param_type, body) => Ok(Value::Function {
            param_name: param_name.clone(),
            param_type: eval_type_expr(param_type, env)?,
            body: Rc::clone(body),
            captured_env: Box::new(env.clone()),
        }),

        // tuple
        ResolvedExpr::Tuple(exprs) => Ok(Value::Tuple(
            exprs
                .iter()
                .map(|expr| Ok(eval_value_expr(expr, env, span)?.into()))
                .collect::<Result<Vec<Value>, FogError>>()?,
        )),

        // function application
        ResolvedExpr::FuncAppl(fn_name, args) => {
            let mut result: Value =
                eval_value_expr(&ResolvedExpr::Identifier(fn_name.clone()), env, span)?;

            for arg in args {
                let argument: Value = eval_value_expr(arg, env, span)?;
                result = apply_value_function(result, argument, span)?;
            }

            Ok(result)
        }
    }
}

fn apply_value_function(function: Value, argument: Value, span: &Span) -> FogResult<Value> {
    match function {
        Value::Function {
            param_name,
            param_type,
            body,
            captured_env,
        } => {
            let mut child_env: Environment = Environment::new(Some(captured_env));

            child_env.variables.insert(
                param_name.clone(),
                Variable {
                    name: param_name.clone(),
                    value: Some(argument),
                    r#type: param_type,
                },
            );

            eval_value_expr(&body, &child_env, span)
        }

        Value::NativeFunction { function, .. } => function(argument),

        _ => Err(FogError::runtime(
            "cannot apply a non-function value".to_string(),
            Some(span.clone()),
        )),
    }
}

// --- type expressions ---

pub fn eval_type_expr(expr: &ResolvedExpr, env: &Environment) -> FogResult<Type> {
    match expr {
        ResolvedExpr::Identifier(name) => Ok(env.get_type(name)?),

        // function type
        ResolvedExpr::FuncAppl(fn_name, args) if fn_name == "->" && args.len() == 2 => {
            let left: Type = eval_type_expr(&args[0], env)?;
            let right: Type = eval_type_expr(&args[1], env)?;

            Ok(Type::Function(Box::new(left), Box::new(right)))
        }

        // product type, a.k.a. tuple
        ResolvedExpr::FuncAppl(fn_name, args) if fn_name == "*" && args.len() == 2 => {
            let left: Type = eval_type_expr(&args[0], env)?;
            let right: Type = eval_type_expr(&args[1], env)?;

            let mut types: Vec<Type> = Vec::new();

            match left {
                Type::Product(ts) => types.extend(ts),
                t => types.push(t),
            }

            match right {
                Type::Product(ts) => types.extend(ts),
                t => types.push(t),
            }

            Ok(Type::Product(types))
        }

        // sum type
        ResolvedExpr::FuncAppl(fn_name, args) if fn_name == "+" && args.len() == 2 => {
            let left: Type = eval_type_expr(&args[0], env)?;
            let right: Type = eval_type_expr(&args[1], env)?;

            let Type::Sum(ctors1) = left else {
                return Err(FogError::runtime(
                    format!(
                        "`{}` is not a data constructor or a sum type",
                        left.to_string()
                    ),
                    None,
                ));
            };
            let Type::Sum(ctors2) = right else {
                return Err(FogError::runtime(
                    format!(
                        "`{}` is not a data constructor or a sum type",
                        right.to_string()
                    ),
                    None,
                ));
            };

            let concatenated: Vec<DataConstructor> = [&ctors1[..], &ctors2[..]].concat();

            Ok(Type::Sum(concatenated))
        }

        // either a function application or a data constructor
        ResolvedExpr::FuncAppl(name, args) => {
            if env.contains_type(name) {
                // the function name is declared, it's a type constructor
                apply_type_constructor(name, args, env)
            } else {
                // otherwise, it's a data constructor
                declare_data_constructor(name, args, env)
            }
        }

        _ => Err(FogError::runtime(
            format!("`{}` is not a type", expr.to_string()),
            None,
        )),
    }
}

fn apply_type_constructor(
    fn_name: &str,
    args: &Vec<ResolvedExpr>,
    env: &Environment,
) -> FogResult<Type> {
    let mut current: Type = env.get_type(fn_name)?;

    for arg in args {
        let Type::Function(param_type, return_type) = current else {
            return Err(FogError::runtime(
                format!("`{}` is not a valid type constructor", current.to_string()),
                None,
            ));
        };

        let arg_type: Type = eval_type_expr(arg, env)?;

        if arg_type != *param_type {
            return Err(FogError::runtime(
                format!(
                    "type mismatch applying `{}`\n\
                     expected `{}`, found `{}`",
                    fn_name,
                    param_type.to_string(),
                    arg_type.to_string()
                ),
                None,
            ));
        }

        current = *return_type;
    }

    Ok(current)
}

fn declare_data_constructor(
    ctor_name: &str,
    args: &Vec<ResolvedExpr>,
    env: &Environment,
) -> FogResult<Type> {
    let ctor: DataConstructor = DataConstructor {
        tag: ctor_name.to_string(),
        types: args
            .iter()
            .map(|arg: &ResolvedExpr| Ok(eval_type_expr(arg, env)?.into()))
            .collect::<Result<Vec<Type>, FogError>>()?,
    };

    env.declare_value(&ctor_name.to_string(), ..., ...)?;

    Ok(Type::Sum(vec![ctor]))
}
