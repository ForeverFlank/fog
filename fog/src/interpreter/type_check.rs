use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval_type::Annotation;
use crate::interpreter::eval_type::eval_annotation_expr;
use crate::interpreter::eval_type::eval_type_annotation_expr;
use crate::interpreter::eval_type::eval_type_definition_expr;
use crate::interpreter::r#type::Type;
use crate::parser::Literal;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreStatement;
use crate::parser::core_expr::DesugaredTupleDeclPattern;
use crate::runtime_error;
use crate::type_check_error;

pub fn expr_type_of(expr: &CoreExpr, env: &Environment) -> FogResult<Type> {
    let span = expr.span();

    match expr {
        CoreExpr::Block { statements, .. } => block_expr_type_of(env, span, statements),

        CoreExpr::Identifier { name, .. } => Ok(env.get_value_var(name, &span)?.r#type),

        CoreExpr::Literal { literal, .. } => match literal {
            Literal::Int32(_) => Ok(Type::Int32),
            Literal::Float32(_) => Ok(Type::Float32),
        },

        CoreExpr::Lambda {
            param_type, body, ..
        } => Ok(Type::Function(
            eval_type_annotation_expr(param_type, env)?.into(),
            expr_type_of(body, env)?.into(),
        )),

        CoreExpr::FunctionAppl { fn_name, args, .. } => {
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

        CoreExpr::Tuple { items, .. } => Ok(Type::Product(
            items
                .iter()
                .map(|expr| expr_type_of(expr, env))
                .collect::<Result<Vec<Type>, FogError>>()?,
        )),

        CoreExpr::Match { match_arms, .. } => match match_arms.first() {
            Some(arm) => expr_type_of(&arm.value_expr, env),
            None => Err(runtime_error!(Some(span), "match with no arms")),
        },
    }
}

fn block_expr_type_of(
    env: &Environment<'_>,
    span: Span,
    statements: &Vec<CoreStatement>,
) -> Result<Type, FogError> {
    let mut block_env = Environment::new(Some(env));

    for stmt in statements {
        match stmt {
            CoreStatement::TypeAnnotation { name, expr, span } => {
                match eval_annotation_expr(expr, &block_env)? {
                    Annotation::Kind(kind) => block_env.annotate_kind(name, kind, span)?,
                    Annotation::Type(r#type) => block_env.annotate_type(name, r#type, span)?,
                };
            }

            CoreStatement::Declaration {
                pattern,
                expr,
                span,
            } => type_check_declaration(pattern, expr, span, &mut block_env)?,

            CoreStatement::Expression { expr, .. } => {
                return expr_type_of(expr, &block_env);
            }
        }
    }

    Err(runtime_error!(
        Some(span),
        "final operand not found in block statement"
    ))
}

// validate and sometimes annotate types
// of declaration statements
fn type_check_declaration(
    pattern: &CoreDeclPattern,
    expr: &CoreExpr,
    span: &Span,
    env: &mut Environment,
) -> FogResult<()> {
    match pattern {
        CoreDeclPattern::Identifier { name, .. } => {
            // check if declaration is a type declaration
            if env.types.contains_key(name) {
                let defined_type = eval_type_definition_expr(expr, env)?;
                return env.declare_type(name, defined_type, span);
            }

            // if not, it's a variable declaration
            let expr_type = expr_type_of(expr, env)?;

            // check if variable have its type annotated before
            if let Some(var) = env.variables.get(name) {
                // if so, check if declaration is actually type-valid
                let annotated_type = var.r#type.clone();

                if expr_type != annotated_type {
                    return Err(type_check_error!(
                        Some(*span),
                        "type mismatch when assigning variable `{expr}` with `{pattern}`\n\
                         expected `{annotated_type}`, found `{expr_type}`"
                    ));
                }

                return Ok(());
            }

            // otherwise, infer the type
            env.annotate_type(name, expr_type, span)
        }

        CoreDeclPattern::Tuple { items, .. } => {
            let expr_type = expr_type_of(expr, env)?;
            bind_tuple_decl_pattern(items, &expr_type, pattern, expr, span, env)
        }
    }
}

// like type_check_declaration but
// it iterates through a possibly nested tuple
fn bind_tuple_decl_pattern(
    items: &Vec<DesugaredTupleDeclPattern>,
    expr_type: &Type,
    pattern: &CoreDeclPattern,
    expr: &CoreExpr,
    span: &Span,
    env: &mut Environment,
) -> FogResult<()> {
    let Type::Product(component_types) = expr_type else {
        return Err(type_check_error!(
            Some(*span),
            "type mismatch when assigning variable `{expr}` with `{pattern}`\n\
             expected a tuple type, found `{expr_type}`"
        ));
    };

    if items.len() != component_types.len() {
        return Err(type_check_error!(
            Some(*span),
            "type mismatch when assigning variable `{expr}` with `{pattern}`\n\
             expected a tuple of {} element(s), found `{expr_type}`",
            items.len()
        ));
    }

    for (item, component_type) in items.iter().zip(component_types) {
        bind_tuple_decl_pattern_item(item, component_type, span, env)?;
    }

    Ok(())
}

fn bind_tuple_decl_pattern_item(
    item: &DesugaredTupleDeclPattern,
    expected_type: &Type,
    span: &Span,
    block_env: &mut Environment,
) -> FogResult<()> {
    match item {
        DesugaredTupleDeclPattern::Identifier { name, .. } => {
            if let Some(var) = block_env.variables.get(name) {
                let annotated_type = var.r#type.clone();

                if annotated_type != *expected_type {
                    return Err(type_check_error!(
                        Some(*span),
                        "type mismatch when binding variable `{name}`\n\
                         expected `{annotated_type}`, found `{expected_type}`"
                    ));
                }

                return Ok(());
            }

            // infer type
            block_env.annotate_type(name, expected_type.clone(), span)
        }

        DesugaredTupleDeclPattern::Tuple { items, .. } => {
            let Type::Product(component_types) = expected_type else {
                return Err(type_check_error!(
                    Some(*span),
                    "expected a tuple type, found `{expected_type}`"
                ));
            };

            if items.len() != component_types.len() {
                return Err(type_check_error!(
                    Some(*span),
                    "expected a tuple of {} element(s), found `{expected_type}`",
                    items.len()
                ));
            }

            for (item, component_type) in items.iter().zip(component_types) {
                bind_tuple_decl_pattern_item(item, component_type, span, block_env)?;
            }

            Ok(())
        }
    }
}
