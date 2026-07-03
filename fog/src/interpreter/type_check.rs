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
use crate::parser::Literal;
use crate::parser::desugared_expr::DesugaredExpr;
use crate::parser::desugared_expr::DesugaredStatement;
use crate::runtime_error;
use crate::type_check_error;

pub fn expr_type_of(expr: &DesugaredExpr, env: &Environment) -> FogResult<Type> {
    let span = expr.span();

    match expr {
        DesugaredExpr::Block { statements, .. } => block_expr_type_of(env, span, statements),

        DesugaredExpr::Identifier { name, .. } => Ok(env.get_value_var(name, &span)?.r#type),

        DesugaredExpr::Literal { literal, .. } => match literal {
            Literal::Int32(_) => Ok(Type::Int32),
            Literal::Float32(_) => Ok(Type::Float32),
        },

        DesugaredExpr::Lambda {
            param_type, body, ..
        } => Ok(Type::Function(
            eval_type_annotation_expr(param_type, env)?.into(),
            expr_type_of(body, env)?.into(),
        )),

        DesugaredExpr::FunctionAppl { fn_name, args, .. } => {
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

fn block_expr_type_of(
    env: &Environment<'_>,
    span: Span,
    statements: &Vec<DesugaredStatement>,
) -> Result<Type, FogError> {
    let mut block_env = Environment::new(Some(env));

    for stmt in statements {
        match stmt {
            DesugaredStatement::TypeAnnotation { name, expr, span } => {
                match eval_annotation_expr(expr, &block_env)? {
                    Annotation::Kind(kind) => block_env.annotate_kind(name, kind, span)?,
                    Annotation::Type(r#type) => block_env.annotate_type(name, r#type, span)?,
                };
            }

            DesugaredStatement::Declaration {
                pattern,
                expr,
                span,
            } => {
                if block_env.variables.contains_key(pattern) {
                    let expr_type = expr_type_of(expr, &block_env)?;
                    let annotated_type = block_env.variables[pattern].r#type.clone();

                    if expr_type != annotated_type {
                        return Err(type_check_error!(
                            Some(*span),
                            "type mismatch when assigning variable `{expr}` with `{pattern}`\n\
                                     expected `{annotated_type}`, found `{expr_type}`"
                        ));
                    }
                } else if block_env.types.contains_key(pattern) {
                    let defined_type = eval_type_definition_expr(expr, &block_env)?;
                    block_env.declare_type(pattern, defined_type, span)?;
                } else {
                    return Err(runtime_error!(
                        Some(*span),
                        "unannotated variable `{}`",
                        pattern
                    ));
                }
            }

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
