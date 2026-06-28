use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval_type::Annotation;
use crate::interpreter::eval_type::eval_annotation_expr;
use crate::interpreter::eval_type::eval_type_annotation_expr;
use crate::interpreter::eval_type::eval_type_definition_expr;
use crate::interpreter::r#type::Type;
use crate::interpreter::r#type::nest_function_types;
use crate::interpreter::typecheck::expr_type_of;
use crate::interpreter::value::Value;
use crate::interpreter::variable::ValueVariable;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::parser::resolved_expr::ResolvedStatement;
use crate::runtime_error;

// --- annotation ---

pub fn annotate(
    name: &String,
    expr: &ResolvedExpr,
    env: &mut Environment,
    span: &Span,
) -> FogResult<()> {
    match eval_annotation_expr(expr, env, span)? {
        Annotation::Kind(kind) => env.annotate_kind(name, kind, span),
        Annotation::Type(r#type) => env.annotate_type(name, r#type, span),
    }
}

// --- declaration ---

pub fn declare(
    name: &String,
    expr: &ResolvedExpr,
    env: &mut Environment,
    span: &Span,
) -> FogResult<()> {
    if env.variables.contains_key(name) {
        env.declare_value(name, eval_value_expr(expr, env, span)?, span)?;
        return Ok(());
    }

    if env.types.contains_key(name) {
        let defined_type = eval_type_definition_expr(expr, env, span)?;
        env.declare_type(name, defined_type.clone(), span)?;

        if let Type::Sum(..) = &defined_type {
            register_data_constructors(env, &defined_type, span)?;
        }

        return Ok(());
    }

    Err(runtime_error!(
        Some(span.clone()),
        "unannotated variable `{}`",
        name
    ))
}

// --- data constructors ---

pub fn register_data_constructors(
    env: &mut Environment,
    parent_sum_type: &Type,
    span: &Span,
) -> FogResult<()> {
    let Type::Sum(ctors) = parent_sum_type else {
        return Err(runtime_error!(
            Some(span.clone()),
            "cannot register data constructors from a non-sum type `{}`",
            parent_sum_type.to_string()
        ));
    };

    for ctor in ctors {
        let ctor_type = nest_function_types(&ctor.types, parent_sum_type.clone());
        let ctor_value = make_data_constructor_function(
            ctor.tag.clone(),
            ctor.types.clone(),
            parent_sum_type.clone(),
            Vec::new(),
        );

        env.annotate_type(&ctor.tag, ctor_type, span)?;
        env.declare_value(&ctor.tag, ctor_value, span)?;
    }

    Ok(())
}

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

// --- value expression evaluator ---

pub fn eval_value_expr(expr: &ResolvedExpr, env: &Environment, span: &Span) -> FogResult<Value> {
    match expr {
        ResolvedExpr::Block { statements } => {
            let mut block_env = Environment::new(Some(env));

            for stmt in statements {
                match stmt {
                    ResolvedStatement::TypeAnnotation { name, expr, span } => {
                        annotate(name, expr, &mut block_env, span)?;
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
        } => {
            let param_type = eval_type_annotation_expr(param_type, env, span)?;
            let return_type = expr_type_of(body, env, span)?;
            Ok(Value::Function {
                param_name: param_name.clone(),
                param_type,
                return_type,
                body: Rc::clone(body),
                captured_env: env.flatten().into(),
            })
        }

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
            ..
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
