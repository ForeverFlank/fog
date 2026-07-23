use std::collections::HashMap;
use std::println;

use crate::core::get_static_check_types;
use crate::core::get_static_check_variables;
use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::parser::Literal;
use crate::parser::core_expr::CoreAtomicTypeExpr;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreMatchArmPattern;
use crate::parser::core_expr::CoreStatement;
use crate::parser::core_expr::CoreTupleDeclPattern;
use crate::parser::core_expr::CoreTypeExpr;
use crate::static_check::environment::Environment;
use crate::static_check::eval::eval_atomic_type_expr;
use crate::static_check::eval::eval_kind_expr;
use crate::static_check::eval::eval_type_expr;
use crate::static_check::eval::wrap_type_scheme;
use crate::static_check::r#type;
use crate::static_check::r#type::DataConstructor;
use crate::static_check::r#type::Monotype;
use crate::static_check::r#type::Type;
use crate::static_check::r#type::kind_of;
use crate::static_check_error;

// --- type check ---

pub fn check(stmts: &Vec<CoreStatement>) -> Vec<FogError> {
    let mut top_env = create_top_env();
    let mut all_errors = Vec::new();

    check_scope(stmts, &mut top_env, &mut all_errors);

    all_errors
}

pub fn create_top_env() -> Environment<'static> {
    let mut env = Environment::new(None);

    for r#type in get_static_check_types() {
        env.types.insert(r#type.name.clone(), r#type);
    }

    for var in get_static_check_variables() {
        env.variables.insert(var.name.clone(), var);
    }

    env
}

fn check_scope(stmts: &Vec<CoreStatement>, env: &mut Environment, all_errors: &mut Vec<FogError>) {
    let mut kind_annotations = Vec::new();
    let mut type_declarations = Vec::new();

    let mut type_annotations = Vec::new();
    let mut var_declarations = Vec::new();

    let mut tailing_operand = None;

    for stmt in stmts {
        match stmt {
            CoreStatement::KindAnnotation { .. } => kind_annotations.push(stmt),
            CoreStatement::TypeDeclaration { .. } => type_declarations.push(stmt),

            CoreStatement::TypeAnnotation { .. } => type_annotations.push(stmt),
            CoreStatement::VarDeclaration { .. } => var_declarations.push(stmt),

            CoreStatement::Expression { span, .. } => match tailing_operand {
                None => tailing_operand = Some(stmt),
                Some(_) => {
                    all_errors.push(static_check_error!(
                        Some(*span),
                        "only one expression is allowed in a scope"
                    ));
                }
            },
        }
    }

    // -- type kind annotations
    for stmt in kind_annotations {
        let CoreStatement::KindAnnotation { name, expr, span } = stmt else {
            unreachable!()
        };

        match eval_kind_expr(expr) {
            Ok(kind) => {
                if let Err(error) = env.annotate_kind(name, kind, span) {
                    all_errors.push(error);
                }
            }
            Err(error) => all_errors.push(error),
        }
    }

    // -- type declarations
    let mut data_ctors = Vec::new();

    for stmt in type_declarations {
        let CoreStatement::TypeDeclaration {
            name,
            params,
            expr,
            span,
        } = stmt
        else {
            unreachable!()
        };

        match check_type_declaration(name, params, expr, span, env) {
            Ok(item) => data_ctors.push(item),
            Err(error) => all_errors.push(error),
        }
    }

    for (parent_named_type, ctors, span) in data_ctors {
        if let Err(error) = register_data_constructors(env, &parent_named_type, &ctors, &span) {
            all_errors.push(error);
        }
    }

    // -- variable type annotations
    for stmt in type_annotations {
        let CoreStatement::TypeAnnotation { name, expr, span } = stmt else {
            unreachable!();
        };

        if let Err(error) = check_type_annotation(name, expr, span, env) {
            all_errors.push(error);
        }
    }

    // -- variable declarations
    for stmt in var_declarations {
        let CoreStatement::VarDeclaration {
            pattern,
            expr,
            span,
        } = stmt
        else {
            unreachable!();
        };

        if let Err(error) = check_declaration(pattern, expr, span, env, &mut HashMap::new()) {
            all_errors.push(error);
        }
    }

    // -- expressions
    if let Some(CoreStatement::Expression { expr, .. }) = tailing_operand {
        if let Err(error) = expr_type_of(expr, env, &mut HashMap::new()) {
            all_errors.push(error);
        }
    }
}

fn check_type_declaration(
    name: &str,
    params: &Vec<String>,
    expr: &CoreTypeExpr,
    span: &Span,
    env: &mut Environment,
) -> FogResult<(Type, Vec<DataConstructor>, Span)> {
    let (type_constructor, named_type, ctors) = eval_type_expr(name, params, expr, env)?;

    if !type_constructor.vars.is_empty() {
        return Err(static_check_error!(
            Some(*span),
            "declared type must be a monotype"
        ));
    }

    if !env.types.contains_key(name) {
        env.annotate_kind(name, kind_of(&type_constructor.monotype), span)?;
    }

    env.declare_type(name, type_constructor.monotype.clone(), span)?;
    // register_data_constructors(env, &r#type, &ctors, span)?;

    Ok((named_type, ctors, *span))
}

fn register_data_constructors(
    env: &mut Environment,
    parent_named_type: &Type,
    ctors: &Vec<DataConstructor>,
    span: &Span,
) -> FogResult<()> {
    for ctor in ctors {
        let types = ctor
            .types
            .clone()
            .into_iter()
            .map(|expr| eval_atomic_type_expr(&expr, env))
            .collect::<Result<Vec<_>, _>>()?;

        let ctor_type = nest_function_types(&types, parent_named_type.clone());
        let ctor_type = wrap_type_scheme(&ctor_type.monotype, &mut Vec::new());
        // println!(":: {}", ctor_type);

        env.annotate_type(&ctor.tag, ctor_type.clone(), span)?;
        env.declare_var(&ctor.tag, ctor_type, span)?;
    }

    Ok(())
}

fn nest_function_types(field_types: &Vec<Type>, return_type: Type) -> Type {
    field_types
        .iter()
        .rev()
        .fold(return_type, |ret, ft| Type::function(ft, &ret))
}

fn check_type_annotation(
    name: &str,
    expr: &CoreAtomicTypeExpr,
    span: &Span,
    env: &mut Environment,
) -> FogResult<()> {
    let r#type = eval_atomic_type_expr(expr, env)?;
    env.annotate_type(name, r#type, span)
}

fn check_declaration(
    pattern: &CoreDeclPattern,
    expr: &CoreExpr,
    span: &Span,
    env: &mut Environment,
    type_var_subst: &mut HashMap<String, Monotype>,
) -> FogResult<()> {
    match pattern {
        CoreDeclPattern::Identifier { name, .. } => {
            let expr_type = expr_type_of(expr, env, type_var_subst)?;
            env.declare_var(name, expr_type, span)
        }

        CoreDeclPattern::Tuple { items, .. } => {
            let expr_type = expr_type_of(expr, env, type_var_subst)?;
            bind_tuple_decl_pattern(items, &expr_type, pattern, expr, span, env)
        }
    }
}

// like check_declaration, but it iterates through a possibly nested tuple
fn bind_tuple_decl_pattern(
    items: &Vec<CoreTupleDeclPattern>,
    expr_type: &Type,
    pattern: &CoreDeclPattern,
    expr: &CoreExpr,
    span: &Span,
    env: &mut Environment,
) -> FogResult<()> {
    let Monotype::Product(ref types) = expr_type.monotype else {
        return Err(static_check_error!(
            Some(*span),
            "type mismatch when assigning variable `{expr}` with `{pattern}`\n\
             expected a tuple type, found `{expr_type}`"
        ));
    };

    if items.len() != types.len() {
        return Err(static_check_error!(
            Some(*span),
            "type mismatch when assigning variable `{expr}` with `{pattern}`\n\
             expected a tuple of {} element(s), found `{expr_type}`",
            items.len()
        ));
    }

    for (item, r#type) in items.iter().zip(types) {
        let expected_type = Type {
            vars: Vec::new(),
            monotype: r#type.clone(),
        };

        bind_tuple_decl_pattern_item(item, &expected_type, span, env)?;
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
            let Monotype::Product(ref types) = expected_type.monotype else {
                return Err(static_check_error!(
                    Some(*span),
                    "expected a tuple type, found `{expected_type}`"
                ));
            };

            if items.len() != types.len() {
                return Err(static_check_error!(
                    Some(*span),
                    "expected a tuple of {} element(s), found `{expected_type}`",
                    items.len()
                ));
            }

            for (item, r#type) in items.iter().zip(types) {
                let expected_type = Type {
                    vars: Vec::new(),
                    monotype: r#type.clone(),
                };

                bind_tuple_decl_pattern_item(item, &expected_type, span, block_env)?;
            }

            Ok(())
        }
    }
}

// --- type of ---

pub fn expr_type_of(
    expr: &CoreExpr,
    env: &mut Environment,
    type_var_subst: &mut HashMap<String, Monotype>,
) -> FogResult<Type> {
    let span = expr.span();

    match expr {
        CoreExpr::Block { statements, .. } => {
            block_expr_type_of(env, statements, type_var_subst, span)
        }

        CoreExpr::Identifier { name, .. } => Ok(env.get_value_var(name, &span)?.r#type),

        CoreExpr::Literal { literal, .. } => Ok(Type::mono(match literal {
            Literal::Int32(_) => Monotype::Int32,
            Literal::Float32(_) => Monotype::Float32,
            Literal::Char(_) => Monotype::Char,
            Literal::String(_) => Monotype::String,
        })),

        CoreExpr::Lambda {
            param_name,
            param_type,
            body,
            span,
        } => {
            let param_type = eval_atomic_type_expr(param_type, env)?;

            let mut body_env = Environment::new(Some(env));
            body_env.declare_var(param_name, param_type.clone(), &span)?;

            let return_type = expr_type_of(body, &mut body_env, type_var_subst)?;

            Ok(Type::function(&param_type, &return_type))
        }

        CoreExpr::FunctionAppl { callee, arg, span } => {
            let callee_type = expr_type_of(callee, env, type_var_subst)?;

            let Monotype::Function(param_type, return_type) = callee_type.monotype else {
                return Err(static_check_error!(
                    Some(*span),
                    "{} is not a function type",
                    callee_type
                ));
            };

            let arg_type = expr_type_of(arg, env, type_var_subst)?;

            unify_type(&*param_type, &arg_type.monotype, type_var_subst, span);

            Ok(Type::mono(substitute_types(&*return_type, type_var_subst)))
        }

        CoreExpr::Tuple { items, .. } => {
            let types = items
                .into_iter()
                .map(|item| expr_type_of(item, env, type_var_subst))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Type::product(&types))
        }

        CoreExpr::Match {
            scrutinee,
            arms,
            span,
        } => {
            let scrutinee_type = expr_type_of(scrutinee, env, type_var_subst)?;
            let mut res_type = None;

            for arm in arms {
                let mut arm_env = Environment::new(Some(env));
                bind_match_arm_pattern(&arm.pattern, &scrutinee_type, &mut arm_env)?;

                let arm_type = expr_type_of(&arm.value_expr, &mut arm_env, type_var_subst)?;

                match res_type {
                    None => res_type = Some(arm_type),

                    Some(res_type) if !can_unify(&arm_type.monotype, &res_type.monotype, span) => {
                        return Err(static_check_error!(
                            Some(*span),
                            "expected type `{}`, found `{}`",
                            res_type,
                            arm_type
                        ));
                    }

                    _ => {}
                }
            }

            if let Some(r#type) = res_type {
                Ok(r#type)
            } else {
                Err(static_check_error!(Some(*span), "match with no arms"))
            }
        }
    }
}

fn bind_match_arm_pattern(
    pattern: &CoreMatchArmPattern,
    expected_type: &Type,
    env: &mut Environment,
) -> FogResult<()> {
    match pattern {
        CoreMatchArmPattern::Literal { literal, span } => {
            let literal_type = match literal {
                Literal::Int32(_) => Monotype::Int32,
                Literal::Float32(_) => Monotype::Float32,
                Literal::Char(_) => Monotype::Char,
                Literal::String(_) => Monotype::String,
            };

            if !can_unify(&expected_type.monotype, &literal_type, span) {
                return Err(static_check_error!(
                    Some(*span),
                    "expected type `{}`, found `{}`",
                    expected_type,
                    literal_type
                ));
            }

            Ok(())
        }

        CoreMatchArmPattern::Identifier { name, span } => {
            if name == "_" {
                // wildcard
                Ok(())
            } else if name.starts_with(|c: char| c.is_uppercase()) {
                // nullary data constructor
                let r#type = env.get_value_var(name, span)?.r#type;

                // HACK
                if !can_unify(&expected_type.monotype, &r#type.monotype, span)
                    && !can_unify(&r#type.monotype, &expected_type.monotype, span)
                {
                    return Err(static_check_error!(
                        Some(*span),
                        "expected type `{}`, found `{}`",
                        expected_type,
                        r#type
                    ));
                }

                Ok(())
            } else {
                // bind value to identifier
                env.declare_var(name, expected_type.clone(), span)
            }
        }

        CoreMatchArmPattern::Tuple { items, span } => {
            let Monotype::Product(ref types) = expected_type.monotype else {
                return Err(static_check_error!(
                    Some(*span),
                    "expected a tuple type, found `{expected_type}`"
                ));
            };

            if items.len() != types.len() {
                return Err(static_check_error!(
                    Some(*span),
                    "expected a tuple of {} element(s), found `{expected_type}`",
                    items.len()
                ));
            }

            for (item, component_type) in items.iter().zip(types.iter()) {
                bind_match_arm_pattern(item, &Type::mono(component_type.clone()), env)?;
            }

            Ok(())
        }

        CoreMatchArmPattern::DataConstructor { name, args, span } => {
            let ctor_type = env.get_value_var(name, span)?.r#type;
            let (param_types, return_type) = uncurry_function_type(&ctor_type, args.len());

            // HACK
            if param_types.len() != args.len() {
                return Err(static_check_error!(
                    Some(*span),
                    "data constructor argument length mismatch"
                ));
            }

            // println!("---");
            // HACK too
            if !can_unify(&return_type.monotype, &expected_type.monotype, span)
                && !can_unify(&expected_type.monotype, &return_type.monotype, span)
            {
                return Err(static_check_error!(
                    Some(*span),
                    "expected type `{}`, found `{}`",
                    expected_type,
                    return_type
                ));
            }

            for (arg, param_type) in args.iter().zip(&param_types) {
                bind_match_arm_pattern(arg, param_type, env)?;
            }

            Ok(())
        }
    }
}

fn uncurry_function_type(r#type: &Type, arity: usize) -> (Vec<Type>, Type) {
    let mut param_types: Vec<Type> = Vec::new();
    let mut current = r#type.clone();

    for _ in 0..arity {
        match current.monotype {
            Monotype::Function(ref param_type, ref return_type) => {
                param_types.push(Type::mono((**param_type).clone()));
                current = Type::mono((**return_type).clone());
            }
            _ => break,
        }
    }

    (param_types, current)
}

fn block_expr_type_of(
    env: &Environment<'_>,
    statements: &Vec<CoreStatement>,
    type_var_subst: &mut HashMap<String, Monotype>,
    span: Span,
) -> FogResult<Type> {
    let mut block_env = Environment::new(Some(env));

    for stmt in statements {
        match stmt {
            CoreStatement::KindAnnotation { name, expr, span } => {
                let kind = eval_kind_expr(expr)?;
                block_env.annotate_kind(name, kind, span)?;
            }

            CoreStatement::TypeDeclaration {
                name,
                params,
                expr,
                span,
            } => {
                check_type_declaration(name, params, expr, span, &mut block_env)?;
            }

            CoreStatement::TypeAnnotation { name, expr, span } => {
                check_type_annotation(name, expr, span, &mut block_env)?;
            }

            CoreStatement::VarDeclaration {
                pattern,
                expr,
                span,
            } => check_declaration(pattern, expr, span, &mut block_env, type_var_subst)?,

            CoreStatement::Expression { expr, .. } => {
                return expr_type_of(expr, &mut block_env, type_var_subst);
            }
        }
    }

    Err(static_check_error!(
        Some(span),
        "final operand not found in block statement"
    ))
}

pub fn can_unify(to: &Monotype, from: &Monotype, span: &Span) -> bool {
    unify_type(to, from, &mut HashMap::new(), span)
}

pub fn unify_type(
    to: &Monotype,
    from: &Monotype,
    type_var_subst: &mut HashMap<String, Monotype>,
    span: &Span,
) -> bool {
    // println!("unifying {} and {}", to, from);

    if let Monotype::Variable(name_1) = to
        && let Monotype::Variable(name_2) = from
        && name_1 == name_2
    {
        return true;
    }

    let to = substitute_types(to, type_var_subst);
    let from = substitute_types(from, type_var_subst);

    match (&to, &from) {
        (Monotype::Variable(name), _) => {
            type_var_subst.insert(name.clone(), from);
            true
        }

        (Monotype::Function(p1, r1), Monotype::Function(p2, r2)) => {
            unify_type(p1, p2, type_var_subst, span) && unify_type(r1, r2, type_var_subst, span)
        }

        (Monotype::Product(types_1), Monotype::Product(types_2))
            if types_1.len() == types_2.len() =>
        {
            types_1
                .iter()
                .zip(types_2)
                .all(|(a, b)| unify_type(a, b, type_var_subst, span))
        }

        (Monotype::Named(name_1, types_1), Monotype::Named(name_2, types_2))
            if name_1 == name_2 && types_1.len() == types_2.len() =>
        {
            types_1
                .iter()
                .zip(types_2)
                .all(|(a, b)| unify_type(a, b, type_var_subst, span))
        }

        (Monotype::IO(type_1), Monotype::IO(type_2)) => {
            unify_type(&type_1, &type_2, type_var_subst, span)
        }

        _ if to == from => true,

        _ => false,
    }
}

// TODO infinite recursion!
fn substitute_types(r#type: &Monotype, type_var_subst: &HashMap<String, Monotype>) -> Monotype {
    match r#type {
        Monotype::Variable(name) => type_var_subst
            .get(name)
            .map(|t2| substitute_types(t2, type_var_subst))
            .unwrap_or(r#type.clone()),
        // .clone(),
        Monotype::Function(param_type, return_type) => Monotype::function(
            substitute_types(param_type, type_var_subst),
            substitute_types(return_type, type_var_subst),
        ),

        Monotype::Product(types) => Monotype::Product(
            types
                .iter()
                .map(|t| substitute_types(t, type_var_subst))
                .collect(),
        ),

        Monotype::Named(name, types) => Monotype::Named(
            name.clone(),
            types
                .iter()
                .map(|t| substitute_types(t, type_var_subst))
                .collect(),
        ),

        _ => r#type.clone(),
    }
}
