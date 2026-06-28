use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval_type::Annotation;
use crate::interpreter::eval_type::eval_annotation_expr;
use crate::interpreter::eval_type::eval_type_definition_expr;
use crate::interpreter::eval_value::eval_value_expr;
use crate::parser::resolved_expr::ResolvedExpr;
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
        // declare variable
        env.declare_value(name, eval_value_expr(expr, env, span)?, span)?;
        return Ok(());
    }

    if env.types.contains_key(name) {
        // declare type
        env.declare_type(name, eval_type_definition_expr(expr, env, span)?, span)?;
        return Ok(());
    }

    Err(runtime_error!(
        Some(span.clone()),
        "unannotated variable `{}`",
        name
    ))
}
