use std::vec;

use crate::error::FogResult;
use crate::error::Span;
use crate::parser::core_expr::CoreAtomicTypeExpr;
use crate::parser::core_expr::CoreDataConstructor;
use crate::parser::core_expr::CoreKindExpr;
use crate::parser::core_expr::CoreTypeExpr;
use crate::static_check::environment::Environment;
use crate::static_check::kind::Kind;
use crate::static_check::r#type::DataConstructor;
use crate::static_check::r#type::Type;
use crate::static_check_error;

// --- kind ---

pub fn eval_kind_expr(expr: &CoreKindExpr) -> FogResult<Kind> {
    match expr {
        CoreKindExpr::Type { .. } => Ok(Kind::Type),

        CoreKindExpr::Constraint { .. } => Ok(Kind::Constraint),

        CoreKindExpr::Function {
            param_kind,
            return_kind,
            ..
        } => Ok(Kind::Function(
            eval_kind_expr(&param_kind)?.into(),
            eval_kind_expr(&return_kind)?.into(),
        )),
    }
}

pub fn eval_type_expr(
    name: &str,
    expr: &CoreTypeExpr,
    env: &Environment,
) -> FogResult<(Type, Vec<DataConstructor>)> {
    match expr {
        CoreTypeExpr::Atomic(expr) => Ok((eval_atomic_type_expr(expr, env)?, vec![])),

        CoreTypeExpr::Sum { ctors, .. } => {
            let ctors = ctors
                .iter()
                .map(|ctor| eval_data_constructor(ctor))
                .collect::<Result<Vec<_>, _>>()?;

            Ok((Type::Sum(name.to_string()), ctors))
        }
    }
}

fn eval_data_constructor(ctor: &CoreDataConstructor) -> FogResult<DataConstructor> {
    Ok(DataConstructor {
        tag: ctor.tag.clone(),
        types: ctor.types.clone(),
    })
}

pub fn eval_atomic_type_expr(expr: &CoreAtomicTypeExpr, env: &Environment) -> FogResult<Type> {
    let span = expr.span();

    match expr {
        CoreAtomicTypeExpr::Identifier { name, .. } => {
            let first_ch = name.chars().next().unwrap();

            if first_ch.is_lowercase() {
                Ok(Type::Variable(name.to_string()))
            } else if let Some(r#type) = env.get_type_var(name, &span)?.r#type {
                Ok(r#type)
            } else {
                Err(static_check_error!(Some(span), "undeclared type `{name}`"))
            }
        }

        CoreAtomicTypeExpr::Function {
            param_type,
            return_type,
            ..
        } => {
            let param_type = eval_atomic_type_expr(param_type, env)?;
            let return_type = eval_atomic_type_expr(return_type, env)?;

            Ok(Type::Function(param_type.into(), return_type.into()))
        }

        CoreAtomicTypeExpr::Product { types, .. } => {
            let types = types
                .into_iter()
                .map(|t| eval_atomic_type_expr(t, env))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Type::Product(types))
        }

        CoreAtomicTypeExpr::FunctionAppl { .. } => {
            let (head, args) = expr.uncurry();

            let CoreAtomicTypeExpr::Identifier { name, .. } = head else {
                return Err(static_check_error!(
                    Some(span),
                    "cannot type annotate a value with data constructor `{}`",
                    expr.to_string()
                ));
            };

            match (name.as_str(), args.as_slice()) {
                _ if env.contains_type(name) => apply_type_level_function(name, &args, env, &span),

                _ => Err(static_check_error!(
                    Some(span),
                    "cannot type annotate a value with data constructor `{}`",
                    expr.to_string()
                )),
            }
        }

        CoreAtomicTypeExpr::ForAll {
            var_name, r#type, ..
        } => {
            let r#type = eval_atomic_type_expr(r#type, env)?;

            Ok(Type::ForAll(var_name.clone(), r#type.into()))
        }
    }
}

pub fn apply_type_level_function(
    fn_name: &str,
    args: &Vec<&CoreAtomicTypeExpr>,
    env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    let mut current = env.get_type(fn_name, span)?;

    for &arg in args {
        let Type::Function(param_type, return_type) = current else {
            return Err(static_check_error!(
                Some(*span),
                "`{}` is not a valid type constructor",
                current.to_string()
            ));
        };

        let arg_kind = eval_atomic_type_expr(arg, env)?;

        if arg_kind != *param_type {
            return Err(static_check_error!(
                Some(*span),
                "type mismatch applying `{}`\n\
                 expected `{}`, found `{}`",
                fn_name,
                param_type.to_string(),
                arg_kind.to_string()
            ));
        }

        current = *return_type;
    }

    Ok(current)
}
