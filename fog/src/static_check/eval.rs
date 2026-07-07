use crate::error::FogResult;
use crate::error::Span;
use crate::parser::core_expr::CoreAtomicTypeExpr;
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

pub fn eval_atomic_type_expr(expr: &CoreAtomicTypeExpr, env: &Environment) -> FogResult<Type> {
    let span = expr.span();

    match expr {
        CoreAtomicTypeExpr::Identifier { name, .. } => env
            .get_type_var(name, &span)?
            .r#type
            .ok_or_else(|| static_check_error!(Some(span), "undeclared type `{}`", name)),

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
                ("->", &[lhs, rhs]) => eval_function_type(lhs, rhs, env),
                ("*", &[lhs, rhs]) => eval_product_type(lhs, rhs, env),

                ("+", _) => Err(static_check_error!(
                    Some(span),
                    "cannot type annotate a value with sum types"
                )),

                _ if env.contains_type(name) => apply_type_level_function(name, &args, env, &span),

                _ => Err(static_check_error!(
                    Some(span),
                    "cannot type annotate a value with data constructor `{}`",
                    expr.to_string()
                )),
            }
        }

        _ => Err(static_check_error!(
            Some(span),
            "`{}` is not a type",
            expr.to_string()
        )),
    }
}

pub fn eval_type_expr(expr: &CoreTypeExpr, env: &Environment) -> FogResult<Type> {
    let span = expr.span();

    match expr {
        CoreTypeExpr::Identifier { name, .. } if env.contains_type(name) => env
            .get_type_var(name, &span)?
            .r#type
            .ok_or_else(|| static_check_error!(Some(span), "undeclared type `{}`", name)),

        CoreTypeExpr::Identifier { name, .. } => Ok(Type::Sum(vec![DataConstructor {
            tag: name.clone(),
            types: Vec::new(),
        }])),

        CoreTypeExpr::FunctionAppl { .. } => {
            let (head, args) = expr.uncurry();

            let CoreAtomicTypeExpr::Identifier { name, .. } = head else {
                return Err(static_check_error!(
                    Some(span),
                    "`{}` is not a valid type definition",
                    expr.to_string()
                ));
            };

            match (name.as_str(), args.as_slice()) {
                ("->", &[lhs, rhs]) => eval_function_type(lhs, rhs, env),
                ("*", &[lhs, rhs]) => eval_product_type(lhs, rhs, env),
                ("+", &[lhs, rhs]) => eval_sum_type(lhs, rhs, env),

                _ if env.contains_type(name) => apply_type_level_function(name, &args, env, &span),

                // data constructor
                _ => {
                    let field_types = args
                        .iter()
                        .map(|&arg| eval_atomic_type_expr(arg, env))
                        .collect::<Result<Vec<Type>, _>>()?;

                    Ok(Type::Sum(vec![DataConstructor {
                        tag: name.clone(),
                        types: field_types,
                    }]))
                }
            }
        }

        _ => Err(static_check_error!(
            Some(span),
            "`{}` is not a valid type definition",
            expr.to_string()
        )),
    }
}

fn eval_product_type(
    left: &CoreAtomicTypeExpr,
    right: &CoreAtomicTypeExpr,
    env: &Environment,
) -> FogResult<Type> {
    let left = eval_atomic_type_expr(left, env)?;
    let right = eval_atomic_type_expr(right, env)?;

    let mut types = Vec::new();

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

fn eval_function_type(
    left: &CoreAtomicTypeExpr,
    right: &CoreAtomicTypeExpr,
    env: &Environment,
) -> FogResult<Type> {
    let left = eval_atomic_type_expr(left, env)?;
    let right = eval_atomic_type_expr(right, env)?;

    Ok(Type::Function(left.into(), right.into()))
}

fn eval_sum_type(
    left: &CoreAtomicTypeExpr,
    right: &CoreAtomicTypeExpr,
    env: &Environment,
) -> FogResult<Type> {
    let left = eval_atomic_type_expr(left, env)?;
    let right = eval_atomic_type_expr(right, env)?;

    let Type::Sum(ctors1) = left else {
        return Err(static_check_error!(
            None,
            "`{}` is not a data constructor or a sum type",
            left.to_string()
        ));
    };
    let Type::Sum(ctors2) = right else {
        return Err(static_check_error!(
            None,
            "`{}` is not a data constructor or a sum type",
            right.to_string()
        ));
    };

    Ok(Type::Sum([&ctors1[..], &ctors2[..]].concat()))
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

// --- data constructors ---

pub fn register_data_constructors(
    env: &mut Environment,
    parent_sum_type: &Type,
    span: &Span,
) -> FogResult<()> {
    let Type::Sum(ctors) = parent_sum_type else {
        return Err(static_check_error!(
            Some(*span),
            "cannot register data constructors from a non-sum type `{}`",
            parent_sum_type.to_string()
        ));
    };

    for ctor in ctors {
        let ctor_type = nest_function_types(&ctor.types, parent_sum_type.clone());

        println!("{}", ctor.tag);

        env.annotate_type(&ctor.tag, ctor_type.clone(), span)?;
        env.declare_var(&ctor.tag, ctor_type, span)?;
    }

    Ok(())
}

pub fn nest_function_types(field_types: &Vec<Type>, return_type: Type) -> Type {
    field_types.iter().rev().fold(return_type, |ret, ft| {
        Type::Function(ft.clone().into(), ret.into())
    })
}
