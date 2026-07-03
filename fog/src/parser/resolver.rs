use std::collections::HashMap;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::parse_error;
use crate::parser::parsed_expr::OpKind;
use crate::parser::parsed_expr::ParsedDeclPattern;
use crate::parser::parsed_expr::ParsedExpr;
use crate::parser::parsed_expr::ParsedStatement;
use crate::parser::resolved_expr::ResolvedDeclPattern;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::parser::resolved_expr::ResolvedMatchArm;
use crate::parser::resolved_expr::ResolvedMatchPattern;
use crate::parser::resolved_expr::ResolvedStatement;

pub fn resolve(parsed_statements: Vec<ParsedStatement>) -> (Vec<ResolvedStatement>, Vec<FogError>) {
    Resolver::resolve(parsed_statements)
}

pub struct Resolver {
    infix_functions: HashMap<InfixFunctionKey, InfixFunctionInfo>,
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

            ParsedStatement::Declaration {
                pattern,
                expr,
                span,
            } => Ok(ResolvedStatement::Declaration {
                pattern: Self::resolve_decl_pattern(pattern)?,
                expr: Self::resolve_expr(expr)?,
                span,
            }),

            ParsedStatement::Expression { expr, span } => Ok(ResolvedStatement::Expression {
                expr: Self::resolve_expr(expr)?,
                span,
            }),
        }
    }

    fn resolve_decl_pattern(decl_pattern: ParsedDeclPattern) -> FogResult<ResolvedDeclPattern> {
        match decl_pattern {
            ParsedDeclPattern::Identifier { name, span } => {
                Ok(ResolvedDeclPattern::Identifier { name, span })
            }

            ParsedDeclPattern::Tuple { items, span } => Ok(ResolvedDeclPattern::Tuple {
                items: items
                    .into_iter()
                    .map(Self::resolve_decl_pattern)
                    .collect::<Result<Vec<_>, _>>()?,
                span,
            }),

            ParsedDeclPattern::Collection { items, span } => {
                if items.len() == 3 {
                    let is_infix = match &items[1] {
                        ParsedDeclPattern::Op { .. } => true,
                        ParsedDeclPattern::Identifier { .. } => {
                            !matches!(items[0], ParsedDeclPattern::Identifier { .. })
                        }
                        _ => false,
                    };

                    if is_infix {
                        let [lhs, op, rhs]: [ParsedDeclPattern; 3] = items.try_into().ok().unwrap();

                        let name = match op {
                            ParsedDeclPattern::Op { kind } => kind.to_string(),
                            ParsedDeclPattern::Identifier { name, .. } => name,
                            _ => unreachable!(),
                        };

                        return Ok(ResolvedDeclPattern::FunctionClause {
                            name,
                            items: vec![
                                Self::resolve_decl_pattern(lhs)?,
                                Self::resolve_decl_pattern(rhs)?,
                            ],
                            span,
                        });
                    }
                }

                let mut items = items.into_iter();

                let name = match items.next() {
                    Some(ParsedDeclPattern::Identifier { name, .. }) => name,
                    _ => {
                        return Err(parse_error!(Some(span), "pattern must start with a name"));
                    }
                };

                Ok(ResolvedDeclPattern::FunctionClause {
                    name,
                    items: items
                        .map(Self::resolve_decl_pattern)
                        .collect::<Result<Vec<_>, _>>()?,
                    span,
                })
            }

            ParsedDeclPattern::Op { .. } => {
                Err(parse_error!(None, "unexpected operator in pattern"))
            }

            ParsedDeclPattern::Int32Literal { value, span } => {
                Ok(ResolvedDeclPattern::Int32Literal { value, span })
            }
            ParsedDeclPattern::Float32Literal { value, span } => {
                Ok(ResolvedDeclPattern::Float32Literal { value, span })
            }
        }
    }

    fn resolve_expr(parsed_expr: ParsedExpr) -> FogResult<ResolvedExpr> {
        match parsed_expr {
            ParsedExpr::Block { statements, span } => Self::resolve_block(statements, span),

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
                let mut index = 0;
                resolver.resolve_collection(&items, i32::MIN, &mut index)
            }

            ParsedExpr::Match {
                expr,
                match_arms,
                span,
            } => Self::resolve_match(expr, match_arms, span),
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

    fn resolve_lambda(
        param_name: String,
        param_type: Box<ParsedExpr>,
        body: Box<ParsedExpr>,
        span: Span,
    ) -> FogResult<ResolvedExpr> {
        Ok(ResolvedExpr::Lambda {
            param_name,
            param_type: Self::resolve_expr(*param_type)?.into(),
            body: Self::resolve_expr(*body)?.into(),
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
        expr: Box<ParsedExpr>,
        match_arms: Vec<crate::parser::parsed_expr::ParsedMatchArm>,
        span: Span,
    ) -> FogResult<ResolvedExpr> {
        let scrutinee = Self::resolve_expr(*expr)?;

        Ok(ResolvedExpr::Match {
            expr: Box::new(scrutinee),
            match_arms: match_arms
                .into_iter()
                .map(|arm| {
                    Ok(ResolvedMatchArm {
                        pattern: Self::resolve_expr_as_match_pattern(arm.pattern)?,
                        value_expr: Self::resolve_expr(arm.value_expr)?,
                    })
                })
                .collect::<FogResult<Vec<_>>>()?,
            span,
        })
    }

    fn resolve_expr_as_match_pattern(expr: ParsedExpr) -> FogResult<ResolvedMatchPattern> {
        let resolved = Self::resolve_expr(expr)?;
        Self::resolved_expr_to_match_pattern(resolved)
    }

    fn resolved_expr_to_match_pattern(expr: ResolvedExpr) -> FogResult<ResolvedMatchPattern> {
        let expr_str = expr.to_string();

        match expr {
            ResolvedExpr::Identifier { name, span } => {
                Ok(ResolvedMatchPattern::Identifier { name, span })
            }

            ResolvedExpr::Int32Literal { value, span } => {
                Ok(ResolvedMatchPattern::Int32Literal { value, span })
            }
            ResolvedExpr::Float32Literal { value, span } => {
                Ok(ResolvedMatchPattern::Float32Literal { value, span })
            }

            ResolvedExpr::Tuple { items, span } => Ok(ResolvedMatchPattern::Tuple {
                items: items
                    .into_iter()
                    .map(Self::resolved_expr_to_match_pattern)
                    .collect::<FogResult<Vec<_>>>()?,
                span,
            }),

            ResolvedExpr::FuncAppl {
                fn_name,
                args,
                span,
            } => Ok(ResolvedMatchPattern::FuncAppl {
                fn_name,
                args: args
                    .into_iter()
                    .map(Self::resolved_expr_to_match_pattern)
                    .collect::<FogResult<Vec<_>>>()?,
                span,
            }),

            ResolvedExpr::Block { span, .. }
            | ResolvedExpr::Lambda { span, .. }
            | ResolvedExpr::Match { span, .. } => Err(parse_error!(
                Some(span),
                "`{expr_str}` cannot be used as a pattern"
            )),
        }
    }

    fn resolve_collection(
        &mut self,
        items: &Vec<ParsedExpr>,
        min_prec: i32,
        index: &mut usize,
    ) -> FogResult<ResolvedExpr> {
        let mut lhs = self.resolve_primary(items, index)?;

        loop {
            if *index >= items.len() {
                break;
            }

            let op = match self.get_binary_op(&items[*index]) {
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

            *index += 1;

            let next_min_prec = match op_assoc {
                Associativity::Left => op_prec + 1,
                Associativity::Right => op_prec,
            };

            let rhs = self.resolve_collection(items, next_min_prec, index)?;

            lhs = ResolvedExpr::FuncAppl {
                fn_name: op_name,
                args: vec![lhs, rhs],
                span: lhs_span,
            };
        }

        Ok(lhs)
    }

    fn resolve_primary(
        &mut self,
        exprs: &Vec<ParsedExpr>,
        index: &mut usize,
    ) -> FogResult<ResolvedExpr> {
        let head = self.resolve_atomic(exprs, index)?;

        let (name, span) = match &head {
            ResolvedExpr::Identifier { name, span } => (name.clone(), span.clone()),
            _ => return Ok(head),
        };

        let mut args = Vec::new();

        while *index < exprs.len() && is_primary_starter(&exprs[*index]) {
            args.push(self.resolve_atomic(exprs, index)?);
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

    fn resolve_atomic(
        &mut self,
        exprs: &Vec<ParsedExpr>,
        index: &mut usize,
    ) -> FogResult<ResolvedExpr> {
        let expr = exprs[*index].clone();
        *index += 1;

        match expr {
            ParsedExpr::Block { statements, span } => Self::resolve_block(statements, span),

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
                let operand = self.resolve_atomic(exprs, index)?;
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

            ParsedExpr::Collection { items, .. } => {
                let mut inner_index = 0;
                self.resolve_collection(&items, i32::MIN, &mut inner_index)
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
