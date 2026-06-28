use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::kind::Kind;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::runtime_error;

pub fn eval_kind_expr(expr: &ResolvedExpr, env: &Environment, span: &Span) -> FogResult<Kind> {
    match expr {
        ResolvedExpr::Identifier { name } if name == "Type" => Ok(Kind::Type),

        ResolvedExpr::FuncAppl { fn_name, args } if fn_name == "->" && args.len() == 2 => {
            eval_function_kind(&args[0], &args[1], env, span)
        }

        _ => Err(runtime_error!(
            Some(span.clone()),
            "invalid kind `{}`",
            expr.to_string()
        )),
    }
}

fn eval_function_kind(
    left: &ResolvedExpr,
    right: &ResolvedExpr,
    env: &Environment,
    span: &Span,
) -> FogResult<Kind> {
    let left = eval_kind_expr(left, env, span)?;
    let right = eval_kind_expr(right, env, span)?;

    Ok(Kind::Function(left.into(), right.into()))
}
