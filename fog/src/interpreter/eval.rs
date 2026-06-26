use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::r#type::DataConstructor;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;
use crate::parser::resolved_expr::ResolvedExpr;

pub fn eval_value_expr(expr: &ResolvedExpr, env: &Environment, span: &Span) -> FogResult<Value> {
    match expr {
        // variable
        ResolvedExpr::Identifier(name) => env.get_var(&name)?.value.ok_or_else(|| {
            FogError::runtime(
                format!("undeclared variable `{}`", name),
                Some(span.clone()),
            )
        }),

        // literals
        ResolvedExpr::Int32Literal(value) => Ok(Value::Int32(*value)),
        ResolvedExpr::Float32Literal(value) => Ok(Value::Float32(*value)),

        // AST lambda -> interpreter function
        ResolvedExpr::Lambda(param_name, param_type, body) => Ok(Value::Function {
            param_name: param_name.clone(),
            param_type: eval_type_annotation_expr(param_type, env)?,
            body: Rc::clone(body),
            captured_env: Box::new(env.clone()),
        }),

        // tuple
        ResolvedExpr::Tuple(exprs) => Ok(Value::Tuple(
            exprs
                .iter()
                .map(|expr| Ok(eval_value_expr(expr, env, span)?.into()))
                .collect::<Result<Vec<Value>, FogError>>()?,
        )),

        // function application
        ResolvedExpr::FuncAppl(fn_name, args) => {
            let mut result: Value =
                eval_value_expr(&ResolvedExpr::Identifier(fn_name.clone()), env, span)?;

            for arg in args {
                let argument: Value = eval_value_expr(arg, env, span)?;
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
            let mut child_env: Environment = Environment::new(Some(captured_env));

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

        Value::NativeFunction { function, .. } => function(argument),

        _ => Err(FogError::runtime(
            "cannot apply a non-function value".to_string(),
            Some(span.clone()),
        )),
    }
}

// --- type expressions ---

pub fn eval_type_annotation_expr(expr: &ResolvedExpr, env: &Environment) -> FogResult<Type> {
    match expr {
        ResolvedExpr::Identifier(name) => Ok(env.get_type(name)?),

        // function type
        ResolvedExpr::FuncAppl(fn_name, args) if fn_name == "->" && args.len() == 2 => {
            let left: Type = eval_type_annotation_expr(&args[0], env)?;
            let right: Type = eval_type_annotation_expr(&args[1], env)?;

            Ok(Type::Function(Box::new(left), Box::new(right)))
        }

        // product type, a.k.a. tuple
        ResolvedExpr::FuncAppl(fn_name, args) if fn_name == "*" && args.len() == 2 => {
            let left: Type = eval_type_annotation_expr(&args[0], env)?;
            let right: Type = eval_type_annotation_expr(&args[1], env)?;

            let mut types: Vec<Type> = Vec::new();

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
        ResolvedExpr::FuncAppl(fn_name, args) if fn_name == "+" && args.len() == 2 => {
            Err(FogError::runtime(
                "cannot type annotate a value with sum types".to_string(),
                None,
            ))
        }

        // either is a function application or a data constructor
        ResolvedExpr::FuncAppl(name, args) => {
            if env.contains_type(name) {
                // the function name is declared,
                // it's a type constructor or a
                // type-level function
                apply_type_level_function(name, args, env)
            } else {
                // otherwise, it's a data constructor,
                // which is invalid here
                Err(FogError::runtime(
                    "cannot type annotate a value with data constructor".to_string(),
                    None,
                ))
            }
        }

        _ => Err(FogError::runtime(
            format!("`{}` is not a type", expr.to_string()),
            None,
        )),
    }
}

pub fn eval_type_definition_expr(expr: &ResolvedExpr, env: &Environment) -> FogResult<Type> {
    match expr {
        ResolvedExpr::Identifier(name) => {
            if env.contains_type(name) {
                // declared type
                env.get_type(name)
            } else {
                // enum
                Ok(Type::Sum(vec![DataConstructor {
                    tag: name.clone(),
                    types: vec![],
                }]))
            }
        }

        ResolvedExpr::FuncAppl(fn_name, args) if fn_name == "+" && args.len() == 2 => {
            let left: Type = eval_type_definition_expr(&args[0], env)?;
            let right: Type = eval_type_definition_expr(&args[1], env)?;

            let Type::Sum(ctors1) = left else {
                return Err(FogError::runtime(
                    format!(
                        "`{}` is not a data constructor or a sum type",
                        left.to_string()
                    ),
                    None,
                ));
            };

            let Type::Sum(ctors2) = right else {
                return Err(FogError::runtime(
                    format!(
                        "`{}` is not a data constructor or a sum type",
                        right.to_string()
                    ),
                    None,
                ));
            };

            Ok(Type::Sum([&ctors1[..], &ctors2[..]].concat()))
        }

        _ => eval_type_annotation_expr(expr, env),
    }
}

fn apply_type_level_function(
    fn_name: &str,
    args: &Vec<ResolvedExpr>,
    env: &Environment,
) -> FogResult<Type> {
    let mut current: Type = env.get_type(fn_name)?;

    for arg in args {
        let Type::Function(param_type, return_type) = current else {
            return Err(FogError::runtime(
                format!("`{}` is not a valid type constructor", current.to_string()),
                None,
            ));
        };

        let arg_type: Type = eval_type_annotation_expr(arg, env)?;

        if arg_type != *param_type {
            return Err(FogError::runtime(
                format!(
                    "type mismatch applying `{}`\n\
                     expected `{}`, found `{}`",
                    fn_name,
                    param_type.to_string(),
                    arg_type.to_string()
                ),
                None,
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

    let next_field: Type = next_field.clone();
    let rest: Vec<Type> = rest.to_vec();
    let parent_type: Type = parent_type.clone();

    let return_type: Type = rest
        .iter()
        .rev()
        .fold(parent_type.clone(), |ret, field_type| {
            Type::Function(Box::new(field_type.clone()), Box::new(ret))
        });

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
