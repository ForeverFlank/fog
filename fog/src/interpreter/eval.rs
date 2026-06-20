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
        // literals
        Expr::Int32Literal(value) => Ok(Value::Int32(*value)),
        Expr::Float32Literal(value) => Ok(Value::Float32(*value)),

        // variable
        Expr::Identifier(name) => env.get_var(&name)?.value.ok_or_else(|| {
            InterpreterError::from_string(format!("undeclared variable `{}`", name,))
        }),

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
    match expr {}
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
