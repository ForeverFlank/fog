use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval_type::eval_type_annotation_expr;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;
use crate::parser::resolved_expr::ResolvedExpr;

pub fn eval_value_expr(expr: &ResolvedExpr, env: &Environment, span: &Span) -> FogResult<Value> {
    match expr {
        // variable
        ResolvedExpr::Identifier { name } => env.get_var(name, span)?.value.ok_or_else(|| {
            FogError::runtime(
                format!("undeclared variable `{}`", name),
                Some(span.clone()),
            )
        }),

        // literals
        ResolvedExpr::Int32Literal { value } => Ok(Value::Int32(*value)),
        ResolvedExpr::Float32Literal { value } => Ok(Value::Float32(*value)),

        // AST lambda -> interpreter function
        ResolvedExpr::Lambda {
            param_name,
            param_type,
            body,
        } => Ok(Value::Function {
            param_name: param_name.clone(),
            param_type: eval_type_annotation_expr(param_type, env, span)?,
            body: Rc::clone(body),
            captured_env: Box::new(env.clone()),
        }),

        // tuple
        ResolvedExpr::Tuple { items } => Ok(Value::Tuple(
            items
                .iter()
                .map(|expr| eval_value_expr(expr, env, span))
                .collect::<Result<Vec<Value>, FogError>>()?,
        )),

        // function application
        ResolvedExpr::FuncAppl { fn_name, args } => {
            let mut result = eval_value_expr(
                &ResolvedExpr::Identifier {
                    name: fn_name.clone(),
                },
                env,
                span,
            )?;

            for arg in args {
                let argument = eval_value_expr(arg, env, span)?;
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
            let mut child_env = Environment::new(Some(captured_env));

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

        Value::NativeFunction { function, .. } => function(argument).map_err(|mut e| {
            if e.span.is_none() {
                e.span = Some(span.clone());
            }
            e
        }),

        _ => Err(FogError::runtime(
            "cannot apply a non-function value".to_string(),
            Some(span.clone()),
        )),
    }
}
