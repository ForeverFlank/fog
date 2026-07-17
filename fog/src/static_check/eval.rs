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
    params: &Vec<String>,
    expr: &CoreTypeExpr,
    env: &Environment,
) -> FogResult<(Type, Vec<DataConstructor>)> {
    let (r#type, ctors) = match expr {
        CoreTypeExpr::Atomic(expr) => (eval_atomic_type_expr(expr, env)?, vec![]),

        CoreTypeExpr::Sum { ctors, .. } => {
            let ctors = ctors
                .iter()
                .map(|ctor| eval_data_constructor(ctor))
                .collect::<Result<Vec<_>, _>>()?;

            (Type::Named(name.to_string(), vec![]), ctors)
        }
    };

    if params.is_empty() {
        Ok((r#type, ctors))
    } else {
        let mut r#type = r#type;

        for param in params {
            r#type = Type::TypeConstructor(param.to_string(), r#type.into())
        }

        Ok((r#type, ctors))
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

        CoreAtomicTypeExpr::FunctionAppl { callee, arg, span } => {
            let callee_type = eval_atomic_type_expr(callee, env)?;

            let Type::TypeConstructor(param, r#type) = callee_type else {
                return Err(static_check_error!(
                    Some(*span),
                    "`{}` is not a valid type constructor",
                    callee_type.to_string()
                ));
            };

            let arg_type = eval_atomic_type_expr(arg, env)?;
            let res_type = (*r#type).substitute_var(&param, &arg_type);

            Ok(res_type)

            // let (head, args) = expr.uncurry();

            // let CoreAtomicTypeExpr::Identifier { name, .. } = head else {
            //     return Err(static_check_error!(
            //         Some(span),
            //         "cannot type annotate a value with data constructor `{}`",
            //         expr.to_string()
            //     ));
            // };

            // match (name.as_str(), args.as_slice()) {
            //     _ if env.contains_type(name) => apply_type_function(name, &args, env, &span),

            //     _ => Err(static_check_error!(
            //         Some(span),
            //         "cannot type annotate a value with data constructor `{}`",
            //         expr.to_string()
            //     )),
            // }
        }
    }
}

pub fn apply_type_function(
    fn_name: &str,
    arg_exprs: &Vec<&CoreAtomicTypeExpr>,
    env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    let mut current = env.get_type(fn_name, span)?;

    for &arg_expr in arg_exprs {
        let Type::TypeConstructor(param, r#type) = current else {
            return Err(static_check_error!(
                Some(*span),
                "`{}` is not a valid type constructor",
                current.to_string()
            ));
        };

        let arg = eval_atomic_type_expr(arg_expr, env)?;

        current = (*r#type).substitute_var(&param, &arg);
    }

    Ok(current)
}

// --- tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Pos;

    const SPAN: Span = Span {
        start: Pos {
            line: 0usize,
            column: 0usize,
        },
        end: Pos {
            line: 0usize,
            column: 0usize,
        },
    };

    #[test]
    fn test_eval_kind_expr() {
        let expr_1 = CoreKindExpr::Type { span: SPAN };

        let expr_2 = CoreKindExpr::Constraint { span: SPAN };

        let expr_3 = CoreKindExpr::Function {
            param_kind: CoreKindExpr::Type { span: SPAN }.into(),
            return_kind: CoreKindExpr::Type { span: SPAN }.into(),
            span: SPAN,
        };

        assert!(matches!(eval_kind_expr(&expr_1), Ok(Kind::Type)));
        assert!(matches!(eval_kind_expr(&expr_2), Ok(Kind::Constraint)));
        assert!(matches!(
            eval_kind_expr(&expr_3),
            Ok(Kind::Function(p, r)) if *p == Kind::Type && *r == Kind::Type
        ));
    }
}
