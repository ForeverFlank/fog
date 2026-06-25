use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;
use crate::parser::nodes::Expr;

pub fn eval_expr(expr: &Expr, env: &Environment, span: &Span) -> FogResult<Value> {
    match expr {
        // variable
        Expr::Identifier(name) => env.get_var(&name)?.value.ok_or_else(|| {
            FogError::runtime(
                format!("undeclared variable `{}`", name),
                Some(span.clone()),
            )
        }),

        // literals
        Expr::Int32Literal(value) => Ok(Value::Int32(*value)),
        Expr::Float32Literal(value) => Ok(Value::Float32(*value)),

        // AST lambda -> interpreter function
        Expr::Lambda {
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
        Expr::FuncAppl {
            fn_name: function_name,
            args: arguments,
        } => {
            let mut result: Value = eval_expr(&Expr::Identifier(function_name.clone()), env, span)?;
            for arg in arguments {
                let argument: Value = eval_expr(arg, env, span)?;
                result = apply_function(result, argument, span)?;
            }
            Ok(result)
        }

        Expr::DataConstructor { .. } => Err(FogError::runtime(
            "cannot evaluate data constructor as value".to_string(),
            Some(span.clone()),
        )),

        Expr::NameCollection(_) => Err(FogError::runtime(
            "unresolved name collection".to_string(),
            Some(span.clone()),
        )),
    }
}

pub fn eval_type_expr(expr: &Expr, env: &Environment) -> FogResult<Type> {
    match expr {
        Expr::Identifier(name) => Ok(env.get_type(name)?),

        // Expr::FuncAppl { function, args } if function == "->" && args.len() == 2 => {
        //     let left: Type = eval_type_expr(args[0].as_ref(), env)?;
        //     let right: Type = eval_type_expr(args[1].as_ref(), env)?;

        //     Ok(Type::Function(Box::new(left), Box::new(right)))
        // }
        Expr::FuncAppl {
            fn_name: function,
            args,
        } => {
            let fn_type: Type = eval_type_expr(&Expr::Identifier(function.clone()), env)?;
            let mut current: Type = fn_type;

            for arg in args {
                current = match current {
                    Type::Function(param, ret) => {
                        let arg_type: Type = eval_type_expr(arg.as_ref(), env)?;
                        if arg_type != *param {
                            return Err(FogError::runtime(
                                format!("type mismatch applying `{}`", function),
                                None,
                            ));
                        }
                        *ret
                    }
                    _ => {
                        return Err(FogError::runtime(
                            format!("`{}` is not a valid type constructor", function),
                            None,
                        ));
                    }
                };
            }

            Ok(current)
        }

        Expr::Lambda { .. } => Err(FogError::runtime("lambda is not a type".to_string(), None)),
        Expr::Int32Literal(_) => Err(FogError::runtime(
            "Int32 literal is not a type".to_string(),
            None,
        )),
        Expr::Float32Literal(_) => Err(FogError::runtime(
            "Float32 literal is not a type".to_string(),
            None,
        )),
        Expr::DataConstructor { .. } => Err(FogError::runtime(
            "data constructor is not a type".to_string(),
            None,
        )),
        Expr::NameCollection(_) => Err(FogError::runtime(
            "unresolved name collection".to_string(),
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
