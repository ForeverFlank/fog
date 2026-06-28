use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::kind::Kind;
use crate::interpreter::r#type::DataConstructor;
use crate::interpreter::r#type::Type;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::runtime_error;

// --- annotation (kind or type) ---

pub enum Annotation {
    Kind(Kind),
    Type(Type),
}

pub fn eval_annotation_expr(
    expr: &ResolvedExpr,
    env: &Environment,
    span: &Span,
) -> FogResult<Annotation> {
    match expr {
        ResolvedExpr::Identifier { name } if name == "Type" => Ok(Annotation::Kind(Kind::Type)),

        ResolvedExpr::Identifier { name } if env.contains_type(name) => {
            Ok(Annotation::Type(env.get_type(name, span)?))
        }

        ResolvedExpr::Identifier { name } => Err(runtime_error!(
            Some(span.clone()),
            "unknown type or kind `{}`",
            name
        )),

        ResolvedExpr::FuncAppl { fn_name, args } if fn_name == "->" && args.len() == 2 => {
            match (
                eval_annotation_expr(&args[0], env, span)?,
                eval_annotation_expr(&args[1], env, span)?,
            ) {
                (Annotation::Kind(k1), Annotation::Kind(k2)) => {
                    Ok(Annotation::Kind(Kind::Function(k1.into(), k2.into())))
                }
                (Annotation::Type(t1), Annotation::Type(t2)) => {
                    Ok(Annotation::Type(Type::Function(t1.into(), t2.into())))
                }
                _ => Err(runtime_error!(
                    Some(span.clone()),
                    "mixed kind and type levels in `{}`",
                    expr.to_string()
                )),
            }
        }

        _ => Ok(Annotation::Type(eval_type_annotation_expr(
            expr, env, span,
        )?)),
    }
}

pub fn eval_type_annotation_expr(
    expr: &ResolvedExpr,
    env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    match expr {
        ResolvedExpr::Identifier { name } => env
            .get_type_var(name, span)?
            .r#type
            .ok_or_else(|| runtime_error!(Some(span.clone()), "undeclared type `{}`", name)),

        ResolvedExpr::FuncAppl { fn_name, args } if fn_name == "->" && args.len() == 2 => {
            eval_function_type(&args[0], &args[1], env, span)
        }

        ResolvedExpr::FuncAppl { fn_name, args } if fn_name == "*" && args.len() == 2 => {
            eval_product_type(&args[0], &args[1], env, span)
        }

        ResolvedExpr::FuncAppl { fn_name, .. } if fn_name == "+" => Err(runtime_error!(
            Some(span.clone()),
            "cannot type annotate a value with sum types"
        )),

        ResolvedExpr::FuncAppl { fn_name, args } if env.contains_type(fn_name) => {
            apply_type_level_function(fn_name, args, env, span)
        }

        ResolvedExpr::FuncAppl { .. } => Err(runtime_error!(
            Some(span.clone()),
            "cannot type annotate a value with data constructor `{}`",
            expr.to_string()
        )),

        _ => Err(runtime_error!(
            Some(span.clone()),
            "`{}` is not a type",
            expr.to_string()
        )),
    }
}

pub fn eval_type_definition_expr(
    expr: &ResolvedExpr,
    env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    match expr {
        ResolvedExpr::Identifier { name } if env.contains_type(name) => env
            .get_type_var(name, span)?
            .r#type
            .ok_or_else(|| runtime_error!(Some(span.clone()), "undeclared type `{}`", name)),

        ResolvedExpr::Identifier { name } => Ok(Type::Sum(vec![DataConstructor {
            tag: name.clone(),
            types: Vec::new(),
        }])),

        ResolvedExpr::FuncAppl { fn_name, args } if fn_name == "->" && args.len() == 2 => {
            eval_function_type(&args[0], &args[1], env, span)
        }

        ResolvedExpr::FuncAppl { fn_name, args } if fn_name == "*" && args.len() == 2 => {
            eval_product_type(&args[0], &args[1], env, span)
        }

        ResolvedExpr::FuncAppl { fn_name, args } if fn_name == "+" && args.len() == 2 => {
            eval_sum_type(&args[0], &args[1], env, span)
        }

        ResolvedExpr::FuncAppl { fn_name, args } if env.contains_type(fn_name) => {
            apply_type_level_function(fn_name, args, env, span)
        }

        // data constructor
        ResolvedExpr::FuncAppl { fn_name, args } => {
            let field_types = args
                .iter()
                .map(|arg| eval_type_annotation_expr(arg, env, span))
                .collect::<Result<Vec<Type>, _>>()?;

            Ok(Type::Sum(vec![DataConstructor {
                tag: fn_name.clone(),
                types: field_types,
            }]))
        }

        _ => Err(runtime_error!(
            Some(span.clone()),
            "`{}` is not a valid type definition",
            expr.to_string()
        )),
    }
}

fn eval_product_type(
    left: &ResolvedExpr,
    right: &ResolvedExpr,
    env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    let left = eval_type_annotation_expr(left, env, span)?;
    let right = eval_type_annotation_expr(right, env, span)?;

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
    left: &ResolvedExpr,
    right: &ResolvedExpr,
    env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    let left = eval_type_annotation_expr(left, env, span)?;
    let right = eval_type_annotation_expr(right, env, span)?;

    Ok(Type::Function(left.into(), right.into()))
}

fn eval_sum_type(
    left: &ResolvedExpr,
    right: &ResolvedExpr,
    env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    let left = eval_type_definition_expr(left, env, span)?;
    let right = eval_type_definition_expr(right, env, span)?;

    let Type::Sum(ctors1) = left else {
        return Err(runtime_error!(
            Some(span.clone()),
            "`{}` is not a data constructor or a sum type",
            left.to_string()
        ));
    };
    let Type::Sum(ctors2) = right else {
        return Err(runtime_error!(
            Some(span.clone()),
            "`{}` is not a data constructor or a sum type",
            right.to_string()
        ));
    };

    Ok(Type::Sum([&ctors1[..], &ctors2[..]].concat()))
}

pub fn apply_type_level_function(
    fn_name: &str,
    args: &Vec<ResolvedExpr>,
    env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    let mut current = env.get_type(fn_name, span)?;

    for arg in args {
        let Type::Function(param_type, return_type) = current else {
            return Err(runtime_error!(
                Some(span.clone()),
                "`{}` is not a valid type constructor",
                current.to_string()
            ));
        };

        let arg_kind = eval_type_annotation_expr(arg, env, span)?;

        if arg_kind != *param_type {
            return Err(runtime_error!(
                Some(span.clone()),
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
