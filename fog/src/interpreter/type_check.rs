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
use crate::parser::resolved_expr::ResolvedExpr;
use crate::parser::resolved_expr::ResolvedStatement;
use crate::runtime_error;
use crate::type_check_error;

pub fn expr_type_of(expr: &ResolvedExpr, env: &Environment, span: &Span) -> FogResult<Type> {
    match expr {
        ResolvedExpr::Block { statements } => {
            let mut block_env = Environment::new(Some(env));

            for stmt in statements {
                match stmt {
                    ResolvedStatement::TypeAnnotation { name, expr, span } => {
                        match eval_annotation_expr(expr, &block_env, span)? {
                            Annotation::Kind(kind) => block_env.annotate_kind(name, kind, span)?,
                            Annotation::Type(r#type) => {
                                block_env.annotate_type(name, r#type, span)?
                            }
                        };
                    }

                    ResolvedStatement::Declaration { name, expr, span } => {
                        if block_env.variables.contains_key(name) {
                            let expr_type = expr_type_of(expr, &block_env, span)?;
                            let annotated_type = block_env.variables[name].r#type.clone();

                            if expr_type != annotated_type {
                                return Err(type_check_error!(
                                    Some(span.clone()),
                                    "type mismatch when assigning variable `{expr}` with `{name}`\n\
                                     expected `{annotated_type}`, found `{expr_type}`"
                                ));
                            }
                        } else if block_env.types.contains_key(name) {
                            let defined_type = eval_type_definition_expr(expr, &block_env, span)?;
                            block_env.declare_type(name, defined_type, span)?;
                        } else {
                            return Err(runtime_error!(
                                Some(span.clone()),
                                "unannotated variable `{}`",
                                name
                            ));
                        }
                    }

                    ResolvedStatement::Expression { span, expr } => {
                        return expr_type_of(expr, &block_env, span);
                    }
                }
            }

            Err(runtime_error!(
                Some(span.clone()),
                "final operand not found in block statement"
            ))
        }

        ResolvedExpr::Identifier { name } => Ok(env.get_value_var(name, span)?.r#type),

        ResolvedExpr::Int32Literal { .. } => Ok(Type::Int32),
        ResolvedExpr::Float32Literal { .. } => Ok(Type::Float32),

        ResolvedExpr::Lambda {
            param_type, body, ..
        } => Ok(Type::Function(
            eval_type_annotation_expr(param_type, env, span)?.into(),
            expr_type_of(body, env, span)?.into(),
        )),

        ResolvedExpr::FuncAppl { fn_name, args } => {
            let mut curr_type = env.get_value_var(fn_name, span)?.r#type.clone();

            for _ in args {
                curr_type = match curr_type {
                    Type::Function(_, return_type) => *return_type,
                    _ => {
                        return Err(runtime_error!(
                            Some(span.clone()),
                            "{} is not a function type",
                            curr_type.to_string()
                        ));
                    }
                };
            }

            Ok(curr_type)
        }

        ResolvedExpr::Tuple { items } => Ok(Product(
            items
                .iter()
                .map(|expr| expr_type_of(expr, env, span))
                .collect::<Result<Vec<Type>, FogError>>()?,
        )),

        ResolvedExpr::Match { match_arms, .. } => match match_arms.first() {
            Some(arm) => expr_type_of(&arm.value_expr, env, span),
            None => Err(runtime_error!(Some(span.clone()), "match with no arms")),
        },
    }
}
