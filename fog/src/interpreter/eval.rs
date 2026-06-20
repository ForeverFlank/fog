use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::nodes::Expr;
use crate::interpreter::environment::Environment;
use crate::interpreter::interpreter::InterpreterError;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;

pub fn eval_expr(expr: &Expr, env: &Environment) -> Result<Value, InterpreterError> {
    match expr {
        // variable
        Expr::Identifier(name) => env.get_var(&name)?.value.ok_or_else(|| {
            InterpreterError::from_string(format!("undeclared variable `{}`", name,))
        }),

        // literals
        Expr::Int32Literal(value) => Ok(Value::Int32(*value)),
        Expr::Float32Literal(value) => Ok(Value::Float32(*value)),

        // AST lambda -> interpreter function
        Expr::Lambda {
            param,
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
            function: function_name,
            args: arguments,
        } => {
            let mut result: Value = eval_expr(&Expr::Identifier(function_name.clone()), env)?;
            for arg in arguments {
                result = apply_function(result, eval_expr(arg, env)?)?;
            }
            Ok(result)
        }
    }
}

pub fn eval_type_expr(expr: &Expr, env: &Environment) -> Result<Type, InterpreterError> {
    match expr {
        Expr::Identifier(name) => {
            let Some(Value::Type(r#type)) = env.get_var(name)?.value else {
                return Err(InterpreterError::from_string(format!(
                    "`{}` is not a type",
                    name
                )));
            };

            Ok(r#type)
        }

        Expr::FuncAppl { function, args } if function == "->" && args.len() == 2 => {
            let left: Type = eval_type_expr(args[0].as_ref(), env)?;
            let right: Type = eval_type_expr(args[1].as_ref(), env)?;

            Ok(Type::Function(Box::new(left), Box::new(right)))
        }

        Expr::FuncAppl { function, args } => {
            let fn_type: Type = eval_type_expr(&Expr::Identifier(function.clone()), env)?;
            let mut current: Type = fn_type;

            for arg in args {
                current = match current {
                    Type::Function(param, ret) => {
                        let arg_type: Type = eval_type_expr(arg.as_ref(), env)?;
                        if arg_type != *param {
                            return Err(InterpreterError::from_string(format!(
                                "type mismatch applying `{}`",
                                function
                            )));
                        }
                        *ret
                    }
                    _ => {
                        return Err(InterpreterError::from_string(format!(
                            "`{}` is not a valid type constructor",
                            function
                        )));
                    }
                };
            }

            Ok(current)
        }

        Expr::Lambda { .. } => Err(InterpreterError::from_str("lambda is not a type")),
        Expr::Int32Literal(_) => Err(InterpreterError::from_str("Int32 literal is not a type")),
        Expr::Float32Literal(_) => Err(InterpreterError::from_str("Float32 literal is not a type")),
    }
}

fn apply_function(function: Value, argument: Value) -> Result<Value, InterpreterError> {
    match function {
        Value::Function {
            param,
            param_type,
            body,
            captured_env,
        } => {
            let mut child_env: Environment = Environment {
                variables: HashMap::new(),
                parent: Some(captured_env),
            };

            child_env.variables.insert(
                param.clone(),
                Variable {
                    name: param.clone(),
                    value: Some(argument),
                    r#type: param_type,
                },
            );

            eval_expr(&body, &child_env)
        }

        Value::NativeFunction { function, .. } => function(argument),

        _ => Err(InterpreterError::from_str(
            "cannot apply a non-function value",
        )),
    }
}
