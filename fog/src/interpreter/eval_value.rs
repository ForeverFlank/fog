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
use crate::parser::Literal;
use crate::parser::desugared_expr::DesugaredDeclPattern;
use crate::parser::desugared_expr::DesugaredExpr;
use crate::parser::desugared_expr::DesugaredStatement;
use crate::runtime_error;

// --- data constructors ---

pub fn register_data_constructors(
    env: &mut Environment,
    parent_sum_type: &Type,
    span: &Span,
) -> FogResult<()> {
    let Type::Sum(ctors) = parent_sum_type else {
        return Err(runtime_error!(
            Some(*span),
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

pub fn eval_value_expr(expr: &DesugaredExpr, env: &Environment) -> FogResult<Value> {
    match expr {
        DesugaredExpr::Block { statements, span } => eval_block(statements, span, env),

        DesugaredExpr::Identifier { name, span } => {
            let var = env.get_value_var(name, span)?;
            var.value
                .borrow()
                .clone()
                .ok_or_else(|| runtime_error!(Some(*span), "undeclared variable `{}`", name))
        }

        DesugaredExpr::Literal { literal, .. } => match *literal {
            Literal::Int32(value) => Ok(Value::Int32(value)),
            Literal::Float32(value) => Ok(Value::Float32(value)),
        },

        DesugaredExpr::Lambda {
            param_name,
            param_type,
            body,
            ..
        } => {
            let param_type = eval_type_annotation_expr(param_type, env)?;
            let return_type = expr_type_of(body, env)?;
            Ok(Value::Function {
                param_name: param_name.clone(),
                param_type,
                return_type,
                body: Rc::clone(body),
                captured_env: env.flatten().into(),
            })
        }

        DesugaredExpr::Tuple { items, .. } => Ok(Value::Tuple(
            items
                .iter()
                .map(|expr| eval_value_expr(expr, env))
                .collect::<Result<Vec<Value>, FogError>>()?,
        )),

        DesugaredExpr::FunctionAppl {
            fn_name,
            args,
            span,
        } => {
            let mut result = eval_value_expr(
                &DesugaredExpr::Identifier {
                    name: fn_name.clone(),
                    span: *span,
                },
                env,
            )?;

            for arg in args {
                let argument = eval_value_expr(arg, env)?;
                result = apply_function(result, argument, span)?;
            }

            Ok(result)
        }

        DesugaredExpr::Match {
            expr,
            match_arms,
            span,
        } => {
            let value = eval_value_expr(expr, env)?;

            for arm in match_arms {
                if let Some(bindings) = match_pattern(&value, &arm.pattern)? {
                    let mut arm_env = Environment::new(Some(env));
                    for (name, val) in &bindings {
                        arm_env.variables.insert(
                            name.clone(),
                            ValueVariable::with_value(name, val.clone(), value_type_of(val)),
                        );
                    }
                    return eval_value_expr(&arm.value_expr, &arm_env);
                }
            }

            Err(runtime_error!(Some(span), "match expression not covered"))
        }
    }
}

pub fn eval_scope(
    statements: &Vec<DesugaredStatement>,
    env: &mut Environment,
) -> FogResult<Option<Value>> {
    // type's kind annotations
    for stmt in statements {
        if let DesugaredStatement::TypeAnnotation { name, expr, span } = stmt {
            if let Ok(Annotation::Kind(kind)) = eval_annotation_expr(expr, env) {
                env.annotate_kind(name, kind, span)?;
            }
        }
    }

    // type definitions
    for stmt in statements {
        if let DesugaredStatement::Declaration {
            pattern,
            expr,
            span,
        } = stmt
        {
            match pattern {
                DesugaredDeclPattern::Identifier { name, span } => {
                    if env.types.contains_key(name) {
                        let defined_type = eval_type_definition_expr(expr, env)?;
                        env.declare_type(name, defined_type.clone(), span)?;

                        if let Type::Sum(_) = &defined_type {
                            register_data_constructors(env, &defined_type, span)?;
                        }
                    }
                }

                _ => todo!(),
            }
        }
    }

    // variable's type annotations
    for stmt in statements {
        if let DesugaredStatement::TypeAnnotation { name, expr, span } = stmt {
            match eval_annotation_expr(expr, env)? {
                Annotation::Type(r#type) => env.annotate_type(name, r#type, span)?,
                _ => (),
            }
        }
    }

    // value declarations
    for stmt in statements {
        if let DesugaredStatement::Declaration {
            pattern,
            expr,
            span,
        } = stmt
        {
            match pattern {
                DesugaredDeclPattern::Identifier { name, span } => {
                    if !env.types.contains_key(name) {
                        let value = eval_value_expr(expr, env)?;
                        env.declare_value(name, value, span)?;
                    }
                }

                _ => todo!(),
            }
        }
    }

    // final expression (blocks only)
    for stmt in statements {
        if let DesugaredStatement::Expression { expr, .. } = stmt {
            return Ok(Some(eval_value_expr(expr, env)?));
        }
    }

    Ok(None)
}

fn eval_block(
    statements: &Vec<DesugaredStatement>,
    span: &Span,
    env: &Environment,
) -> FogResult<Value> {
    let mut block_env = Environment::new(Some(env));

    eval_scope(statements, &mut block_env)?
        .ok_or_else(|| runtime_error!(Some(*span), "final operand not found in block statement"))
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

            eval_value_expr(&body, &child_env)
        }

        Value::NativeFunction { function, .. } => function(argument).map_err(|mut e| {
            if e.span.is_none() {
                e.span = Some(*span);
            }
            e
        }),

        _ => Err(runtime_error!(
            Some(*span),
            "cannot apply a non-function value"
        )),
    }
}

fn match_pattern(
    value: &Value,
    pattern: &DesugaredExpr,
) -> FogResult<Option<HashMap<String, Value>>> {
    let span = pattern.span();

    match pattern {
        DesugaredExpr::Literal { literal, .. } => match (literal, value) {
            (Literal::Int32(p), Value::Int32(v)) if p == v => Ok(Some(HashMap::new())),
            (Literal::Float32(p), Value::Float32(v)) if p == v => Ok(Some(HashMap::new())),
            _ => Ok(None),
        },

        DesugaredExpr::Identifier { name, .. } => {
            if name == "_" {
                // wildcard
                Ok(Some(HashMap::new()))
            } else if name.starts_with(|c: char| c.is_uppercase()) {
                // nullary constructor
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

        DesugaredExpr::Tuple { items, .. } => match value {
            Value::Tuple(values) if values.len() == items.len() => {
                let mut bindings = HashMap::new();
                for (v, p) in values.iter().zip(items) {
                    match match_pattern(v, p)? {
                        None => return Ok(None),
                        Some(b) => bindings.extend(b),
                    }
                }
                Ok(Some(bindings))
            }
            _ => Ok(None),
        },

        // data constructor pattern
        DesugaredExpr::FunctionAppl { fn_name, args, .. } => match value {
            Value::Constructor { tag, values, .. }
                if tag == fn_name && values.len() == args.len() =>
            {
                let mut bindings = HashMap::new();
                for (v, p) in values.iter().zip(args) {
                    match match_pattern(v, p)? {
                        None => return Ok(None),
                        Some(b) => bindings.extend(b),
                    }
                }
                Ok(Some(bindings))
            }
            _ => Ok(None),
        },

        DesugaredExpr::Block { .. }
        | DesugaredExpr::Lambda { .. }
        | DesugaredExpr::Match { .. } => Err(runtime_error!(
            Some(span),
            "unsupported pattern `{pattern}`"
        )),
    }
}
