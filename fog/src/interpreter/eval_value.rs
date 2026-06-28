use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval_statement::annotate_type;
use crate::interpreter::eval_statement::declare;
use crate::interpreter::eval_type::eval_type_annotation_expr;
use crate::interpreter::value::Value;
use crate::interpreter::variable::ValueVariable;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::parser::resolved_expr::ResolvedStatement;
use crate::runtime_error;

pub fn eval_value_expr(expr: &ResolvedExpr, env: &Environment, span: &Span) -> FogResult<Value> {
    match expr {
        ResolvedExpr::Block { statements } => {
            let mut block_env = Environment::new(Some(env));

            for stmt in statements {
                match stmt {
                    ResolvedStatement::TypeAnnotation { name, expr, span } => {
                        annotate_type(name, expr, &mut block_env, span)?;
                    }
                    ResolvedStatement::Declaration { name, expr, span } => {
                        declare(name, expr, &mut block_env, span)?;
                    }
                    ResolvedStatement::Expression { span, expr } => {
                        return eval_value_expr(expr, &mut block_env, span);
                    }
                }
            }

            Err(runtime_error!(
                Some(span.clone()),
                "final operand not found in block statement"
            ))
        }

        ResolvedExpr::Identifier { name } => env
            .get_value_var(name, span)?
            .value
            .ok_or_else(|| runtime_error!(Some(span.clone()), "undeclared variable `{}`", name)),

        ResolvedExpr::Int32Literal { value } => Ok(Value::Int32(*value)),
        ResolvedExpr::Float32Literal { value } => Ok(Value::Float32(*value)),

        ResolvedExpr::Lambda {
            param_name,
            param_type,
            body,
        } => Ok(Value::Function {
            param_name: param_name.clone(),
            param_type: eval_type_annotation_expr(param_type, env, span)?,
            body: Rc::clone(body),
            captured_env: env.flatten().into(),
        }),

        ResolvedExpr::Tuple { items } => Ok(Value::Tuple(
            items
                .iter()
                .map(|expr| eval_value_expr(expr, env, span))
                .collect::<Result<Vec<Value>, FogError>>()?,
        )),

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
                result = apply_function(result, argument, span)?;
            }

            Ok(result)
        }
    }
}

fn apply_function(function: Value, argument: Value, span: &Span) -> FogResult<Value> {
    match function {
        Value::Function {
            param_name,
            param_type,
            body,
            captured_env,
        } => {
            let mut child_env = Environment::new(Some(captured_env.as_ref()));

            child_env.variables.insert(
                param_name.clone(),
                ValueVariable {
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

        _ => Err(runtime_error!(
            Some(span.clone()),
            "cannot apply a non-function value"
        )),
    }
}
