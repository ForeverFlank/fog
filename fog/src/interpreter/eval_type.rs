use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::r#type::DataConstructor;
use crate::interpreter::r#type::Type;
use crate::interpreter::r#type::nest_function_types;
use crate::interpreter::value::Value;
use crate::parser::resolved_expr::ResolvedExpr;

pub fn eval_type_annotation_expr(
    expr: &ResolvedExpr,
    env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    match expr {
        ResolvedExpr::Identifier { name } => Ok(env.get_type(name, span)?),

        // function type
        ResolvedExpr::FuncAppl { fn_name, args } if fn_name == "->" && args.len() == 2 => {
            let left = eval_type_annotation_expr(&args[0], env, span)?;
            let right = eval_type_annotation_expr(&args[1], env, span)?;

            Ok(Type::Function(Box::new(left), Box::new(right)))
        }

        // product type, a.k.a. tuple
        ResolvedExpr::FuncAppl { fn_name, args } if fn_name == "*" && args.len() == 2 => {
            let left = eval_type_annotation_expr(&args[0], env, span)?;
            let right = eval_type_annotation_expr(&args[1], env, span)?;

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

        // sum type
        ResolvedExpr::FuncAppl { fn_name, .. } if fn_name == "+" => Err(FogError::runtime(
            "cannot type annotate a value with sum types".to_string(),
            Some(span.clone()),
        )),

        // either is a function application or a data constructor
        ResolvedExpr::FuncAppl { fn_name, args } => {
            if env.contains_type(fn_name) {
                // the function name is declared,
                // it's a type constructor or a
                // type-level function
                apply_type_level_function(fn_name, args, env, span)
            } else {
                // otherwise, it's a data constructor,
                // which is invalid here
                Err(FogError::runtime(
                    format!(
                        "cannot type annotate a value with data constructor `{}`",
                        expr.to_string()
                    ),
                    Some(span.clone()),
                ))
            }
        }

        _ => Err(FogError::runtime(
            format!("`{}` is not a type", expr.to_string()),
            Some(span.clone()),
        )),
    }
}

pub fn eval_type_definition_expr(
    expr: &ResolvedExpr,
    env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    match expr {
        ResolvedExpr::Identifier { name } => {
            if env.contains_type(name) {
                // declared type
                env.get_type(name, span)
            } else {
                // enum
                Ok(Type::Sum(vec![DataConstructor {
                    tag: name.clone(),
                    types: vec![],
                }]))
            }
        }

        ResolvedExpr::FuncAppl { fn_name, args } if fn_name == "+" && args.len() == 2 => {
            let left = eval_type_definition_expr(&args[0], env, span)?;
            let right = eval_type_definition_expr(&args[1], env, span)?;

            let Type::Sum(ctors1) = left else {
                return Err(FogError::runtime(
                    format!(
                        "`{}` is not a data constructor or a sum type",
                        left.to_string()
                    ),
                    Some(span.clone()),
                ));
            };

            let Type::Sum(ctors2) = right else {
                return Err(FogError::runtime(
                    format!(
                        "`{}` is not a data constructor or a sum type",
                        right.to_string()
                    ),
                    Some(span.clone()),
                ));
            };

            Ok(Type::Sum([&ctors1[..], &ctors2[..]].concat()))
        }

        _ => eval_type_annotation_expr(expr, env, span),
    }
}

fn apply_type_level_function(
    fn_name: &str,
    args: &Vec<ResolvedExpr>,
    env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    let mut current = env.get_type(fn_name, span)?;

    for arg in args {
        let Type::Function(param_type, return_type) = current else {
            return Err(FogError::runtime(
                format!("`{}` is not a valid type constructor", current.to_string()),
                Some(span.clone()),
            ));
        };

        let arg_type = eval_type_annotation_expr(arg, env, span)?;

        if arg_type != *param_type {
            return Err(FogError::runtime(
                format!(
                    "type mismatch applying `{}`\n\
                     expected `{}`, found `{}`",
                    fn_name,
                    param_type.to_string(),
                    arg_type.to_string()
                ),
                Some(span.clone()),
            ));
        }

        current = *return_type;
    }

    Ok(current)
}

// --- data constructors ---

pub fn make_data_constructor_function(
    tag: String,
    remaining_fields: Vec<Type>,
    parent_type: Type,
    collected_fields: Vec<Value>,
) -> Value {
    let [next_field, rest @ ..] = remaining_fields.as_slice() else {
        return Value::Constructor {
            tag,
            values: collected_fields,
            r#type: parent_type,
        };
    };

    let next_field = next_field.clone();
    let rest = rest.to_vec();
    let parent_type = parent_type.clone();

    let return_type = nest_function_types(&rest, parent_type.clone());

    Value::NativeFunction {
        param_type: next_field,
        return_type,
        function: Rc::new(move |val: Value| {
            let mut collected_fields = collected_fields.clone();
            collected_fields.push(val);
            Ok(make_data_constructor_function(
                tag.clone(),
                rest.clone(),
                parent_type.clone(),
                collected_fields,
            ))
        }),
    }
}
