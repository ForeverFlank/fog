use std::vec;

use crate::error::FogResult;
use crate::parser::core_expr::CoreAtomicTypeExpr;
use crate::parser::core_expr::CoreDataConstructor;
use crate::parser::core_expr::CoreKindExpr;
use crate::parser::core_expr::CoreTypeExpr;
use crate::static_check::environment::Environment;
use crate::static_check::kind::Kind;
use crate::static_check::r#type::DataConstructor;
use crate::static_check::r#type::Type;
use crate::static_check_error;

// --- kind ---

pub fn eval_kind_expr(expr: &CoreKindExpr) -> FogResult<Kind> {
    match expr {
        CoreKindExpr::Type { .. } => Ok(Kind::Type),

        CoreKindExpr::Constraint { .. } => Ok(Kind::Constraint),

        CoreKindExpr::Function {
            param_kind,
            return_kind,
            ..
        } => Ok(Kind::Function(
            eval_kind_expr(&param_kind)?.into(),
            eval_kind_expr(&return_kind)?.into(),
        )),
    }
}

pub fn eval_type_expr(
    name: &str,
    params: &Vec<String>,
    expr: &CoreTypeExpr,
    env: &Environment,
) -> FogResult<(Type, Vec<DataConstructor>)> {
    let (r#type, ctors) = match expr {
        CoreTypeExpr::Atomic(expr) => (eval_atomic_type_expr(expr, env)?, vec![]),

        CoreTypeExpr::Sum { ctors, .. } => {
            let named_type = Type::Named(
                name.to_string(),
                params
                    .iter()
                    .map(|param| Type::Variable(param.to_string()))
                    .collect(),
            );

            let ctors = ctors
                .iter()
                .map(|ctor| eval_data_constructor(ctor))
                .collect::<Result<Vec<_>, _>>()?;

            (named_type, ctors)
        }
    };

    let wrapped_type = wrap_forall_type(&r#type, params);

    if params.is_empty() {
        Ok((wrapped_type, ctors))
    } else {
        let mut res_type = wrapped_type;

        for param in params {
            res_type = Type::TypeConstructor(param.to_string(), res_type.into())
        }

        Ok((res_type, ctors))
    }
}

fn eval_data_constructor(ctor: &CoreDataConstructor) -> FogResult<DataConstructor> {
    Ok(DataConstructor {
        tag: ctor.tag.clone(),
        types: ctor.types.clone(),
    })
}

fn wrap_forall_type(r#type: &Type, params: &Vec<String>) -> Type {
    let mut vars = Vec::new();
    find_type_variables(r#type, &mut vars, params);

    if vars.is_empty() {
        r#type.clone()
    } else {
        Type::ForAll(vars, r#type.clone().into())
    }
}

fn find_type_variables(r#type: &Type, vars: &mut Vec<String>, params: &Vec<String>) {
    match r#type {
        Type::Int32
        | Type::Float32
        | Type::Char
        | Type::String
        | Type::IOUnit
        | Type::ForAll(_, _) => {}

        Type::Variable(name) => {
            if !params.contains(name) {
                vars.push(name.to_string());
            }
        }

        Type::Function(param_type, return_type) => {
            find_type_variables(param_type, vars, params);
            find_type_variables(return_type, vars, params);
        }

        Type::Product(types) | Type::Named(_, types) => types
            .iter()
            .for_each(|t| find_type_variables(t, vars, params)),

        Type::TypeConstructor(_, r#type) => {
            find_type_variables(r#type, vars, params);
        }
    }
}

pub fn eval_atomic_type_expr(expr: &CoreAtomicTypeExpr, env: &Environment) -> FogResult<Type> {
    let span = expr.span();

    match expr {
        CoreAtomicTypeExpr::Identifier { name, .. } => {
            let first_ch = name.chars().next().unwrap();

            if first_ch.is_lowercase() {
                Ok(Type::Variable(name.to_string()))
            } else if let Some(r#type) = env.get_type_var(name, &span)?.r#type {
                Ok(r#type)
            } else {
                Err(static_check_error!(Some(span), "undeclared type `{name}`"))
            }
        }

        CoreAtomicTypeExpr::Function {
            param_type,
            return_type,
            ..
        } => {
            let param_type = eval_atomic_type_expr(param_type, env)?;
            let return_type = eval_atomic_type_expr(return_type, env)?;

            Ok(Type::Function(param_type.into(), return_type.into()))
        }

        CoreAtomicTypeExpr::Product { types, .. } => {
            let types = types
                .into_iter()
                .map(|t| eval_atomic_type_expr(t, env))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Type::Product(types))
        }

        CoreAtomicTypeExpr::FunctionAppl { callee, arg, span } => {
            let callee_type = eval_atomic_type_expr(callee, env)?;

            let Type::TypeConstructor(param, r#type) = callee_type else {
                return Err(static_check_error!(
                    Some(*span),
                    "`{}` is not a valid type constructor",
                    callee_type.to_string()
                ));
            };

            let arg_type = eval_atomic_type_expr(arg, env)?;
            let res_type = (*r#type).substitute_var(&param, &arg_type);

            Ok(res_type)
        }
    }
}

// --- tests ---

#[cfg(test)]
mod tests {
    use super::*;

    use crate::error::Pos;
    use crate::error::Span;
    use crate::static_check::variable::TypeVariable;

    const SPAN: Span = Span {
        start: Pos {
            line: 0usize,
            column: 0usize,
        },
        end: Pos {
            line: 0usize,
            column: 0usize,
        },
    };

    #[test]
    fn test_eval_kind_expr() {
        let expr_1 = CoreKindExpr::Type { span: SPAN };
        assert!(matches!(eval_kind_expr(&expr_1), Ok(Kind::Type)));

        let expr_2 = CoreKindExpr::Constraint { span: SPAN };
        assert!(matches!(eval_kind_expr(&expr_2), Ok(Kind::Constraint)));

        let expr_3 = CoreKindExpr::Function {
            param_kind: CoreKindExpr::Type { span: SPAN }.into(),
            return_kind: CoreKindExpr::Type { span: SPAN }.into(),
            span: SPAN,
        };
        assert!(matches!(
            eval_kind_expr(&expr_3),
            Ok(Kind::Function(p, r))
                if *p == Kind::Type &&
                   *r == Kind::Type
        ));
    }

    #[test]
    fn test_eval_type_expr() {
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

        // --------------------

        // Option a = Some a | None
        let expr_1 = CoreTypeExpr::Sum {
            ctors: vec![
                CoreDataConstructor {
                    tag: "Some".to_string(),
                    types: vec![CoreAtomicTypeExpr::Identifier {
                        name: "a".to_string(),
                        span: SPAN,
                    }],
                },
                CoreDataConstructor {
                    tag: "None".to_string(),
                    types: vec![],
                },
            ],
            span: SPAN,
        };
        let Ok((type_1, ctors_1)) = eval_type_expr("Option", &vec!["a".to_string()], &expr_1, &env)
        else {
            panic!()
        };

        assert!(matches!(type_1, Type::TypeConstructor(_, _)));
        assert!(ctors_1.len() == 2);
    }

    #[test]
    fn test_eval_atomic_type_expr() {
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

        // --------------------

        let expr_1 = CoreAtomicTypeExpr::Identifier {
            name: "Int32".to_string(),
            span: SPAN,
        };
        assert!(matches!(
            eval_atomic_type_expr(&expr_1, &env),
            Ok(Type::Int32)
        ));

        let expr_2 = CoreAtomicTypeExpr::Product {
            types: vec![
                CoreAtomicTypeExpr::Identifier {
                    name: "Int32".to_string(),
                    span: SPAN,
                },
                CoreAtomicTypeExpr::Identifier {
                    name: "Unit".to_string(),
                    span: SPAN,
                },
            ],
            span: SPAN,
        };
        assert!(matches!(
            eval_atomic_type_expr(&expr_2, &env),
            Ok(Type::Product(types))
                if types[0] == Type::Int32 &&
                   types[1] == Type::Product(Vec::new())
        ));

        let expr_3 = CoreAtomicTypeExpr::Function {
            param_type: CoreAtomicTypeExpr::Identifier {
                name: "Unit".to_string(),
                span: SPAN,
            }
            .into(),
            return_type: CoreAtomicTypeExpr::Identifier {
                name: "Int32".to_string(),
                span: SPAN,
            }
            .into(),
            span: SPAN,
        };
        assert!(matches!(
            eval_atomic_type_expr(&expr_3, &env),
            Ok(Type::Function(p, r))
                if *p == Type::Product(Vec::new()) &&
                   *r == Type::Int32
        ));
    }
}
