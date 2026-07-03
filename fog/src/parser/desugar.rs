use std::collections::HashMap;

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

pub fn desugar(resolved_stmts: Vec<ResolvedStatement>) -> (Vec<DesugaredStatement>, Vec<FogError>) {
    let mut statements = Vec::new();
    let mut errors = Vec::new();

    for resolved_stmt in resolved_stmts {
        match desugar_statement(resolved_stmt) {
            Ok(stmt) => statements.push(stmt),
            Err(error) => errors.push(error),
        }
    }

    (statements, errors)
}

fn desugar_block(
    resolved_stmts: Vec<ResolvedStatement>,
    span: Span,
) -> (FogResult<DesugaredExpr>, Vec<FogError>) {
    let mut fn_decl_patterns: HashMap<String, Vec<(Vec<ResolvedDeclPattern>, DesugaredExpr)>> =
        HashMap::new();
    let mut statements = Vec::new();
    let mut errors = Vec::new();

    for resolved_stmt in resolved_stmts {
        match desugar_statement(resolved_stmt) {
            Ok(res) => match res {
                DesugarResult::Statement(stmt) => statements.push(stmt),

                DesugarResult::Declaration { pattern, expr, .. } => match pattern {
                    ResolvedDeclPattern::Identifier { name, span } => {
                        statements.push(DesugaredStatement::Declaration {
                            pattern: DesugaredDeclPattern::Identifier { name, span },
                            expr,
                            span,
                        })
                    }

                    ResolvedDeclPattern::Tuple { items, span } => {
                        statements.push(DesugaredStatement::Declaration {
                            pattern: DesugaredDeclPattern::Tuple {
                                items: items
                                    .into_iter()
                                    .map(desugar_pattern)
                                    .collect::<Result<Vec<_>, _>>()?,
                                span,
                            },
                            expr,
                            span,
                        });
                    }

                    ResolvedDeclPattern::FunctionClause { name, items, span } => {
                        let mut clauses = fn_decl_patterns.get_mut(&name).unwrap_or({
                            let mut vec = Vec::new();
                            fn_decl_patterns.insert(name, vec);
                            &mut vec
                        });

                        clauses.push((items, expr));
                    }

                    ResolvedDeclPattern::Literal { span, .. } => {
                        errors.push(parse_error!(Some(span), "cannot declare to a literal"));
                    }
                },
            },

            Err(error) => errors.push(error),
        }
    }

    let block = DesugaredExpr::Block {
        statements: statements,
        span,
    };

    (Ok(block), errors)
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

fn desugar_pattern(resolved_pattern: ResolvedDeclPattern) -> FogResult<DesugaredDeclPattern> {
    match resolved_pattern {
        ResolvedDeclPattern::Identifier { name, span } => {
            Ok(DesugaredDeclPattern::Identifier { name, span })
        }

        ResolvedDeclPattern::Literal { value, span } => {
            Ok(DesugaredDeclPattern::Identifier { name: (), span })
        }

        ResolvedDeclPattern::Tuple { items, span } => todo!(),

        ResolvedDeclPattern::FunctionClause { name, items, span } => todo!(),
    }
}

fn desugar_expr(resolved_expr: ResolvedExpr) -> FogResult<DesugaredExpr> {
    match resolved_expr {
        ResolvedExpr::Block { statements, span } => Ok(desugar_block(statements, span).0?),

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

        ResolvedExpr::FunctionAppl {
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
