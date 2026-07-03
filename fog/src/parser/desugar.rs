use std::collections::HashMap;
use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::parse_error;
use crate::parser::desugared_expr::DesugaredDeclPattern;
use crate::parser::desugared_expr::DesugaredExpr;
use crate::parser::desugared_expr::DesugaredMatchArm;
use crate::parser::desugared_expr::DesugaredMatchArmPattern;
use crate::parser::desugared_expr::DesugaredStatement;
use crate::parser::desugared_expr::DesugaredTupleDeclPattern;
use crate::parser::resolved_expr::ResolvedDeclPattern;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::parser::resolved_expr::ResolvedMatchArmPattern;
use crate::parser::resolved_expr::ResolvedStatement;
use crate::parser::resolved_expr::ResolvedTupleDeclPattern;

enum DesugarResult {
    Statement(DesugaredStatement),
    Declaration {
        pattern: ResolvedDeclPattern,
        expr: DesugaredExpr,
        span: Span,
    },
}

pub fn desugar(resolved_stmts: Vec<ResolvedStatement>) -> (Vec<DesugaredStatement>, Vec<FogError>) {
    desugar_statements(resolved_stmts)
}

fn desugar_block(
    resolved_stmts: Vec<ResolvedStatement>,
    span: Span,
) -> (FogResult<DesugaredExpr>, Vec<FogError>) {
    let (statements, errors) = desugar_statements(resolved_stmts);
    let block = DesugaredExpr::Block { statements, span };

    (Ok(block), errors)
}

fn desugar_statements(
    resolved_stmts: Vec<ResolvedStatement>,
) -> (Vec<DesugaredStatement>, Vec<FogError>) {
    let mut fn_decl_patterns: HashMap<String, Vec<(Vec<DesugaredMatchArmPattern>, DesugaredExpr)>> =
        HashMap::new();
    let mut statements = Vec::new();
    let mut errors = Vec::new();

    // process type annotations and expressions normally,
    // and collect pattern-based function declarations
    for resolved_stmt in resolved_stmts {
        let res = match desugar_statement(resolved_stmt) {
            Ok(res) => res,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };

        let (pattern, expr) = match res {
            DesugarResult::Statement(stmt) => {
                statements.push(stmt);
                continue;
            }
            DesugarResult::Declaration { pattern, expr, .. } => (pattern, expr),
        };

        match pattern {
            ResolvedDeclPattern::Identifier { name, span } => {
                statements.push(DesugaredStatement::Declaration {
                    pattern: DesugaredDeclPattern::Identifier { name, span },
                    expr,
                    span,
                });
            }

            ResolvedDeclPattern::Tuple { items, span } => {
                let items = match items
                    .into_iter()
                    .map(desugar_tuple_decl_pattern)
                    .collect::<Result<Vec<_>, _>>()
                {
                    Ok(items) => items,
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                };

                statements.push(DesugaredStatement::Declaration {
                    pattern: DesugaredDeclPattern::Tuple { items, span },
                    expr,
                    span,
                });
            }

            ResolvedDeclPattern::FunctionClause { name, items, .. } => {
                let desugared_items = match items
                    .into_iter()
                    .map(desugar_match_arm_pattern)
                    .collect::<Result<Vec<_>, _>>()
                {
                    Ok(items) => items,
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                };

                fn_decl_patterns
                    .entry(name)
                    .or_default()
                    .push((desugared_items, expr));
            }
        }
    }

    // combining those pattern-based function declarations
    // into one big lambda function assignment
    for (fn_name, patterns) in fn_decl_patterns {
        let span = patterns[0].0[0].span();
        let arity = patterns[0].0.len();

        if patterns.iter().any(|(items, _)| items.len() != arity) {
            errors.push(parse_error!(
                Some(span),
                "clauses of function `{fn_name}` don't all take the same number of arguments"
            ));
            continue;
        }

        let param_types = match find_fn_clause_param_types(&statements, &fn_name, arity, span) {
            Ok(param_types) => param_types,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };

        let match_arms = patterns
            .into_iter()
            .map(|(mut items, value_expr)| {
                let pattern = if arity == 1 {
                    items.remove(0)
                } else {
                    DesugaredMatchArmPattern::Tuple {
                        items: items.clone(),
                        span: items[0].span(),
                    }
                };

                DesugaredMatchArm {
                    pattern,
                    value_expr,
                }
            })
            .collect::<Vec<_>>();

        let param_names = (0..arity).map(|i| format!("arg{i}")).collect::<Vec<_>>();

        let scrutinee = if arity == 1 {
            DesugaredExpr::Identifier {
                name: param_names[0].clone(),
                span,
            }
        } else {
            DesugaredExpr::Tuple {
                items: param_names
                    .iter()
                    .map(|name| DesugaredExpr::Identifier {
                        name: name.clone(),
                        span,
                    })
                    .collect(),
                span,
            }
        };

        let match_expr = DesugaredExpr::Match {
            scrutinee: scrutinee.into(),
            match_arms,
            span,
        };

        // build the chained lambda
        let lambda = param_names.into_iter().zip(param_types).rev().fold(
            match_expr,
            |body, (param_name, param_type)| DesugaredExpr::Lambda {
                param_name,
                param_type: param_type.into(),
                body: Rc::new(body),
                span,
            },
        );

        statements.push(DesugaredStatement::Declaration {
            pattern: DesugaredDeclPattern::Identifier {
                name: fn_name,
                span,
            },
            expr: lambda,
            span,
        });
    }

    (statements, errors)
}

fn find_fn_clause_param_types(
    statements: &Vec<DesugaredStatement>,
    fn_name: &str,
    arity: usize,
    span: Span,
) -> FogResult<Vec<DesugaredExpr>> {
    let mut remaining_type = statements
        .iter()
        .find_map(|stmt| match stmt {
            DesugaredStatement::TypeAnnotation { name, expr, .. } if name == fn_name => {
                Some(expr.clone())
            }
            _ => None,
        })
        .ok_or_else(|| {
            parse_error!(
                Some(span),
                "function `{fn_name}` needs a type annotation to use pattern-matched clauses"
            )
        })?;

    let mut param_types = Vec::with_capacity(arity);

    for _ in 0..arity {
        match remaining_type {
            DesugaredExpr::FunctionAppl {
                fn_name: op,
                mut args,
                ..
            } if op == "->" && args.len() == 2 => {
                remaining_type = args.pop().unwrap();
                param_types.push(args.pop().unwrap());
            }

            _ => {
                return Err(parse_error!(
                    Some(span),
                    "function `{fn_name}` is declared with {arity} argument(s), \
                     but its type signature only accounts for {}",
                    param_types.len()
                ));
            }
        }
    }

    Ok(param_types)
}

fn desugar_statement(stmt: ResolvedStatement) -> FogResult<DesugarResult> {
    match stmt {
        ResolvedStatement::TypeAnnotation { name, expr, span } => Ok(DesugarResult::Statement(
            DesugaredStatement::TypeAnnotation {
                name,
                expr: desugar_expr(expr)?,
                span,
            },
        )),

        ResolvedStatement::Declaration {
            pattern,
            expr,
            span,
        } => Ok(DesugarResult::Declaration {
            pattern,
            expr: desugar_expr(expr)?,
            span,
        }),

        ResolvedStatement::Expression { expr, span } => {
            Ok(DesugarResult::Statement(DesugaredStatement::Expression {
                expr: desugar_expr(expr)?,
                span,
            }))
        }
    }
}

fn desugar_tuple_decl_pattern(
    pattern: ResolvedTupleDeclPattern,
) -> FogResult<DesugaredTupleDeclPattern> {
    match pattern {
        ResolvedTupleDeclPattern::Identifier { name, span } => {
            Ok(DesugaredTupleDeclPattern::Identifier { name, span })
        }

        ResolvedTupleDeclPattern::Tuple { items, span } => Ok(DesugaredTupleDeclPattern::Tuple {
            items: items
                .into_iter()
                .map(desugar_tuple_decl_pattern)
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),
    }
}

fn desugar_match_arm_pattern(
    pattern: ResolvedMatchArmPattern,
) -> FogResult<DesugaredMatchArmPattern> {
    match pattern {
        ResolvedMatchArmPattern::Literal { literal, span } => {
            Ok(DesugaredMatchArmPattern::Literal { literal, span })
        }

        ResolvedMatchArmPattern::Tuple { items, span } => Ok(DesugaredMatchArmPattern::Tuple {
            items: items
                .into_iter()
                .map(desugar_match_arm_pattern)
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),

        ResolvedMatchArmPattern::Identifier { name, span } => {
            Ok(DesugaredMatchArmPattern::Identifier { name, span })
        }

        ResolvedMatchArmPattern::DataConstructor { name, args, span } => {
            Ok(DesugaredMatchArmPattern::DataConstructor {
                name,
                args: args
                    .into_iter()
                    .map(desugar_match_arm_pattern)
                    .collect::<Result<Vec<_>, _>>()?,
                span,
            })
        }
    }
}

fn desugar_expr(resolved_expr: ResolvedExpr) -> FogResult<DesugaredExpr> {
    match resolved_expr {
        ResolvedExpr::Block { statements, span } => Ok(desugar_block(statements, span).0?),

        ResolvedExpr::Identifier { name, span } => Ok(DesugaredExpr::Identifier { name, span }),

        ResolvedExpr::Literal { literal, span } => Ok(DesugaredExpr::Literal { literal, span }),

        ResolvedExpr::Lambda {
            param_name,
            param_type,
            body,
            span,
        } => Ok(DesugaredExpr::Lambda {
            param_name,
            param_type: desugar_expr(*param_type)?.into(),
            body: desugar_expr((*body).clone())?.into(),
            span,
        }),

        ResolvedExpr::Tuple { items, span } => Ok(DesugaredExpr::Tuple {
            items: items
                .into_iter()
                .map(desugar_expr)
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),

        ResolvedExpr::FunctionAppl {
            fn_name,
            args,
            span,
        } => Ok(DesugaredExpr::FunctionAppl {
            fn_name,
            args: args
                .into_iter()
                .map(desugar_expr)
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),

        ResolvedExpr::Match {
            scrutinee,
            match_arms,
            span,
        } => Ok(DesugaredExpr::Match {
            scrutinee: desugar_expr(*scrutinee)?.into(),
            match_arms: match_arms
                .into_iter()
                .map(|arm| {
                    Ok(DesugaredMatchArm {
                        pattern: desugar_match_arm_pattern(arm.pattern)?,
                        value_expr: desugar_expr(arm.value_expr)?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),
    }
}
