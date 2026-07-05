use crate::error::FogResult;
use crate::error::Span;
use crate::parser::core_expr::CoreExpr;
use crate::static_check::environment::Environment;
use crate::static_check::kind::Kind;
use crate::static_check::r#type::DataConstructor;
use crate::static_check::r#type::Type;
use crate::type_check_error;

// --- annotation (kind or type) ---

pub enum Annotation {
    Kind(Kind),
    Type(Type),
}

pub fn eval_annotation_expr(expr: &CoreExpr, env: &Environment) -> FogResult<Annotation> {
    let span = expr.span();

    match expr {
        CoreExpr::Identifier { name, .. } if name == "Type" => Ok(Annotation::Kind(Kind::Type)),

        CoreExpr::Identifier { name, .. } if env.contains_type(name) => {
            Ok(Annotation::Type(env.get_type(name, &span)?))
        }

        CoreExpr::Identifier { name, .. } => Err(type_check_error!(
            Some(span),
            "unknown type or kind `{}`",
            name
        )),

        CoreExpr::FunctionAppl { .. } => {
            let (head, args) = expr.uncurry();

            if let (CoreExpr::Identifier { name, .. }, &[lhs, rhs]) = (head, args.as_slice()) {
                if name == "->" {
                    return match (
                        eval_annotation_expr(lhs, env)?,
                        eval_annotation_expr(rhs, env)?,
                    ) {
                        (Annotation::Kind(k1), Annotation::Kind(k2)) => {
                            Ok(Annotation::Kind(Kind::Function(k1.into(), k2.into())))
                        }
                        (Annotation::Type(t1), Annotation::Type(t2)) => {
                            Ok(Annotation::Type(Type::Function(t1.into(), t2.into())))
                        }
                        _ => Err(type_check_error!(
                            Some(span),
                            "mixed kind and type levels in `{}`",
                            expr.to_string()
                        )),
                    };
                }
            }

            Ok(Annotation::Type(eval_type_annotation_expr(expr, env)?))
        }

        _ => Ok(Annotation::Type(eval_type_annotation_expr(expr, env)?)),
    }
}

pub fn eval_type_annotation_expr(expr: &CoreExpr, env: &Environment) -> FogResult<Type> {
    let span = expr.span();

    match expr {
        CoreExpr::Identifier { name, .. } => env
            .get_type_var(name, &span)?
            .r#type
            .ok_or_else(|| type_check_error!(Some(span), "undeclared type `{}`", name)),

        CoreExpr::FunctionAppl { .. } => {
            let (head, args) = expr.uncurry();

            let CoreExpr::Identifier { name, .. } = head else {
                return Err(type_check_error!(
                    Some(span),
                    "cannot type annotate a value with data constructor `{}`",
                    expr.to_string()
                ));
            };

            match (name.as_str(), args.as_slice()) {
                ("->", &[lhs, rhs]) => eval_function_type(lhs, rhs, env),
                ("*", &[lhs, rhs]) => eval_product_type(lhs, rhs, env),

                ("+", _) => Err(type_check_error!(
                    Some(span),
                    "cannot type annotate a value with sum types"
                )),

                _ if env.contains_type(name) => apply_type_level_function(name, &args, env, &span),

                _ => Err(type_check_error!(
                    Some(span),
                    "cannot type annotate a value with data constructor `{}`",
                    expr.to_string()
                )),
            }
        }

        _ => Err(type_check_error!(
            Some(span),
            "`{}` is not a type",
            expr.to_string()
        )),
    }
}

pub fn eval_type_definition_expr(expr: &CoreExpr, env: &Environment) -> FogResult<Type> {
    let span = expr.span();

    match expr {
        CoreExpr::Identifier { name, .. } if env.contains_type(name) => env
            .get_type_var(name, &span)?
            .r#type
            .ok_or_else(|| type_check_error!(Some(span), "undeclared type `{}`", name)),

        CoreExpr::Identifier { name, .. } => Ok(Type::Sum(vec![DataConstructor {
            tag: name.clone(),
            types: Vec::new(),
        }])),

        CoreExpr::FunctionAppl { .. } => {
            let (head, args) = expr.uncurry();

            let CoreExpr::Identifier { name, .. } = head else {
                return Err(type_check_error!(
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
                        .map(|&arg| eval_type_annotation_expr(arg, env))
                        .collect::<Result<Vec<Type>, _>>()?;

                    Ok(Type::Sum(vec![DataConstructor {
                        tag: name.clone(),
                        types: field_types,
                    }]))
                }
            }
        }

        _ => Err(type_check_error!(
            Some(span),
            "`{}` is not a valid type definition",
            expr.to_string()
        )),
    }
}

fn eval_product_type(left: &CoreExpr, right: &CoreExpr, env: &Environment) -> FogResult<Type> {
    let left = eval_type_annotation_expr(left, env)?;
    let right = eval_type_annotation_expr(right, env)?;

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

fn eval_function_type(left: &CoreExpr, right: &CoreExpr, env: &Environment) -> FogResult<Type> {
    let left = eval_type_annotation_expr(left, env)?;
    let right = eval_type_annotation_expr(right, env)?;

    Ok(Type::Function(left.into(), right.into()))
}

fn eval_sum_type(left: &CoreExpr, right: &CoreExpr, env: &Environment) -> FogResult<Type> {
    let left = eval_type_definition_expr(left, env)?;
    let right = eval_type_definition_expr(right, env)?;

    let Type::Sum(ctors1) = left else {
        return Err(type_check_error!(
            None,
            "`{}` is not a data constructor or a sum type",
            left.to_string()
        ));
    };
    let Type::Sum(ctors2) = right else {
        return Err(type_check_error!(
            None,
            "`{}` is not a data constructor or a sum type",
            right.to_string()
        ));
    };

    Ok(Type::Sum([&ctors1[..], &ctors2[..]].concat()))
}

pub fn apply_type_level_function(
    fn_name: &str,
    args: &Vec<&CoreExpr>,
    env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    let mut current = env.get_type(fn_name, span)?;

    for &arg in args {
        let Type::Function(param_type, return_type) = current else {
            return Err(type_check_error!(
                Some(*span),
                "`{}` is not a valid type constructor",
                current.to_string()
            ));
        };

        let arg_kind = eval_type_annotation_expr(arg, env)?;

        if arg_kind != *param_type {
            return Err(type_check_error!(
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
