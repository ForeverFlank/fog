use std::collections::HashMap;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::parse_error;
use crate::parser::core_expr::CoreAtomicTypeExpr;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreMatchArm;
use crate::parser::core_expr::CoreMatchArmPattern;
use crate::parser::core_expr::CoreStatement;
use crate::parser::core_expr::CoreTupleDeclPattern;
use crate::parser::core_expr::CoreTypeExpr;
use crate::parser::resolved_expr::ResolvedDeclPattern;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::parser::resolved_expr::ResolvedMatchArmPattern;
use crate::parser::resolved_expr::ResolvedStatement;
use crate::parser::resolved_expr::ResolvedTupleDeclPattern;

enum DesugarResult {
    Statement(CoreStatement),
    Declaration {
        pattern: ResolvedDeclPattern,
        expr: CoreExpr,
        span: Span,
    },
}

pub fn desugar(resolved_stmts: Vec<ResolvedStatement>) -> (Vec<CoreStatement>, Vec<FogError>) {
    desugar_statements(resolved_stmts)
}

fn desugar_block(
    resolved_stmts: Vec<ResolvedStatement>,
    span: Span,
) -> (FogResult<CoreExpr>, Vec<FogError>) {
    let (statements, errors) = desugar_statements(resolved_stmts);
    let block = CoreExpr::Block { statements, span };

    (Ok(block), errors)
}

fn desugar_statements(
    resolved_stmts: Vec<ResolvedStatement>,
) -> (Vec<CoreStatement>, Vec<FogError>) {
    let mut fn_decl_patterns: HashMap<String, Vec<(Vec<CoreMatchArmPattern>, CoreExpr)>> =
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
                statements.push(CoreStatement::VarDeclaration {
                    pattern: CoreDeclPattern::Identifier { name, span },
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

                statements.push(CoreStatement::VarDeclaration {
                    pattern: CoreDeclPattern::Tuple { items, span },
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

        let arms = patterns
            .into_iter()
            .map(|(mut items, value_expr)| {
                let pattern = if arity == 1 {
                    items.remove(0)
                } else {
                    CoreMatchArmPattern::Tuple {
                        items: items.clone(),
                        span: items[0].span(),
                    }
                };

                CoreMatchArm {
                    pattern,
                    value_expr,
                }
            })
            .collect::<Vec<_>>();

        let param_names = (0..arity).map(|i| format!("arg{i}")).collect::<Vec<_>>();

        let scrutinee = if arity == 1 {
            CoreExpr::Identifier {
                name: param_names[0].clone(),
                span,
            }
        } else {
            CoreExpr::Tuple {
                items: param_names
                    .iter()
                    .map(|name| CoreExpr::Identifier {
                        name: name.clone(),
                        span,
                    })
                    .collect(),
                span,
            }
        };

        let match_expr = CoreExpr::Match {
            scrutinee: scrutinee.into(),
            arms,
            span,
        };

        // build the chained lambda
        let lambda = param_names.into_iter().zip(param_types).rev().fold(
            match_expr,
            |body, (param_name, param_type)| CoreExpr::Lambda {
                param_name,
                param_type,
                body: body.into(),
                span,
            },
        );

        statements.push(CoreStatement::VarDeclaration {
            pattern: CoreDeclPattern::Identifier {
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
    statements: &Vec<CoreStatement>,
    fn_name: &str,
    arity: usize,
    span: Span,
) -> FogResult<Vec<CoreAtomicTypeExpr>> {
    let mut remaining_type = statements
        .iter()
        .find_map(|stmt| match stmt {
            CoreStatement::TypeAnnotation { name, expr, .. } if name == fn_name => {
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
        let arity_error = || {
            parse_error!(
                Some(span),
                "function `{fn_name}` is declared with {arity} argument(s), \
                 but its type signature only accounts for {}",
                param_types.len()
            )
        };

        let CoreAtomicTypeExpr::FunctionAppl {
            callee,
            arg: return_type,
            ..
        } = remaining_type
        else {
            return Err(arity_error());
        };

        let CoreAtomicTypeExpr::FunctionAppl {
            callee: op,
            arg: param_type,
            ..
        } = *callee
        else {
            return Err(arity_error());
        };

        let CoreAtomicTypeExpr::Identifier { name: op_name, .. } = *op else {
            return Err(arity_error());
        };

        if op_name != "->" {
            return Err(arity_error());
        }

        param_types.push(*param_type);
        remaining_type = *return_type;
    }

    Ok(param_types)
}

fn desugar_statement(stmt: ResolvedStatement) -> FogResult<DesugarResult> {
    match stmt {
        ResolvedStatement::KindAnnotation { name, expr, span } => {
            Ok(DesugarResult::Statement(CoreStatement::KindAnnotation {
                name,
                expr,
                span,
            }))
        }

        ResolvedStatement::TypeDeclaration { name, expr, span } => {
            Ok(DesugarResult::Statement(CoreStatement::TypeDeclaration {
                name,
                expr: desugar_type_expr(expr)?,
                span,
            }))
        }

        ResolvedStatement::TypeAnnotation { name, expr, span } => {
            Ok(DesugarResult::Statement(CoreStatement::TypeAnnotation {
                name,
                expr: desugar_atomic_type_expr(expr)?,
                span,
            }))
        }

        ResolvedStatement::VarDeclaration {
            pattern,
            expr,
            span,
        } => Ok(DesugarResult::Declaration {
            pattern,
            expr: desugar_expr(expr)?,
            span,
        }),

        ResolvedStatement::Expression { expr, span } => {
            Ok(DesugarResult::Statement(CoreStatement::Expression {
                expr: desugar_expr(expr)?,
                span,
            }))
        }
    }
}

fn desugar_tuple_decl_pattern(
    pattern: ResolvedTupleDeclPattern,
) -> FogResult<CoreTupleDeclPattern> {
    match pattern {
        ResolvedTupleDeclPattern::Identifier { name, span } => {
            Ok(CoreTupleDeclPattern::Identifier { name, span })
        }

        ResolvedTupleDeclPattern::Tuple { items, span } => Ok(CoreTupleDeclPattern::Tuple {
            items: items
                .into_iter()
                .map(desugar_tuple_decl_pattern)
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),
    }
}

fn desugar_match_arm_pattern(pattern: ResolvedMatchArmPattern) -> FogResult<CoreMatchArmPattern> {
    match pattern {
        ResolvedMatchArmPattern::Literal { literal, span } => {
            Ok(CoreMatchArmPattern::Literal { literal, span })
        }

        ResolvedMatchArmPattern::Tuple { items, span } => Ok(CoreMatchArmPattern::Tuple {
            items: items
                .into_iter()
                .map(desugar_match_arm_pattern)
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),

        ResolvedMatchArmPattern::Identifier { name, span } => {
            Ok(CoreMatchArmPattern::Identifier { name, span })
        }

        ResolvedMatchArmPattern::DataConstructor { name, args, span } => {
            Ok(CoreMatchArmPattern::DataConstructor {
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

fn desugar_type_expr(resolved_expr: ResolvedExpr) -> FogResult<CoreTypeExpr> {
    let span = resolved_expr.span();

    if let Some(operands) = flatten_operator("+", &resolved_expr) {
        return Ok(CoreTypeExpr::Sum {
            ctors: operands
                .into_iter()
                .map(desugar_data_constructor)
                .collect::<Result<_, _>>()?,
            span,
        });
    }

    if let Some(operands) = flatten_operator("*", &resolved_expr) {
        return Ok(CoreTypeExpr::Product {
            types: operands
                .into_iter()
                .map(|e| desugar_atomic_type_expr(e.clone()))
                .collect::<Result<_, _>>()?,
            span,
        });
    }

    Ok(desugar_atomic_type_expr(resolved_expr)?.to_type_expr())
}

fn flatten_operator<'a>(op: &str, expr: &'a ResolvedExpr) -> Option<Vec<&'a ResolvedExpr>> {
    let ResolvedExpr::FunctionAppl { .. } = expr else {
        return None;
    };
    let (head, args) = expr.uncurry();
    let ResolvedExpr::Identifier { name, .. } = head else {
        return None;
    };
    if name != op || args.len() != 2 {
        return None;
    }

    let mut operands = Vec::new();
    for arg in args {
        match flatten_operator(op, arg) {
            Some(nested) => operands.extend(nested),
            None => operands.push(arg),
        }
    }
    Some(operands)
}

fn desugar_atomic_type_expr(resolved_expr: ResolvedExpr) -> FogResult<CoreAtomicTypeExpr> {
    match resolved_expr {
        ResolvedExpr::Identifier { name, span } => {
            Ok(CoreAtomicTypeExpr::Identifier { name, span })
        }

        ResolvedExpr::Tuple { items, span } => Ok(CoreAtomicTypeExpr::Product {
            types: items
                .into_iter()
                .map(desugar_atomic_type_expr)
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),

        ResolvedExpr::FunctionAppl { callee, arg, span } => Ok(CoreAtomicTypeExpr::FunctionAppl {
            callee: desugar_atomic_type_expr(*callee)?.into(),
            arg: desugar_atomic_type_expr(*arg)?.into(),
            span,
        }),

        ResolvedExpr::Block { span, .. }
        | ResolvedExpr::Literal { span, .. }
        | ResolvedExpr::Lambda { span, .. }
        | ResolvedExpr::Match { span, .. } => {
            Err(parse_error!(Some(span), "invalid type expression"))
        }
    }
}

fn desugar_expr(resolved_expr: ResolvedExpr) -> FogResult<CoreExpr> {
    match resolved_expr {
        ResolvedExpr::Block { statements, span } => Ok(desugar_block(statements, span).0?),

        ResolvedExpr::Identifier { name, span } => Ok(CoreExpr::Identifier { name, span }),

        ResolvedExpr::Literal { literal, span } => Ok(CoreExpr::Literal { literal, span }),

        ResolvedExpr::Lambda {
            param_name,
            param_type,
            body,
            span,
        } => Ok(CoreExpr::Lambda {
            param_name,
            param_type: desugar_atomic_type_expr(*param_type)?,
            body: desugar_expr((*body).clone())?.into(),
            span,
        }),

        ResolvedExpr::Tuple { items, span } => Ok(CoreExpr::Tuple {
            items: items
                .into_iter()
                .map(desugar_expr)
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),

        ResolvedExpr::FunctionAppl { callee, arg, span } => Ok(CoreExpr::FunctionAppl {
            callee: desugar_expr(*callee)?.into(),
            arg: desugar_expr(*arg)?.into(),
            span,
        }),

        ResolvedExpr::Match {
            scrutinee,
            arms,
            span,
        } => Ok(CoreExpr::Match {
            scrutinee: desugar_expr(*scrutinee)?.into(),
            arms: arms
                .into_iter()
                .map(|arm| {
                    Ok(CoreMatchArm {
                        pattern: desugar_match_arm_pattern(arm.pattern)?,
                        value_expr: desugar_expr(arm.value_expr)?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),
    }
}
