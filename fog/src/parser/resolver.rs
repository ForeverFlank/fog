use std::collections::HashMap;

use crate::error::FogError;
use crate::error::FogResult;
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
        | ParsedExpr::Tuple { .. } => true,

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
            ParsedExpr::Identifier { name } => InfixFunctionKey::Identifier(name.clone()),
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
    ) -> FogResult<ResolvedExpr> {
        Ok(ResolvedExpr::Lambda {
            param_name,
            param_type: Self::resolve_expr(*param_type)?.into(),
            body: Self::resolve_expr(*body)?.into(),
        })
    }

    fn resolve_tuple(items: Vec<ParsedExpr>) -> FogResult<ResolvedExpr> {
        Ok(ResolvedExpr::Tuple {
            items: items
                .into_iter()
                .map(Self::resolve_expr)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    fn resolve_expr(parsed_expr: ParsedExpr) -> FogResult<ResolvedExpr> {
        match parsed_expr {
            ParsedExpr::Block { statements } => resolve_block(statements),

            ParsedExpr::Identifier { name } => Ok(ResolvedExpr::Identifier { name }),
            ParsedExpr::Op { .. } => unreachable!(),

            ParsedExpr::Int32Literal { value } => Ok(ResolvedExpr::Int32Literal { value }),
            ParsedExpr::Float32Literal { value } => Ok(ResolvedExpr::Float32Literal { value }),

            ParsedExpr::Lambda {
                param_name,
                param_type,
                body,
            } => Self::resolve_lambda(param_name, param_type, body),

            ParsedExpr::Tuple { items } => Self::resolve_tuple(items),

            ParsedExpr::Collection { items } => {
                let mut resolver = Resolver::new();
                resolver.resolve_collection(&items, i32::MIN)
            }

            ParsedExpr::Match { expr, match_arms } => Ok(ResolvedExpr::Match {
                expr: Self::resolve_expr(*expr)?.into(),
                match_arms: match_arms
                    .into_iter()
                    .map(|arm| {
                        Ok(MatchArm {
                            pattern: Self::resolve_expr(arm.pattern)?,
                            value_expr: Self::resolve_expr(arm.value_expr)?,
                        })
                    })
                    .collect::<FogResult<Vec<_>>>()?,
            }),
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

            self.index += 1;

            let next_min_prec = match op_assoc {
                Associativity::Left => op_prec + 1,
                Associativity::Right => op_prec,
            };

            let rhs = self.resolve_collection(items, next_min_prec)?;

            lhs = ResolvedExpr::FuncAppl {
                fn_name: op_name,
                args: vec![lhs, rhs],
            };
        }

        Ok(lhs)
    }

    fn resolve_primary(&mut self, exprs: &Vec<ParsedExpr>) -> FogResult<ResolvedExpr> {
        let head = self.resolve_atomic(exprs)?;

        let name = match &head {
            ResolvedExpr::Identifier { name } => name.clone(),
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
            })
        }
    }

    fn resolve_atomic(&mut self, exprs: &Vec<ParsedExpr>) -> FogResult<ResolvedExpr> {
        let expr = exprs[self.index].clone();
        self.index += 1;

        match expr {
            ParsedExpr::Block { statements } => resolve_block(statements),

            ParsedExpr::Identifier { name } => Ok(ResolvedExpr::Identifier { name }),

            ParsedExpr::Int32Literal { value } => Ok(ResolvedExpr::Int32Literal { value }),
            ParsedExpr::Float32Literal { value } => Ok(ResolvedExpr::Float32Literal { value }),

            ParsedExpr::Op {
                kind: OpKind::Minus,
            } => {
                let operand = self.resolve_atomic(exprs)?;
                Ok(ResolvedExpr::FuncAppl {
                    fn_name: "-".to_string(),
                    args: vec![operand],
                })
            }

            ParsedExpr::Lambda {
                param_name,
                param_type,
                body,
            } => Self::resolve_lambda(param_name, param_type, body),

            ParsedExpr::Tuple { items } => Self::resolve_tuple(items),

            // TODO: this looks really hacky
            ParsedExpr::Collection { items } => {
                let saved_index = self.index;
                self.index = 0;
                let result = self.resolve_collection(&items, i32::MIN)?;
                self.index = saved_index;
                Ok(result)
            }

            ParsedExpr::Op { .. } => Err(parse_error!(None, "unexpected infix operator")),

            ParsedExpr::Match { expr, match_arms } => Ok(ResolvedExpr::Match {
                expr: Self::resolve_expr(*expr)?.into(),
                match_arms: match_arms
                    .into_iter()
                    .map(|arm| {
                        Ok(MatchArm {
                            pattern: Self::resolve_expr(arm.pattern)?,
                            value_expr: Self::resolve_expr(arm.value_expr)?,
                        })
                    })
                    .collect::<FogResult<Vec<_>>>()?,
            }),
        }
    }
}

fn resolve_block(statements: Vec<ParsedStatement>) -> FogResult<ResolvedExpr> {
    Ok(ResolvedExpr::Block {
        statements: statements
            .iter()
            .map(|stmt| Resolver::resolve_statement(stmt.clone()))
            .collect::<Result<Vec<_>, _>>()?,
    })
}
