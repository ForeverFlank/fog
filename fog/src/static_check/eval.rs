use std::collections::BTreeSet;
use std::env::var;
use std::println;
use std::vec;

use crate::error::FogResult;
use crate::parser::core_expr::CoreAtomicTypeExpr;
use crate::parser::core_expr::CoreDataConstructor;
use crate::parser::core_expr::CoreKindExpr;
use crate::parser::core_expr::CoreTypeExpr;
use crate::static_check::environment::Environment;
use crate::static_check::kind::Kind;
use crate::static_check::r#type::DataConstructor;
use crate::static_check::r#type::Monotype;
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
) -> FogResult<(Type, Type, Vec<DataConstructor>)> {
    match expr {
        CoreTypeExpr::Atomic(expr) => {
            let r#type = eval_atomic_type_expr(expr, env)?;
            Ok((r#type.clone(), r#type, vec![]))
        }

        CoreTypeExpr::Sum { ctors, .. } => {
            let named_monotype = Monotype::Named(
                name.to_string(),
                params
                    .iter()
                    .map(|param| Monotype::Variable(param.to_string()))
                    .collect(),
            );

            let ctors = ctors
                .iter()
                .map(|ctor| eval_data_constructor(ctor))
                .collect::<Result<Vec<_>, _>>()?;

            let mut type_constructor = named_monotype.clone();

            for param in params {
                type_constructor =
                    Monotype::TypeConstructor(param.to_string(), type_constructor.into())
            }

            let type_constructor = wrap_type_scheme(&type_constructor, params);
            // let named_type = wrap_type_scheme(&named_monotype, params);
            // println!("1> {}", named_monotype);
            // println!("2> {}", type_constructor);

            Ok((type_constructor, Type::mono(named_monotype), ctors))
        }
    }
}

fn eval_data_constructor(ctor: &CoreDataConstructor) -> FogResult<DataConstructor> {
    Ok(DataConstructor {
        tag: ctor.tag.clone(),
        types: ctor.types.clone(),
    })
}

pub fn wrap_type_scheme(monotype: &Monotype, params: &Vec<String>) -> Type {
    let mut vars = BTreeSet::new();
    find_type_variables(monotype, &mut vars, params);

    Type {
        vars: vars.into_iter().collect(),
        monotype: monotype.clone(),
    }
}

fn find_type_variables(r#type: &Monotype, vars: &mut BTreeSet<String>, params: &Vec<String>) {
    match r#type {
        Monotype::Int32
        | Monotype::Float32
        | Monotype::Char
        | Monotype::String
        | Monotype::IOUnit => {}

        Monotype::Variable(name) => {
            if !params.contains(name) {
                vars.insert(name.to_string());
            }
        }

        Monotype::Function(param_type, return_type) => {
            find_type_variables(param_type, vars, params);
            find_type_variables(return_type, vars, params);
        }

        Monotype::Product(types) | Monotype::Named(_, types) => types
            .iter()
            .for_each(|t| find_type_variables(t, vars, params)),

        Monotype::TypeConstructor(_, r#type) => {
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
                Ok(Type::mono(Monotype::Variable(name.to_string())))
            } else if let Some(r#type) = env.get_type_var(name, &span)?.r#type {
                Ok(Type::mono(r#type))
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

            Ok(Type::function(&param_type, &return_type))
        }

        CoreAtomicTypeExpr::Product { types, .. } => {
            let types = types
                .into_iter()
                .map(|t| eval_atomic_type_expr(t, env))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Type::product(&types))
        }

        CoreAtomicTypeExpr::FunctionAppl { callee, arg, span } => {
            let callee_type = eval_atomic_type_expr(callee, env)?;

            let Monotype::TypeConstructor(param, r#type) = callee_type.monotype else {
                return Err(static_check_error!(
                    Some(*span),
                    "`{}` is not a valid type constructor",
                    callee_type.to_string()
                ));
            };

            let arg_type = eval_atomic_type_expr(arg, env)?;
            let res_type = (*r#type).substitute_var(&param, &arg_type.monotype);
            let res_type = Type::poly(callee_type.vars, res_type);

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
                r#type: Some(Monotype::Int32),
                kind: Kind::Type,
            },
        );

        env.types.insert(
            "Unit".to_string(),
            TypeVariable {
                name: "Unit".to_string(),
                r#type: Some(Monotype::Product(Vec::new())),
                kind: Kind::Type,
            },
        );

        // --------------------

        // Option a = Some a | None
        let _expr_1 = CoreTypeExpr::Sum {
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
    }

    #[test]
    fn test_eval_type_expr_sum_uses_named_type() {
        let mut env = Environment::new(None);

        env.types.insert(
            "Int32".to_string(),
            TypeVariable {
                name: "Int32".to_string(),
                r#type: Some(Monotype::Int32),
                kind: Kind::Type,
            },
        );

        let expr = CoreTypeExpr::Sum {
            ctors: vec![CoreDataConstructor {
                tag: "Some".to_string(),
                types: vec![CoreAtomicTypeExpr::Identifier {
                    name: "a".to_string(),
                    span: SPAN,
                }],
            }],
            span: SPAN,
        };

        let (type_constructor, named_type, _) =
            eval_type_expr("Option", &vec!["a".to_string()], &expr, &env).unwrap();

        assert!(matches!(
            type_constructor.monotype,
            Monotype::TypeConstructor(_, _)
        ));

        assert!(matches!(
            named_type.monotype,
            Monotype::Named(ref name, ref args)
                if name == "Option" && args.len() == 1 && args[0] == Monotype::Variable("a".to_string())
        ));
    }

    #[test]
    fn test_eval_atomic_type_expr() {
        let mut env = Environment::new(None);

        env.types.insert(
            "Int32".to_string(),
            TypeVariable {
                name: "Int32".to_string(),
                r#type: Some(Monotype::Int32),
                kind: Kind::Type,
            },
        );

        env.types.insert(
            "Unit".to_string(),
            TypeVariable {
                name: "Unit".to_string(),
                r#type: Some(Monotype::Product(Vec::new())),
                kind: Kind::Type,
            },
        );

        // --------------------

        let expr_1 = CoreAtomicTypeExpr::Identifier {
            name: "Int32".to_string(),
            span: SPAN,
        };
        assert_eq!(
            eval_atomic_type_expr(&expr_1, &env).unwrap(),
            Type::mono(Monotype::Int32)
        );

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
        let t2 = eval_atomic_type_expr(&expr_2, &env).unwrap();
        match t2.monotype {
            Monotype::Product(ref types) => {
                assert_eq!(types[0], Monotype::Int32);
                assert_eq!(types[1], Monotype::Product(Vec::new()));
            }
            _ => panic!(),
        }

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
        let t3 = eval_atomic_type_expr(&expr_3, &env).unwrap();
        match t3.monotype {
            Monotype::Function(ref p, ref r) => {
                assert_eq!(**p, Monotype::Product(Vec::new()));
                assert_eq!(**r, Monotype::Int32);
            }
            _ => panic!(),
        }
    }
}
