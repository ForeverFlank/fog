use std::collections::HashMap;
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
use crate::interpreter::type_check::expr_type_of;
use crate::interpreter::value::Value;
use crate::interpreter::value::value_type_of;
use crate::interpreter::variable::ValueVariable;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::parser::resolved_expr::ResolvedStatement;
use crate::runtime_error;

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
        ResolvedExpr::Block { statements } => eval_block(statements, env, span),

        ResolvedExpr::Identifier { name } => {
            let var = env.get_value_var(name, span)?;
            var.value
                .borrow()
                .clone()
                .ok_or_else(|| runtime_error!(Some(span.clone()), "undeclared variable `{}`", name))
        }

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

        ResolvedExpr::Match { expr, match_arms } => {
            let value = eval_value_expr(expr, env, span)?;

            for arm in match_arms {
                if let Some(bindings) = match_pattern(&value, &arm.pattern, span)? {
                    let mut arm_env = Environment::new(Some(env));
                    for (name, val) in &bindings {
                        arm_env.variables.insert(
                            name.clone(),
                            ValueVariable::with_value(name, val.clone(), value_type_of(val)),
                        );
                    }
                    return eval_value_expr(&arm.value_expr, &arm_env, span);
                }
            }

            Err(runtime_error!(
                Some(span.clone()),
                "match statement not covered"
            ))
        }
    }
}

pub fn eval_scope(
    statements: &Vec<ResolvedStatement>,
    env: &mut Environment,
) -> FogResult<Option<Value>> {
    // types' kind annotations
    for stmt in statements {
        if let ResolvedStatement::TypeAnnotation { name, expr, span } = stmt {
            if let Ok(Annotation::Kind(kind)) = eval_annotation_expr(expr, env, span) {
                env.annotate_kind(name, kind, span)?;
            }
        }
    }

    // type definitions
    for stmt in statements {
        if let ResolvedStatement::Declaration { name, expr, span } = stmt {
            if env.types.contains_key(name) {
                let defined_type = eval_type_definition_expr(expr, env, span)?;
                env.declare_type(name, defined_type.clone(), span)?;

                if let Type::Sum(_) = &defined_type {
                    register_data_constructors(env, &defined_type, span)?;
                }
            }
        }
    }

    // variables' type annotations
    for stmt in statements {
        if let ResolvedStatement::TypeAnnotation { name, expr, span } = stmt {
            match eval_annotation_expr(expr, env, span)? {
                Annotation::Type(r#type) => env.annotate_type(name, r#type, span)?,
                _ => (),
            }
        }
    }

    // value declarations
    for stmt in statements {
        if let ResolvedStatement::Declaration { name, expr, span } = stmt {
            if env.variables.contains_key(name) {
                let value = eval_value_expr(expr, env, span)?;
                env.declare_value(name, value, span)?;
            }
        }
    }

    // Final expression (blocks only).
    for stmt in statements {
        if let ResolvedStatement::Expression { span, expr } = stmt {
            return Ok(Some(eval_value_expr(expr, env, span)?));
        }
    }

    Ok(None)
}

fn eval_block(
    statements: &Vec<ResolvedStatement>,
    env: &Environment,
    span: &Span,
) -> FogResult<Value> {
    let mut block_env = Environment::new(Some(env));

    eval_scope(statements, &mut block_env)?.ok_or_else(|| {
        runtime_error!(
            Some(span.clone()),
            "final operand not found in block statement"
        )
    })
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
                ValueVariable::with_value(&param_name, argument, param_type),
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

fn match_pattern(
    value: &Value,
    pattern: &ResolvedExpr,
    span: &Span,
) -> FogResult<Option<HashMap<String, Value>>> {
    match pattern {
        ResolvedExpr::Int32Literal { value: pat_val } => match value {
            Value::Int32(val) if val == pat_val => Ok(Some(HashMap::new())),
            _ => Ok(None),
        },

        ResolvedExpr::Float32Literal { value: pat_val } => match value {
            Value::Float32(val) if val == pat_val => Ok(Some(HashMap::new())),
            _ => Ok(None),
        },

        ResolvedExpr::Identifier { name } => {
            if name == "_" {
                // 'else' arm
                Ok(Some(HashMap::new()))
            } else if name.starts_with(|c: char| c.is_uppercase()) {
                // enums
                match value {
                    Value::Constructor { tag, values, .. } if tag == name && values.is_empty() => {
                        Ok(Some(HashMap::new()))
                    }
                    _ => Ok(None),
                }
            } else {
                // bind value to identifier
                let mut bindings = HashMap::new();
                bindings.insert(name.clone(), value.clone());
                Ok(Some(bindings))
            }
        }

        ResolvedExpr::Tuple { items } => match value {
            Value::Tuple(values) if values.len() == items.len() => {
                let mut bindings = HashMap::new();
                for (v, p) in values.iter().zip(items) {
                    match match_pattern(v, p, span)? {
                        None => return Ok(None),
                        Some(b) => bindings.extend(b),
                    }
                }
                Ok(Some(bindings))
            }
            _ => Ok(None),
        },

        // data constructor
        ResolvedExpr::FuncAppl { fn_name, args } => match value {
            Value::Constructor { tag, values, .. }
                if tag == fn_name && values.len() == args.len() =>
            {
                let mut bindings = HashMap::new();
                for (v, p) in values.iter().zip(args) {
                    match match_pattern(v, p, span)? {
                        None => return Ok(None),
                        Some(b) => bindings.extend(b),
                    }
                }
                Ok(Some(bindings))
            }
            _ => Ok(None),
        },

        ResolvedExpr::Block { .. } | ResolvedExpr::Lambda { .. } | ResolvedExpr::Match { .. } => {
            Err(runtime_error!(None, "unsupported pattern `{pattern}`"))
        }
    }
}
