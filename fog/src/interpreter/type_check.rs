use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval_type::Annotation;
use crate::interpreter::eval_type::eval_annotation_expr;
use crate::interpreter::eval_type::eval_type_annotation_expr;
use crate::interpreter::eval_type::eval_type_definition_expr;
use crate::interpreter::r#type::Type;
use crate::interpreter::r#type::Type::Product;
use crate::parser::desugared_expr::DesugaredDeclPattern;
use crate::parser::desugared_expr::DesugaredExpr;
use crate::parser::desugared_expr::DesugaredStatement;
use crate::runtime_error;
use crate::type_check_error;

pub fn expr_type_of(expr: &DesugaredExpr, env: &Environment) -> FogResult<Type> {
    let span = expr.span();

    match expr {
        DesugaredExpr::Block { statements, .. } => {
            let mut block_env = Environment::new(Some(env));

            for stmt in statements {
                match stmt {
                    DesugaredStatement::TypeAnnotation { name, expr, span } => {
                        match eval_annotation_expr(expr, &block_env)? {
                            Annotation::Kind(kind) => block_env.annotate_kind(name, kind, span)?,
                            Annotation::Type(r#type) => {
                                block_env.annotate_type(name, r#type, span)?
                            }
                        };
                    }

                    DesugaredStatement::Declaration {
                        pattern,
                        expr,
                        span,
                    } => check_pattern_declaration(pattern, expr, span, &mut block_env)?,

                    DesugaredStatement::Expression { expr, .. } => {
                        return expr_type_of(expr, &block_env);
                    }
                }
            }

            Err(runtime_error!(
                Some(span),
                "final operand not found in block statement"
            ))
        }

        DesugaredExpr::Identifier { name, .. } => Ok(env.get_value_var(name, &span)?.r#type),

        DesugaredExpr::Int32Literal { .. } => Ok(Type::Int32),
        DesugaredExpr::Float32Literal { .. } => Ok(Type::Float32),

        DesugaredExpr::Lambda {
            param_type, body, ..
        } => Ok(Type::Function(
            eval_type_annotation_expr(param_type, env)?.into(),
            expr_type_of(body, env)?.into(),
        )),

        DesugaredExpr::FuncAppl { fn_name, args, .. } => {
            let mut curr_type = env.get_value_var(fn_name, &span)?.r#type.clone();

            for _ in args {
                curr_type = match curr_type {
                    Type::Function(_, return_type) => *return_type,
                    _ => {
                        return Err(runtime_error!(
                            Some(span),
                            "{} is not a function type",
                            curr_type.to_string()
                        ));
                    }
                };
            }

            Ok(curr_type)
        }

        DesugaredExpr::Tuple { items, .. } => Ok(Product(
            items
                .iter()
                .map(|expr| expr_type_of(expr, env))
                .collect::<Result<Vec<Type>, FogError>>()?,
        )),

        DesugaredExpr::Match { match_arms, .. } => match match_arms.first() {
            Some(arm) => expr_type_of(&arm.value_expr, env),
            None => Err(runtime_error!(Some(span), "match with no arms")),
        },
    }
}

fn check_pattern_declaration(
    pattern: &DesugaredDeclPattern,
    expr: &DesugaredExpr,
    span: &Span,
    block_env: &mut Environment,
) -> FogResult<()> {
    match pattern {
        DesugaredDeclPattern::Identifier { name, .. } => {
            if block_env.variables.contains_key(name) {
                let expr_type = expr_type_of(expr, block_env)?;
                let annotated_type = block_env.variables[name].r#type.clone();

                if expr_type != annotated_type {
                    return Err(type_check_error!(
                        Some(span.clone()),
                        "type mismatch when assigning variable `{expr}` with `{name}`\n\
                         expected `{annotated_type}`, found `{expr_type}`"
                    ));
                }
            } else if block_env.types.contains_key(name) {
                let defined_type = eval_type_definition_expr(expr, block_env)?;
                block_env.declare_type(name, defined_type, span)?;
            } else {
                return Err(runtime_error!(
                    Some(span.clone()),
                    "unannotated variable `{}`",
                    name
                ));
            }
        }

        DesugaredDeclPattern::Tuple { .. } => {
            let expected_type = expected_type_of_pattern(pattern, block_env, span)?;
            let expr_type = expr_type_of(expr, block_env)?;

            if expr_type != expected_type {
                return Err(type_check_error!(
                    Some(span.clone()),
                    "type mismatch when assigning variable `{expr}` with `{pattern}`\n\
                     expected `{expected_type}`, found `{expr_type}`"
                ));
            }
        }
    }

    Ok(())
}

fn expected_type_of_pattern(
    pattern: &DesugaredDeclPattern,
    block_env: &Environment,
    span: &Span,
) -> FogResult<Type> {
    match pattern {
        DesugaredDeclPattern::Identifier { name, .. } => block_env
            .variables
            .get(name)
            .map(|var| var.r#type.clone())
            .ok_or_else(|| runtime_error!(Some(span.clone()), "unannotated variable `{}`", name)),

        DesugaredDeclPattern::Tuple { items, .. } => Ok(Product(
            items
                .iter()
                .map(|item| expected_type_of_pattern(item, block_env, span))
                .collect::<FogResult<Vec<_>>>()?,
        )),
    }
}
