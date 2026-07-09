use std::collections::HashMap;
use std::rc::Rc;

use crate::anf::anf::ANFDeclPattern;
use crate::anf::anf::ANFExpr;
use crate::anf::anf::ANFValueExpr;
use crate::anf::anf::AtomicExpr;
use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::value::Value;
use crate::interpreter::variable::ValueVariable;
use crate::parser::Literal;
use crate::parser::core_expr::CoreMatchArmPattern;
use crate::runtime_error;

// --- data constructors ---

// --- value expression evaluator ---

pub fn eval_value_expr(expr: &ANFValueExpr, env: &Environment) -> FogResult<Value> {
    match expr {
        ANFValueExpr::Atomic(expr) => eval_atomic_expr(expr, env),

        ANFValueExpr::FunctionAppl(callee, arg) => {
            let function = eval_atomic_expr(callee, env)?;
            let argument = eval_atomic_expr(arg, env)?;

            eval_function_appl(function, argument)
        }
    }
}

pub fn eval_atomic_expr(expr: &AtomicExpr, env: &Environment) -> FogResult<Value> {
    match expr {
        AtomicExpr::Block { anfs, span } => eval_block(anfs, span, env),

        AtomicExpr::Literal { literal, span } => match *literal {
            Literal::Int32(value) => Ok(Value::Int32(value)),
            Literal::Float32(value) => Ok(Value::Float32(value)),
        },

        AtomicExpr::Var { var, span } => {
            let value_var = env.get_value_var(&format!("{var}"))?;
            value_var
                .value
                .borrow()
                .clone()
                .ok_or_else(|| runtime_error!(Some(*span), "undeclared variable `{var}`"))
        }

        AtomicExpr::Lambda { param, body, .. } => Ok(Value::Function {
            param: param.to_string(),
            body: Rc::new((**body).clone()),
            captured_env: env.flatten().into(),
        }),

        AtomicExpr::Tuple { items, span } => Ok(Value::Tuple(
            items
                .iter()
                .map(|expr| eval_value_expr(expr, env))
                .collect::<Result<Vec<Value>, FogError>>()?,
        )),

        AtomicExpr::Match {
            scrutinee,
            arms,
            span,
        } => {
            let value = eval_atomic_expr(scrutinee, env)?;

            for (pattern, expr) in arms {
                if let Some(bindings) = match_pattern(&value, pattern) {
                    let mut arm_env = Environment::new(Some(env));

                    for (name, val) in &bindings {
                        arm_env
                            .variables
                            .insert(name.clone(), ValueVariable::with_value(name, val.clone()));
                    }

                    return eval_value_expr(expr, &arm_env);
                }
            }

            Err(runtime_error!(Some(*span), "match expression not covered"))
        }
    }
}

pub fn eval_scope(anfs: &Vec<ANFExpr>, env: &mut Environment) -> FogResult<Option<Value>> {
    // value declarations
    for stmt in anfs {
        if let ANFExpr::Declaration(pattern, expr) = stmt {
            match pattern {
                ANFDeclPattern::Single(var) => {
                    let value = eval_value_expr(expr, env)?;
                    env.declare_value(&format!("{var}"), value)?;
                }

                ANFDeclPattern::Tuple(vars) => {
                    let value = eval_value_expr(expr, env)?;
                    eval_tuple_items_declaration(vars, value, env)?;
                }
            }
        }
    }

    // final expression (blocks only)
    for stmt in anfs {
        if let ANFExpr::Atomic(expr) = stmt {
            return Ok(Some(eval_atomic_expr(expr, env)?));
        }
    }

    Ok(None)
}

fn eval_tuple_items_declaration(
    pattern_items: &Vec<ANFDeclPattern>,
    value: Value,
    env: &mut Environment,
) -> FogResult<()> {
    let Value::Tuple(expr_items) = value else {
        return Err(runtime_error!(
            // Some(*span),
            None,
            "cannot declare a non-tuple value `{value}` to a tuple"
        ));
    };

    if pattern_items.len() != expr_items.len() {
        return Err(runtime_error!(
            // Some(*span),
            None,
            "tuple declaration size mismatch"
        ));
    }

    for (pattern_item, value) in pattern_items.iter().zip(expr_items) {
        match pattern_item {
            ANFDeclPattern::Single(var) => {
                // let value = eval_value_expr(expr_item, env)?;
                env.declare_value(&format!("{var}"), value)?;
            }

            ANFDeclPattern::Tuple(vars) => eval_tuple_items_declaration(vars, value, env)?,
        }
    }

    Ok(())
}

fn eval_block(anfs: &Vec<ANFExpr>, span: &Span, env: &Environment) -> FogResult<Value> {
    let mut block_env = Environment::new(Some(env));

    eval_scope(anfs, &mut block_env)?
        .ok_or_else(|| runtime_error!(Some(*span), "final operand not found in block statement"))
}

fn eval_function_appl(function: Value, argument: Value) -> FogResult<Value> {
    match function {
        Value::Function {
            param,
            body,
            captured_env,
            ..
        } => {
            let mut child_env = Environment::new(Some(captured_env.as_ref()));

            child_env
                .variables
                .insert(param.clone(), ValueVariable::with_value(&param, argument));

            eval_value_expr(&body, &child_env)
        }

        Value::NativeFunction { function, .. } => function(argument).map_err(|e| {
            // if e.span.is_none() {
            // e.span = Some(*span);
            // }
            e
        }),

        _ => Err(runtime_error!(
            // Some(*span),
            None,
            "cannot apply a non-function value"
        )),
    }
}

fn match_pattern(value: &Value, pattern: &CoreMatchArmPattern) -> Option<HashMap<String, Value>> {
    // let span = pattern.span();

    match pattern {
        CoreMatchArmPattern::Literal { literal, .. } => match (literal, value) {
            (Literal::Int32(p), Value::Int32(v)) if p == v => Some(HashMap::new()),
            (Literal::Float32(p), Value::Float32(v)) if p == v => Some(HashMap::new()),
            _ => None,
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
