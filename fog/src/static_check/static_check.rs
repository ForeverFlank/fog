use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::parser::Literal;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreStatement;
use crate::parser::core_expr::CoreTupleDeclPattern;
use crate::parser::core_expr::CoreTypeAtomExpr;
use crate::parser::core_expr::CoreTypeExpr;
use crate::static_check::environment::Environment;
use crate::static_check::eval::eval_atomic_type_expr;
use crate::static_check::eval::eval_kind_expr;
use crate::static_check::eval::eval_type_expr;
use crate::static_check::eval::register_data_constructors;
use crate::static_check::kind::Kind;
use crate::static_check::r#type::Type;
use crate::static_check::r#type::kind_of;
use crate::static_check::variable::TypeVariable;
use crate::static_check::variable::ValueVariable;
use crate::static_check_error;

// --- type check ---

pub fn check(stmts: &Vec<CoreStatement>) -> Vec<FogError> {
    let mut top_env = create_top_env();
    let mut all_errors = Vec::new();

    check_scope(stmts, &mut top_env, &mut all_errors);

    all_errors
}

fn create_top_env() -> Environment<'static> {
    let mut env = Environment::new(None);

    env.types.insert(
        "Int32".to_string(),
        TypeVariable {
            name: "Int32".to_string(),
            r#type: Some(Type::Int32),
            kind: Kind::Type,
        },
    );

    env.types.insert(
        "Unit".to_string(),
        TypeVariable {
            name: "Unit".to_string(),
            r#type: Some(Type::Product(Vec::new())),
            kind: Kind::Type,
        },
    );

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
    // type kind annotations
    for stmt in stmts {
        if let CoreStatement::KindAnnotation { name, expr, span } = stmt {
            match eval_kind_expr(expr) {
                Ok(kind) => {
                    if let Err(error) = env.annotate_kind(name, kind, span) {
                        all_errors.push(error);
                    }
                }
                Err(error) => all_errors.push(error),
            }
        }
    }

    // type declarations
    for stmt in stmts {
        if let CoreStatement::TypeDeclaration { name, expr, span } = stmt {
            if let Err(error) = check_type_declaration(name, expr, span, env) {
                all_errors.push(error);
            }
        }
    }

    // variable type annotations
    for stmt in stmts {
        if let CoreStatement::TypeAnnotation { name, expr, span } = stmt {
            if let Err(error) = check_type_annotation(name, expr, span, env) {
                all_errors.push(error);
            }
        }
    }

    // variable declarations
    for stmt in stmts {
        if let CoreStatement::VarDeclaration {
            pattern,
            expr,
            span,
        } = stmt
        {
            if let Err(error) = check_declaration(pattern, expr, span, env) {
                all_errors.push(error);
            }
        }
    }

    // expressions (e.g. top-level statements without a binding)
    for stmt in stmts {
        if let CoreStatement::Expression { expr, .. } = stmt {
            if let Err(error) = expr_type_of(expr, env) {
                all_errors.push(error);
            }
        }
    }
}

fn check_type_declaration(
    name: &str,
    expr: &CoreTypeExpr,
    span: &Span,
    env: &mut Environment,
) -> FogResult<()> {
    let defined_type = eval_type_expr(expr, env)?;

    if !env.types.contains_key(name) {
        env.annotate_kind(name, kind_of(&defined_type), span)?;
    }

    env.declare_type(name, defined_type.clone(), span)?;

    if let Type::Sum(_) = &defined_type {
        register_data_constructors(env, &defined_type, span)?;
    }

    Ok(())
}

fn check_type_annotation(
    name: &str,
    expr: &CoreTypeAtomExpr,
    span: &Span,
    env: &mut Environment,
) -> FogResult<()> {
    let r#type = eval_atomic_type_expr(expr, env)?;
    env.annotate_type(name, r#type, span)
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
            let expr_type = expr_type_of(expr, env)?;
            env.declare_var(name, expr_type, span)
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
        return Err(static_check_error!(
            Some(*span),
            "type mismatch when assigning variable `{expr}` with `{pattern}`\n\
             expected a tuple type, found `{expr_type}`"
        ));
    };

    if items.len() != component_types.len() {
        return Err(static_check_error!(
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
            block_env.declare_var(name, expected_type.clone(), span)
        }

        CoreTupleDeclPattern::Tuple { items, .. } => {
            let Type::Product(component_types) = expected_type else {
                return Err(static_check_error!(
                    Some(*span),
                    "expected a tuple type, found `{expected_type}`"
                ));
            };

            if items.len() != component_types.len() {
                return Err(static_check_error!(
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
            eval_atomic_type_expr(param_type, env)?.into(),
            expr_type_of(body, env)?.into(),
        )),

        CoreExpr::FunctionAppl { callee, .. } => {
            let callee_type = expr_type_of(callee, env)?;

            match callee_type {
                Type::Function(_, return_type) => Ok(*return_type),
                _ => Err(static_check_error!(
                    Some(span),
                    "{} is not a function type",
                    callee_type.to_string()
                )),
            }
        }

        CoreExpr::Tuple { items, .. } => Ok(Type::Product(
            items
                .iter()
                .map(|expr| expr_type_of(expr, env))
                .collect::<Result<Vec<Type>, FogError>>()?,
        )),

        CoreExpr::Match { arms, .. } => match arms.first() {
            Some(arm) => expr_type_of(&arm.value_expr, env),
            None => Err(static_check_error!(Some(span), "match with no arms")),
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
            CoreStatement::KindAnnotation { name, expr, span } => {
                let kind = eval_kind_expr(expr)?;
                block_env.annotate_kind(name, kind, span)?;
            }

            CoreStatement::TypeDeclaration { name, expr, span } => {
                check_type_declaration(name, expr, span, &mut block_env)?;
            }

            CoreStatement::TypeAnnotation { name, expr, span } => {
                check_type_annotation(name, expr, span, &mut block_env)?;
            }

            CoreStatement::VarDeclaration {
                pattern,
                expr,
                span,
            } => check_declaration(pattern, expr, span, &mut block_env)?,

            CoreStatement::Expression { expr, .. } => {
                return expr_type_of(expr, &block_env);
            }
        }
    }

    Err(static_check_error!(
        Some(span),
        "final operand not found in block statement"
    ))
}
