use std::collections::HashMap;
use std::rc::Rc;

use crate::anf::anf::ANFExpr;
use crate::anf::anf::AtomicExpr;
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
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreMatchArmPattern;
use crate::parser::core_expr::CoreStatement;
use crate::parser::core_expr::CoreTupleDeclPattern;
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

pub fn eval_value_expr(expr: &AtomicExpr, env: &Environment) -> FogResult<Value> {
    match expr {
        AtomicExpr::Block { anfs, span } => eval_block(anfs, span, env),

        AtomicExpr::Var { var, span } => {
            let var = env.get_value_var(name, span)?;
            var.value
                .borrow()
                .clone()
                .ok_or_else(|| runtime_error!(Some(*span), "undeclared variable `{}`", name))
        }

        AtomicExpr::Literal { literal, .. } => match *literal {
            Literal::Int32(value) => Ok(Value::Int32(value)),
            Literal::Float32(value) => Ok(Value::Float32(value)),
        },

        AtomicExpr::Lambda {
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
                body: Rc::new((**body).clone()),
                captured_env: env.flatten().into(),
            })
        }

        AtomicExpr::Tuple { items, .. } => Ok(Value::Tuple(
            items
                .iter()
                .map(|expr| eval_value_expr(expr, env))
                .collect::<Result<Vec<Value>, FogError>>()?,
        )),

        AtomicExpr::FunctionAppl { callee, arg, span } => {
            let function = eval_value_expr(callee, env)?;
            let argument = eval_value_expr(arg, env)?;

            eval_function_appl(function, argument, span)
        }

        AtomicExpr::Match {
            scrutinee,
            match_arms,
            span,
        } => {
            let value = eval_value_expr(scrutinee, env)?;

            for arm in match_arms {
                if let Some(bindings) = match_pattern(&value, &arm.pattern) {
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

            Err(runtime_error!(Some(*span), "match expression not covered"))
        }
    }
}

pub fn eval_scope(anfs: &Vec<ANFExpr>, env: &mut Environment) -> FogResult<Option<Value>> {
    // value declarations
    for stmt in anfs {
        if let CoreStatement::Declaration { pattern, expr, .. } = stmt {
            match pattern {
                CoreDeclPattern::Identifier { name, span } => {
                    if !env.types.contains_key(name) {
                        let value = eval_value_expr(expr, env)?;
                        env.declare_value(name, value, span)?;
                    }
                }

                CoreDeclPattern::Tuple {
                    items: pattern_items,
                    span,
                } => {
                    let value = eval_value_expr(expr, env)?;
                    eval_tuple_items_declaration(pattern_items, value, env, span)?;
                }
            }
        }
    }

    // final expression (blocks only)
    for stmt in anfs {
        if let ANFExpr::Atomic(expr) = stmt {
            return Ok(Some(eval_value_expr(expr, env)?));
        }
    }

    Ok(None)
}

fn eval_tuple_items_declaration(
    pattern_items: &Vec<CoreTupleDeclPattern>,
    value: Value,
    env: &mut Environment,
    span: &Span,
) -> FogResult<()> {
    let Value::Tuple(expr_items) = value else {
        return Err(runtime_error!(
            Some(*span),
            "cannot declare a non-tuple value `{value}` to a tuple"
        ));
    };

    if pattern_items.len() != expr_items.len() {
        return Err(runtime_error!(
            Some(*span),
            "tuple declaration size mismatch"
        ));
    }

    for (pattern_item, value) in pattern_items.iter().zip(expr_items) {
        match pattern_item {
            CoreTupleDeclPattern::Identifier { name, span } => {
                // let value = eval_value_expr(expr_item, env)?;
                env.declare_value(name, value, span)?;
            }

            CoreTupleDeclPattern::Tuple {
                items: pattern_items,
                span,
            } => eval_tuple_items_declaration(pattern_items, value, env, span)?,
        }
    }

    Ok(())
}

fn eval_block(statements: &Vec<CoreStatement>, span: &Span, env: &Environment) -> FogResult<Value> {
    let mut block_env = Environment::new(Some(env));

    eval_scope(statements, &mut block_env)?
        .ok_or_else(|| runtime_error!(Some(*span), "final operand not found in block statement"))
}

fn eval_function_appl(function: Value, argument: Value, span: &Span) -> FogResult<Value> {
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
    pattern: &CoreMatchArmPattern,
) -> FogResult<Option<HashMap<String, Value>>> {
    // let span = pattern.span();

    match pattern {
        CoreMatchArmPattern::Literal { literal, .. } => match (literal, value) {
            (Literal::Int32(p), Value::Int32(v)) if p == v => Ok(Some(HashMap::new())),
            (Literal::Float32(p), Value::Float32(v)) if p == v => Ok(Some(HashMap::new())),
            _ => Ok(None),
        },

        CoreMatchArmPattern::Identifier { name, .. } => {
            if name == "_" {
                // wildcard
                Some(HashMap::new())
            } else if name.starts_with(|c: char| c.is_uppercase()) {
                // nullary constructor
                match value {
                    Value::Constructor { tag, values, .. } if tag == name && values.is_empty() => {
                        Some(HashMap::new())
                    }
                    _ => None,
                }
            } else {
                // bind value to identifier
                let mut bindings = HashMap::new();
                bindings.insert(name.clone(), value.clone());
                Some(bindings)
            }
        }

        CoreMatchArmPattern::Tuple { items, .. } => match value {
            Value::Tuple(values) if values.len() == items.len() => {
                let mut bindings = HashMap::new();
                for (v, p) in values.iter().zip(items) {
                    match match_pattern(v, p) {
                        None => return None,
                        Some(b) => bindings.extend(b),
                    }
                }
                Some(bindings)
            }
            _ => None,
        },

        // data constructor pattern
        CoreMatchArmPattern::DataConstructor { name, args, .. } => match value {
            Value::Constructor { tag, values, .. } if tag == name && values.len() == args.len() => {
                let mut bindings = HashMap::new();
                for (v, p) in values.iter().zip(args) {
                    match match_pattern(v, p) {
                        None => return None,
                        Some(b) => bindings.extend(b),
                    }
                }
                Some(bindings)
            }
            _ => None,
        },
    }
}
