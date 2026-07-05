use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::parser::Literal;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreStatement;
use crate::parser::core_expr::CoreTupleDeclPattern;
use crate::static_check::environment::Environment;
use crate::static_check::eval_type::Annotation;
use crate::static_check::eval_type::eval_annotation_expr;
use crate::static_check::eval_type::eval_type_annotation_expr;
use crate::static_check::eval_type::eval_type_definition_expr;
use crate::static_check::r#type::Type;
use crate::static_check::variable::ValueVariable;
use crate::type_check_error;

// --- type check ---

pub fn check(stmts: &Vec<CoreStatement>) -> Vec<FogError> {
    let mut top_env = create_top_env();
    let mut all_errors = Vec::new();

    check_scope(stmts, &mut top_env, &mut all_errors);

    all_errors
}

fn create_top_env() -> Environment<'static> {
    let mut env = Environment::new(None);

    let var_add_int32 = ValueVariable::new(
        "addInt32",
        Type::function(Type::Int32, Type::function(Type::Int32, Type::Int32)),
        true,
    );

    let var_subtract_int32 = ValueVariable::new(
        "subtractInt32",
        Type::function(Type::Int32, Type::function(Type::Int32, Type::Int32)),
        true,
    );

    vec![var_add_int32, var_subtract_int32]
        .into_iter()
        .for_each(|var| {
            env.variables.insert(var.name.clone(), var);
        });

    env
}

fn check_scope(stmts: &Vec<CoreStatement>, env: &mut Environment, all_errors: &mut Vec<FogError>) {
    for stmt in stmts {
        check_statement(stmt, env, all_errors);
    }
}

fn check_statement(stmt: &CoreStatement, env: &mut Environment, all_errors: &mut Vec<FogError>) {
    let result = match stmt {
        CoreStatement::TypeAnnotation { name, expr, span } => {
            check_type_annotation(name, expr, span, env)
        }

        CoreStatement::Declaration {
            pattern,
            expr,
            span,
        } => check_declaration(pattern, expr, span, env),

        CoreStatement::Expression { expr, .. } => expr_type_of(expr, env).map(|_| ()),
    };

    if let Err(error) = result {
        all_errors.push(error);
    }
}

fn check_type_annotation(
    name: &str,
    expr: &CoreExpr,
    span: &Span,
    env: &mut Environment,
) -> FogResult<()> {
    match eval_annotation_expr(expr, env)? {
        Annotation::Kind(kind) => env.annotate_kind(name, kind, span),
        Annotation::Type(r#type) => env.annotate_type(name, r#type, span),
    }
}

// validate and sometimes annotate types
// of declaration statements
fn check_declaration(
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

            env.declare(name, expr_type, span)
        }

        CoreDeclPattern::Tuple { items, .. } => {
            let expr_type = expr_type_of(expr, env)?;
            bind_tuple_decl_pattern(items, &expr_type, pattern, expr, span, env)
        }
    }
}

// like check_declaration but
// it iterates through a possibly nested tuple
fn bind_tuple_decl_pattern(
    items: &Vec<CoreTupleDeclPattern>,
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
    item: &CoreTupleDeclPattern,
    expected_type: &Type,
    span: &Span,
    block_env: &mut Environment,
) -> FogResult<()> {
    match item {
        CoreTupleDeclPattern::Identifier { name, .. } => {
            block_env.declare(name, expected_type.clone(), span)
        }

        CoreTupleDeclPattern::Tuple { items, .. } => {
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

// --- type of ---

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
                        return Err(type_check_error!(
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
            None => Err(type_check_error!(Some(span), "match with no arms")),
        },
    }
}

fn block_expr_type_of(
    env: &Environment<'_>,
    span: Span,
    statements: &Vec<CoreStatement>,
) -> FogResult<Type> {
    let mut block_env = Environment::new(Some(env));

    for stmt in statements {
        match stmt {
            CoreStatement::TypeAnnotation { name, expr, span } => {
                check_type_annotation(name, expr, span, &mut block_env)?;
            }

            CoreStatement::Declaration {
                pattern,
                expr,
                span,
            } => check_declaration(pattern, expr, span, &mut block_env)?,

            CoreStatement::Expression { expr, .. } => {
                return expr_type_of(expr, &block_env);
            }
        }
    }

    Err(type_check_error!(
        Some(span),
        "final operand not found in block statement"
    ))
}
