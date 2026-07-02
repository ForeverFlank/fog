use std::collections::HashMap;
use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::parse_error;
use crate::parser::desugared_expr::DesugaredDeclPattern;
use crate::parser::desugared_expr::DesugaredExpr;
use crate::parser::desugared_expr::DesugaredMatchArm;
use crate::parser::desugared_expr::DesugaredStatement;
use crate::parser::resolved_expr::ResolvedDeclPattern;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::parser::resolved_expr::ResolvedStatement;

enum DesugarResult {
    Statement(DesugaredStatement),
    Declaration {
        pattern: ResolvedDeclPattern,
        expr: DesugaredExpr,
        span: Span,
    },
}
struct FunctionClause {
    arg_patterns: Vec<ResolvedDeclPattern>,
    body: DesugaredExpr,
    span: Span,
}

pub fn desugar(resolved_stmts: Vec<ResolvedStatement>) -> (Vec<DesugaredStatement>, Vec<FogError>) {
    let mut errors = Vec::new();

    let statements = match desugar_block(resolved_stmts, Span { line: 0, column: 0 }) {
        Ok(DesugaredExpr::Block { statements, .. }) => statements,
        Ok(_) => unreachable!(),
        Err(error) => {
            errors.push(error);
            Vec::new()
        }
    };

    (statements, errors)
}

fn desugar_block(resolved_stmts: Vec<ResolvedStatement>, span: Span) -> FogResult<DesugaredExpr> {
    let mut statements: Vec<DesugaredStatement> = Vec::new();

    let mut fn_clauses: HashMap<String, Vec<FunctionClause>> = HashMap::new();
    let mut fn_stmt_index: HashMap<String, usize> = HashMap::new();

    for resolved_stmt in resolved_stmts {
        match desugar_statement(resolved_stmt)? {
            DesugarResult::Statement(stmt) => statements.push(stmt),

            DesugarResult::Declaration {
                pattern,
                expr,
                span,
            } => match pattern {
                ResolvedDeclPattern::Identifier { name, span: _ } => {
                    statements.push(DesugaredStatement::Declaration {
                        pattern: DesugaredDeclPattern::Identifier {
                            name,
                            span: span.clone(),
                        },
                        expr,
                        span,
                    })
                }

                ResolvedDeclPattern::Tuple { items, span: _ } => {
                    statements.push(DesugaredStatement::Declaration {
                        pattern: DesugaredDeclPattern::Tuple {
                            items: items
                                .into_iter()
                                .map(decl_pattern_to_desugared_decl_pattern)
                                .collect::<FogResult<Vec<_>>>()?,
                            span: span.clone(),
                        },
                        expr,
                        span,
                    });
                }

                ResolvedDeclPattern::FunctionClause { name, items, span } => {
                    let clause = FunctionClause {
                        arg_patterns: items,
                        body: expr,
                        span,
                    };

                    if let Some(clauses) = fn_clauses.get_mut(&name) {
                        clauses.push(clause);
                    } else {
                        fn_stmt_index.insert(name.clone(), statements.len());
                        statements.push(DesugaredStatement::Expression {
                            expr: DesugaredExpr::Tuple {
                                items: Vec::new(),
                                span: clause.span.clone(),
                            },
                            span: clause.span.clone(),
                        });
                        fn_clauses.insert(name, vec![clause]);
                    }
                }
            },
        }
    }

    for (name, index) in fn_stmt_index {
        let clauses = fn_clauses.remove(&name).unwrap();
        statements[index] = desugar_fn_clauses(name, clauses)?;
    }

    Ok(DesugaredExpr::Block { statements, span })
}

fn desugar_fn_clauses(name: String, clauses: Vec<FunctionClause>) -> FogResult<DesugaredStatement> {
    let span = clauses[0].span.clone();
    let arity = clauses[0].arg_patterns.len();

    for clause in &clauses {
        if clause.arg_patterns.len() != arity {
            return Err(parse_error!(
                Some(clause.span.clone()),
                "all clauses of `{name}` must take the same number of arguments"
            ));
        }
    }

    let param_names: Vec<String> = (0..arity).map(|i| format!("_arg{i}")).collect();

    let match_arms = clauses
        .into_iter()
        .map(|clause| {
            let pattern = decl_patterns_to_pattern_expr(clause.arg_patterns, clause.span)?;
            Ok(DesugaredMatchArm {
                pattern,
                value_expr: clause.body,
            })
        })
        .collect::<FogResult<Vec<_>>>()?;

    let scrutinee = param_names_to_expr(&param_names, &span);

    let match_expr = DesugaredExpr::Match {
        expr: Box::new(scrutinee),
        match_arms,
        span: span.clone(),
    };

    let body = param_names
        .into_iter()
        .rev()
        .fold(match_expr, |acc, param_name| DesugaredExpr::Lambda {
            param_name,
            param_type: Box::new(DesugaredExpr::Identifier {
                name: "_".to_string(),
                span: span.clone(),
            }),
            body: Rc::new(acc),
            span: span.clone(),
        });

    Ok(DesugaredStatement::Declaration {
        pattern: DesugaredDeclPattern::Identifier {
            name,
            span: span.clone(),
        },
        expr: body,
        span,
    })
}

fn param_names_to_expr(param_names: &[String], span: &Span) -> DesugaredExpr {
    if param_names.len() == 1 {
        DesugaredExpr::Identifier {
            name: param_names[0].clone(),
            span: span.clone(),
        }
    } else {
        DesugaredExpr::Tuple {
            items: param_names
                .iter()
                .map(|name| DesugaredExpr::Identifier {
                    name: name.clone(),
                    span: span.clone(),
                })
                .collect(),
            span: span.clone(),
        }
    }
}

fn decl_patterns_to_pattern_expr(
    patterns: Vec<ResolvedDeclPattern>,
    span: Span,
) -> FogResult<DesugaredExpr> {
    if patterns.len() == 1 {
        decl_pattern_to_pattern_expr(patterns.into_iter().next().unwrap())
    } else {
        Ok(DesugaredExpr::Tuple {
            items: patterns
                .into_iter()
                .map(decl_pattern_to_pattern_expr)
                .collect::<FogResult<Vec<_>>>()?,
            span,
        })
    }
}

fn decl_pattern_to_pattern_expr(pattern: ResolvedDeclPattern) -> FogResult<DesugaredExpr> {
    match pattern {
        ResolvedDeclPattern::Identifier { name, span } => {
            Ok(DesugaredExpr::Identifier { name, span })
        }

        ResolvedDeclPattern::Tuple { items, span } => Ok(DesugaredExpr::Tuple {
            items: items
                .into_iter()
                .map(decl_pattern_to_pattern_expr)
                .collect::<FogResult<Vec<_>>>()?,
            span,
        }),

        ResolvedDeclPattern::FunctionClause { name, items, span } => Ok(DesugaredExpr::FuncAppl {
            fn_name: name,
            args: items
                .into_iter()
                .map(decl_pattern_to_pattern_expr)
                .collect::<FogResult<Vec<_>>>()?,
            span,
        }),
    }
}

// for declaring to a name or a tuple
fn decl_pattern_to_desugared_decl_pattern(
    pattern: ResolvedDeclPattern,
) -> FogResult<DesugaredDeclPattern> {
    match pattern {
        ResolvedDeclPattern::Identifier { name, span } => {
            Ok(DesugaredDeclPattern::Identifier { name, span })
        }

        ResolvedDeclPattern::Tuple { items, span } => Ok(DesugaredDeclPattern::Tuple {
            items: items
                .into_iter()
                .map(decl_pattern_to_desugared_decl_pattern)
                .collect::<FogResult<Vec<_>>>()?,
            span,
        }),

        ResolvedDeclPattern::FunctionClause { name, span, .. } => Err(parse_error!(
            Some(span),
            "`{name}` cannot be used as a value pattern"
        )),
    }
}

fn desugar_statement(resolved_stmt: ResolvedStatement) -> FogResult<DesugarResult> {
    match resolved_stmt {
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

fn desugar_expr(resolved_expr: ResolvedExpr) -> FogResult<DesugaredExpr> {
    match resolved_expr {
        ResolvedExpr::Block { statements, span } => desugar_block(statements, span),

        ResolvedExpr::Identifier { name, span } => Ok(DesugaredExpr::Identifier { name, span }),

        ResolvedExpr::Int32Literal { value, span } => {
            Ok(DesugaredExpr::Int32Literal { value, span })
        }
        ResolvedExpr::Float32Literal { value, span } => {
            Ok(DesugaredExpr::Float32Literal { value, span })
        }

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

        ResolvedExpr::FuncAppl {
            fn_name,
            args,
            span,
        } => Ok(DesugaredExpr::FuncAppl {
            fn_name,
            args: args
                .into_iter()
                .map(desugar_expr)
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),

        ResolvedExpr::Match {
            expr,
            match_arms,
            span,
        } => Ok(DesugaredExpr::Match {
            expr: desugar_expr(*expr)?.into(),
            match_arms: match_arms
                .into_iter()
                .map(|arm| {
                    Ok(DesugaredMatchArm {
                        pattern: desugar_expr(arm.pattern)?,
                        value_expr: desugar_expr(arm.value_expr)?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),
    }
}
