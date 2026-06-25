use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::r#type::DataConstructor;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;
use crate::parser::parsed_expr::ParsedExpr;

pub fn eval_expr(expr: &ParsedExpr, env: &Environment, span: &Span) -> FogResult<Value> {
    match expr {
        // variable
        ParsedExpr::Identifier(name) => env.get_var(&name)?.value.ok_or_else(|| {
            FogError::runtime(
                format!("undeclared variable `{}`", name),
                Some(span.clone()),
            )
        }),

        // literals
        ParsedExpr::Int32Literal(value) => Ok(Value::Int32(*value)),
        ParsedExpr::Float32Literal(value) => Ok(Value::Float32(*value)),

        // AST lambda -> interpreter function
        ParsedExpr::Lambda {
            param_name: param,
            param_type,
            body,
        } => Ok(Value::Function {
            param: param.clone(),
            param_type: eval_type_expr(param_type, env)?,
            body: Rc::clone(body),
            captured_env: Box::new(env.clone()),
        }),

        // function application
        ParsedExpr::FuncAppl(fn_name, args) => {
            let mut result: Value = eval_expr(&ParsedExpr::Identifier(fn_name.clone()), env, span)?;
            for arg in args {
                let argument: Value = eval_expr(arg, env, span)?;
                result = apply_function(result, argument, span)?;
            }
            Ok(result)
        }

        // tuple
        ParsedExpr::Tuple(exprs) => Ok(Value::Tuple(
            exprs
                .iter()
                .map(|expr| Ok(eval_expr(expr, env, span)?.into()))
                .collect::<Result<Vec<Value>, FogError>>()?,
        )),

        // etc.
        ParsedExpr::DataConstructor { .. } => Err(FogError::runtime(
            "cannot evaluate data constructor as value".to_string(),
            Some(span.clone()),
        )),
        ParsedExpr::Collection(_) => Err(FogError::runtime(
            "unresolved name collection".to_string(),
            Some(span.clone()),
        )),
    }
}

pub fn eval_type_expr(expr: &ParsedExpr, env: &Environment) -> FogResult<Type> {
    match expr {
        ParsedExpr::Identifier(name) => Ok(env.get_type(name)?),

        ParsedExpr::FuncAppl(fn_name, args) if fn_name == "->" && args.len() == 2 => {
            let left: Type = eval_type_expr(&args[0], env)?;
            let right: Type = eval_type_expr(&args[1], env)?;

            Ok(Type::Function(Box::new(left), Box::new(right)))
        }

        ParsedExpr::FuncAppl(fn_name, args) if fn_name == "*" && args.len() == 2 => {
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

        ParsedExpr::FuncAppl(fn_name, args) if fn_name == "+" && args.len() == 2 => {
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

        ParsedExpr::FuncAppl(fn_name, args) => {
            let fn_type: Type = eval_type_expr(&ParsedExpr::Identifier(fn_name.clone()), env)?;
            let mut current_type: Type = fn_type;

            for arg in args {
                let Type::Function(param_type, return_type) = current_type else {
                    return Err(FogError::runtime(
                        format!(
                            "`{}` is not a valid type constructor",
                            current_type.to_string()
                        ),
                        None,
                    ));
                };

                let arg_type: Type = eval_type_expr(&arg, env)?;

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

                current_type = *return_type
            }

            Ok(current_type)
        }

        ParsedExpr::DataConstructor(name, args) => {
            let ctor: DataConstructor = DataConstructor {
                variant: name.clone(),
                types: args
                    .iter()
                    .map(|arg: &ParsedExpr| Ok(eval_type_expr(arg, env)?.into()))
                    .collect::<Result<Vec<Type>, FogError>>()?,
            };

            Ok(Type::Sum(vec![ctor]))
        }

        ParsedExpr::Collection(_) => Err(FogError::runtime(
            "unresolved name collection".to_string(),
            None,
        )),

        _ => Err(FogError::runtime(
            format!("`{}` is not a type", expr.to_string()),
            None,
        )),
    }
}

fn apply_function(function: Value, argument: Value, span: &Span) -> FogResult<Value> {
    match function {
        Value::Function {
            param,
            param_type,
            body,
            captured_env,
        } => {
            let mut child_env: Environment = Environment::new(Some(captured_env));

            child_env.variables.insert(
                param.clone(),
                Variable {
                    name: param.clone(),
                    value: Some(argument),
                    r#type: param_type,
                },
            );

            eval_expr(&body, &child_env, span)
        }

        Value::NativeFunction { function, .. } => function(argument),

        _ => Err(FogError::runtime(
            "cannot apply a non-function value".to_string(),
            Some(span.clone()),
        )),
    }
}
