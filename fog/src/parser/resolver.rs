use std::collections::HashMap;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::parse_error;
use crate::parser::parsed_expr::OpKind;
use crate::parser::parsed_expr::ParsedExpr;
use crate::parser::parsed_expr::ParsedStatement;
use crate::parser::resolved_expr::MatchArm;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::parser::resolved_expr::ResolvedStatement;

pub struct Resolver {
    infix_functions: HashMap<InfixFunctionKey, InfixFunctionInfo>,
    index: usize,
}

#[derive(Clone)]
pub enum Associativity {
    Left,
    Right,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum InfixFunctionKey {
    Op(OpKind),
    Identifier(String),
}

pub struct InfixFunctionInfo {
    pub name: String,
    pub associativity: Associativity,
    pub precedence: i32,
}

fn is_primary_starter(parsed_expr: &ParsedExpr) -> bool {
    match parsed_expr {
        ParsedExpr::Identifier { .. }
        | ParsedExpr::Int32Literal { .. }
        | ParsedExpr::Float32Literal { .. }
        | ParsedExpr::Tuple { .. }
        | ParsedExpr::Collection { .. } => true,

        ParsedExpr::Op { kind } => match kind {
            OpKind::Minus => true,
            _ => false,
        },

        _ => false,
    }
}

impl Resolver {
    fn new() -> Self {
        Self {
            infix_functions: HashMap::from([
                (
                    InfixFunctionKey::Op(OpKind::Star),
                    InfixFunctionInfo {
                        name: "*".into(),
                        associativity: Associativity::Left,
                        precedence: 3,
                    },
                ),
                (
                    InfixFunctionKey::Op(OpKind::Slash),
                    InfixFunctionInfo {
                        name: "/".into(),
                        associativity: Associativity::Left,
                        precedence: 3,
                    },
                ),
                (
                    InfixFunctionKey::Op(OpKind::Plus),
                    InfixFunctionInfo {
                        name: "+".into(),
                        associativity: Associativity::Left,
                        precedence: 2,
                    },
                ),
                (
                    InfixFunctionKey::Op(OpKind::Minus),
                    InfixFunctionInfo {
                        name: "-".into(),
                        associativity: Associativity::Left,
                        precedence: 2,
                    },
                ),
                (
                    InfixFunctionKey::Op(OpKind::Arrow),
                    InfixFunctionInfo {
                        name: "->".into(),
                        associativity: Associativity::Right,
                        precedence: 1,
                    },
                ),
            ]),
            index: 0,
        }
    }

    fn get_binary_op(&self, parsed_expr: &ParsedExpr) -> Option<&InfixFunctionInfo> {
        let key = match parsed_expr {
            ParsedExpr::Op { kind } => InfixFunctionKey::Op(kind.clone()),
            ParsedExpr::Identifier { name, .. } => InfixFunctionKey::Identifier(name.clone()),
            _ => return None,
        };
        self.infix_functions.get(&key)
    }

    pub fn resolve(
        parsed_statements: Vec<ParsedStatement>,
    ) -> (Vec<ResolvedStatement>, Vec<FogError>) {
        let mut resolved_statements = Vec::new();
        let mut errors = Vec::new();

        for parsed_stmt in parsed_statements {
            match Self::resolve_statement(parsed_stmt) {
                Ok(resolved_stmt) => resolved_statements.push(resolved_stmt),
                Err(error) => errors.push(error),
            }
        }

        (resolved_statements, errors)
    }

    fn resolve_statement(parsed_statement: ParsedStatement) -> FogResult<ResolvedStatement> {
        match parsed_statement {
            ParsedStatement::TypeAnnotation { name, expr, span } => {
                Ok(ResolvedStatement::TypeAnnotation {
                    name,
                    expr: Self::resolve_expr(expr)?,
                    span,
                })
            }

            ParsedStatement::Declaration { name, expr, span } => {
                Ok(ResolvedStatement::Declaration {
                    name,
                    expr: Self::resolve_expr(expr)?,
                    span,
                })
            }

            ParsedStatement::Expression { expr, span } => Ok(ResolvedStatement::Expression {
                expr: Self::resolve_expr(expr)?,
                span,
            }),
        }
    }

    fn resolve_lambda(
        param_name: String,
        param_type: Box<ParsedExpr>,
        body: Box<ParsedExpr>,
        span: Span,
    ) -> FogResult<ResolvedExpr> {
        // Desugar `param => match { arms }` into `param => match param { arms }`.
        let body = match *body {
            ParsedExpr::Match {
                expr: None,
                match_arms,
                span: match_span,
            } => ParsedExpr::Match {
                expr: Some(Box::new(ParsedExpr::Identifier {
                    name: param_name.clone(),
                    span: match_span.clone(),
                })),
                match_arms,
                span: match_span,
            },
            other => other,
        };

        Ok(ResolvedExpr::Lambda {
            param_name,
            param_type: Self::resolve_expr(*param_type)?.into(),
            body: Self::resolve_expr(body)?.into(),
            span,
        })
    }

    fn resolve_tuple(items: Vec<ParsedExpr>, span: Span) -> FogResult<ResolvedExpr> {
        Ok(ResolvedExpr::Tuple {
            items: items
                .into_iter()
                .map(Self::resolve_expr)
                .collect::<Result<Vec<_>, _>>()?,
            span,
        })
    }

    fn resolve_match(
        expr: Option<Box<ParsedExpr>>,
        match_arms: Vec<crate::parser::parsed_expr::MatchArm>,
        span: Span,
    ) -> FogResult<ResolvedExpr> {
        let scrutinee = match expr {
            Some(e) => Self::resolve_expr(*e)?,
            None => {
                return Err(parse_error!(
                    Some(span),
                    "`match {{ ... }}` without a scrutinee can only appear as a direct lambda body"
                ));
            }
        };

        Ok(ResolvedExpr::Match {
            expr: Box::new(scrutinee),
            match_arms: match_arms
                .into_iter()
                .map(|arm| {
                    Ok(MatchArm {
                        pattern: Self::resolve_expr(arm.pattern)?,
                        value_expr: Self::resolve_expr(arm.value_expr)?,
                    })
                })
                .collect::<FogResult<Vec<_>>>()?,
            span,
        })
    }

    fn resolve_expr(parsed_expr: ParsedExpr) -> FogResult<ResolvedExpr> {
        match parsed_expr {
            ParsedExpr::Block { statements, span } => resolve_block(statements, span),

            ParsedExpr::Identifier { name, span } => Ok(ResolvedExpr::Identifier { name, span }),
            ParsedExpr::Op { .. } => unreachable!(),

            ParsedExpr::Int32Literal { value, span } => {
                Ok(ResolvedExpr::Int32Literal { value, span })
            }
            ParsedExpr::Float32Literal { value, span } => {
                Ok(ResolvedExpr::Float32Literal { value, span })
            }

            ParsedExpr::Lambda {
                param_name,
                param_type,
                body,
                span,
            } => Self::resolve_lambda(param_name, param_type, body, span),

            ParsedExpr::Tuple { items, span } => Self::resolve_tuple(items, span),

            ParsedExpr::Collection { items, span: _ } => {
                let mut resolver = Resolver::new();
                resolver.resolve_collection(&items, i32::MIN)
            }

            ParsedExpr::Match {
                expr,
                match_arms,
                span,
            } => Self::resolve_match(expr, match_arms, span),
        }
    }

    fn resolve_collection(
        &mut self,
        items: &Vec<ParsedExpr>,
        min_prec: i32,
    ) -> FogResult<ResolvedExpr> {
        let mut lhs = self.resolve_primary(items)?;

        loop {
            if self.index >= items.len() {
                break;
            }

            let op = match self.get_binary_op(&items[self.index]) {
                Some(op) => op,
                None => break,
            };

            if op.precedence < min_prec {
                break;
            }

            let op_name = op.name.clone();
            let op_prec = op.precedence;
            let op_assoc = op.associativity.clone();
            let lhs_span = lhs.span();

            self.index += 1;

            let next_min_prec = match op_assoc {
                Associativity::Left => op_prec + 1,
                Associativity::Right => op_prec,
            };

            let rhs = self.resolve_collection(items, next_min_prec)?;

            lhs = ResolvedExpr::FuncAppl {
                fn_name: op_name,
                args: vec![lhs, rhs],
                span: lhs_span,
            };
        }

        Ok(lhs)
    }

    fn resolve_primary(&mut self, exprs: &Vec<ParsedExpr>) -> FogResult<ResolvedExpr> {
        let head = self.resolve_atomic(exprs)?;

        let (name, span) = match &head {
            ResolvedExpr::Identifier { name, span } => (name.clone(), span.clone()),
            _ => return Ok(head),
        };

        let mut args = Vec::new();

        while self.index < exprs.len() && is_primary_starter(&exprs[self.index]) {
            args.push(self.resolve_atomic(exprs)?);
        }

        if args.is_empty() {
            Ok(head)
        } else {
            Ok(ResolvedExpr::FuncAppl {
                fn_name: name,
                args,
                span,
            })
        }
    }

    fn resolve_atomic(&mut self, exprs: &Vec<ParsedExpr>) -> FogResult<ResolvedExpr> {
        let expr = exprs[self.index].clone();
        self.index += 1;

        match expr {
            ParsedExpr::Block { statements, span } => resolve_block(statements, span),

            ParsedExpr::Identifier { name, span } => Ok(ResolvedExpr::Identifier { name, span }),

            ParsedExpr::Int32Literal { value, span } => {
                Ok(ResolvedExpr::Int32Literal { value, span })
            }
            ParsedExpr::Float32Literal { value, span } => {
                Ok(ResolvedExpr::Float32Literal { value, span })
            }

            ParsedExpr::Op {
                kind: OpKind::Minus,
            } => {
                let operand = self.resolve_atomic(exprs)?;
                let span = operand.span();
                Ok(ResolvedExpr::FuncAppl {
                    fn_name: "-".to_string(),
                    args: vec![operand],
                    span,
                })
            }

            ParsedExpr::Lambda {
                param_name,
                param_type,
                body,
                span,
            } => Self::resolve_lambda(param_name, param_type, body, span),

            ParsedExpr::Tuple { items, span } => Self::resolve_tuple(items, span),

            // TODO: this looks really hacky
            ParsedExpr::Collection { items, .. } => {
                let saved_index = self.index;
                self.index = 0;
                let result = self.resolve_collection(&items, i32::MIN)?;
                self.index = saved_index;
                Ok(result)
            }

            ParsedExpr::Op { .. } => Err(parse_error!(None, "unexpected infix operator")),

            ParsedExpr::Match {
                expr,
                match_arms,
                span,
            } => Self::resolve_match(expr, match_arms, span),
        }
    }
}

fn resolve_block(statements: Vec<ParsedStatement>, span: Span) -> FogResult<ResolvedExpr> {
    Ok(ResolvedExpr::Block {
        statements: statements
            .iter()
            .map(|stmt| Resolver::resolve_statement(stmt.clone()))
            .collect::<Result<Vec<_>, _>>()?,
        span,
    })
}
