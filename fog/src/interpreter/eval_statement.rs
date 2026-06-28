use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval_type::eval_type_definition_expr;
use crate::interpreter::eval_value::eval_value_expr;
use crate::interpreter::kind::Kind;
use crate::interpreter::r#type::Type;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::runtime_error;

// --- type annotation ---

enum AnnotationLevel {
    Kind(Kind),
    Type(Type),
}

fn annotation_level_of(
    expr: &ResolvedExpr,
    env: &mut Environment,
    span: &Span,
) -> FogResult<AnnotationLevel> {
    match expr {
        ResolvedExpr::Block { statements } => todo!(),

        ResolvedExpr::Identifier { name } => {
            if env.contains_kind(name) {
                Ok(AnnotationLevel::Kind(env.get_kind(name, span)?))
            } else if env.contains_type(name) {
                Ok(AnnotationLevel::Type(env.get_type(name, span)?))
            } else {
                Err(runtime_error!(
                    Some(span.clone()),
                    "unknown type or type constructor `{expr}`"
                ))
            }
        }

        ResolvedExpr::Lambda {
            param_name,
            param_type,
            body,
        } => todo!(),

        ResolvedExpr::Tuple { items } => todo!(),
        ResolvedExpr::FuncAppl { fn_name, args } => todo!(),

        ResolvedExpr::Int32Literal { value } => todo!(),
        ResolvedExpr::Float32Literal { value } => todo!(),
    }
}

pub fn annotate_type(
    name: &String,
    expr: &ResolvedExpr,
    env: &mut Environment,
    span: &Span,
) -> FogResult<()> {
    let level = annotation_level_of(expr, env, span)?;

    match level {
        AnnotationLevel::Type(r#type) => env.annotate_type(name, r#type, span),
        AnnotationLevel::Kind(kind) => env.annotate_kind(name, kind, span),
    }
}

// --- declaration ---

pub fn declare(
    name: &String,
    expr: &ResolvedExpr,
    env: &mut Environment,
    span: &Span,
) -> FogResult<()> {
    // declare variable
    if env.variables.contains_key(name) {
        env.declare_value(name, eval_value_expr(expr, env, span)?, span)?;
        return Ok(());
    }

    // declare type
    if env.types.contains_key(name) {
        env.declare_type(name, eval_type_definition_expr(expr, env, span)?, span)?;
        return Ok(());
    }

    Err(runtime_error!(
        Some(span.clone()),
        "unannotated variable `{}`",
        name
    ))
}
