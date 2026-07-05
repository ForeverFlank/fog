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
use crate::parser::resolved_expr::ResolvedMatchArmPattern;
use crate::parser::resolved_expr::ResolvedStatement;
use crate::parser::resolved_expr::ResolvedTupleDeclPattern;

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
        | ParsedExpr::Literal { .. }
        | ParsedExpr::Tuple { .. }
        | ParsedExpr::Collection { .. } => true,

        ParsedExpr::Op { kind, .. } => match kind {
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
            ParsedExpr::Op { kind, .. } => InfixFunctionKey::Op(kind.clone()),
            ParsedExpr::Identifier { name, .. } => InfixFunctionKey::Identifier(name.clone()),
            _ => return None,
        };

        self.infix_functions.get(&key)
    }

    pub fn resolve(
        parsed_statements: Vec<ParsedStatement>,
    ) -> (Vec<ResolvedStatement>, Vec<FogError>) {
        let resolver = Resolver::new();
        let mut resolved_statements = Vec::new();
        let mut errors = Vec::new();

        for parsed_stmt in parsed_statements {
            match resolver.resolve_statement(parsed_stmt) {
                Ok(resolved_stmt) => resolved_statements.push(resolved_stmt),
                Err(error) => errors.push(error),
            }
        }

        (resolved_statements, errors)
    }

    fn resolve_statement(&self, parsed_statement: ParsedStatement) -> FogResult<ResolvedStatement> {
        match parsed_statement {
            ParsedStatement::TypeAnnotation { name, expr, span } => {
                Ok(ResolvedStatement::TypeAnnotation {
                    name,
                    expr: self.resolve_expr(expr)?,
                    span,
                })
            }

            ParsedStatement::Declaration {
                pattern,
                expr,
                span,
            } => Ok(ResolvedStatement::Declaration {
                pattern: self.resolve_decl_pattern(pattern)?,
                expr: self.resolve_expr(expr)?,
                span,
            }),

            ParsedStatement::Expression { expr, span } => Ok(ResolvedStatement::Expression {
                expr: self.resolve_expr(expr)?,
                span,
            }),
        }
    }

    fn resolve_decl_pattern(&self, pattern: ParsedDeclPattern) -> FogResult<ResolvedDeclPattern> {
        match pattern {
            ParsedDeclPattern::Identifier { name, span } => {
                Ok(ResolvedDeclPattern::Identifier { name, span })
            }

            ParsedDeclPattern::Literal { span, .. } => {
                Err(parse_error!(Some(span), "invalid declaration pattern"))
            }

            ParsedDeclPattern::Tuple { items, span } => Ok(ResolvedDeclPattern::Tuple {
                items: items
                    .into_iter()
                    .map(Self::resolve_tuple_decl_pattern)
                    .collect::<Result<Vec<_>, _>>()?,
                span,
            }),

            ParsedDeclPattern::Collection { items, span } => {
                self.resolve_function_clause(items, span)
            }
        }
    }

    fn resolve_function_clause(
        &self,
        items: Vec<ParsedDeclPattern>,
        span: Span,
    ) -> FogResult<ResolvedDeclPattern> {
        let is_infix = matches!(
            &items.as_slice(),
            [_, ParsedDeclPattern::Identifier { name, .. }, _] if self.is_infix_function_name(name)
        );

        if is_infix {
            let mut iter = items.into_iter();

            let lhs = iter.next().unwrap();
            let name = match iter.next().unwrap() {
                ParsedDeclPattern::Identifier { name, .. } => name,
                _ => unreachable!(),
            };
            let rhs = iter.next().unwrap();

            Ok(ResolvedDeclPattern::FunctionClause {
                name,
                items: vec![
                    Self::resolve_function_clause_item(lhs)?,
                    Self::resolve_function_clause_item(rhs)?,
                ],
                span,
            })
        } else {
            let mut iter = items.into_iter();
            let first = iter.next().unwrap();

            let ParsedDeclPattern::Identifier { name, .. } = first else {
                return Err(parse_error!(
                    Some(span),
                    "function clause's first element must be an identifier"
                ));
            };

            Ok(ResolvedDeclPattern::FunctionClause {
                name,
                items: iter
                    .map(Self::resolve_function_clause_item)
                    .collect::<Result<Vec<_>, _>>()?,
                span,
            })
        }
    }

    fn is_infix_function_name(&self, name: &str) -> bool {
        self.infix_functions.values().any(|info| info.name == name)
    }

    fn resolve_function_clause_item(
        pattern: ParsedDeclPattern,
    ) -> FogResult<ResolvedMatchArmPattern> {
        match pattern {
            ParsedDeclPattern::Identifier { name, span } => {
                Ok(ResolvedMatchArmPattern::Identifier { name, span })
            }

            ParsedDeclPattern::Literal { literal, span } => {
                Ok(ResolvedMatchArmPattern::Literal { literal, span })
            }

            ParsedDeclPattern::Tuple { items, span } => Ok(ResolvedMatchArmPattern::Tuple {
                items: items
                    .into_iter()
                    .map(Self::resolve_function_clause_item)
                    .collect::<Result<Vec<_>, _>>()?,
                span,
            }),

            ParsedDeclPattern::Collection { items, span } => {
                let mut iter = items.into_iter();

                let head = iter.next().unwrap();

                let ParsedDeclPattern::Identifier { name, .. } = head else {
                    return Err(parse_error!(
                        Some(span),
                        "collection pattern's first element must be an identifier"
                    ));
                };

                let first_char = name.chars().next().unwrap();

                if !first_char.is_uppercase() {
                    return Err(parse_error!(
                        Some(span),
                        "data constructor's name must starts with an uppercase letter"
                    ));
                }

                Ok(ResolvedMatchArmPattern::DataConstructor {
                    name,
                    args: iter
                        .map(Self::resolve_function_clause_item)
                        .collect::<Result<Vec<_>, _>>()?,
                    span,
                })
            }
        }
    }

    fn resolve_tuple_decl_pattern(
        pattern: ParsedDeclPattern,
    ) -> FogResult<ResolvedTupleDeclPattern> {
        match pattern {
            ParsedDeclPattern::Identifier { name, span } => {
                Ok(ResolvedTupleDeclPattern::Identifier { name, span })
            }

            ParsedDeclPattern::Tuple { items, span } => Ok(ResolvedTupleDeclPattern::Tuple {
                items: items
                    .into_iter()
                    .map(Self::resolve_tuple_decl_pattern)
                    .collect::<Result<Vec<_>, _>>()?,
                span,
            }),

            ParsedDeclPattern::Literal { span, .. }
            | ParsedDeclPattern::Collection { span, .. } => {
                Err(parse_error!(Some(span), "invalid tuple pattern"))
            }
        }
    }

    fn resolve_expr(&self, parsed_expr: ParsedExpr) -> FogResult<ResolvedExpr> {
        match parsed_expr {
            ParsedExpr::Block { statements, span } => self.resolve_block(statements, span),

            ParsedExpr::Identifier { name, span } => Ok(ResolvedExpr::Identifier { name, span }),
            ParsedExpr::Op { .. } => unreachable!(),

            ParsedExpr::Literal { literal, span } => Ok(ResolvedExpr::Literal { literal, span }),

            ParsedExpr::Lambda {
                param_name,
                param_type,
                body,
                span,
            } => self.resolve_lambda(param_name, param_type, body, span),

            ParsedExpr::Tuple { items, span } => self.resolve_tuple(items, span),

            ParsedExpr::Collection {
                items: args,
                span: _,
            } => {
                let mut index = 0;
                self.resolve_collection(&args, i32::MIN, &mut index)
            }

            ParsedExpr::Match {
                scrutinee,
                match_arms,
                span,
            } => self.resolve_match(scrutinee, match_arms, span),
        }
    }

    fn resolve_block(
        &self,
        statements: Vec<ParsedStatement>,
        span: Span,
    ) -> FogResult<ResolvedExpr> {
        Ok(ResolvedExpr::Block {
            statements: statements
                .iter()
                .map(|stmt| self.resolve_statement(stmt.clone()))
                .collect::<Result<Vec<_>, _>>()?,
            span,
        })
    }

    fn resolve_lambda(
        &self,
        param_name: String,
        param_type: Box<ParsedExpr>,
        body: Box<ParsedExpr>,
        span: Span,
    ) -> FogResult<ResolvedExpr> {
        Ok(ResolvedExpr::Lambda {
            param_name,
            param_type: self.resolve_expr(*param_type)?.into(),
            body: self.resolve_expr(*body)?.into(),
            span,
        })
    }

    fn resolve_tuple(&self, items: Vec<ParsedExpr>, span: Span) -> FogResult<ResolvedExpr> {
        Ok(ResolvedExpr::Tuple {
            items: items
                .into_iter()
                .map(|item| self.resolve_expr(item))
                .collect::<Result<Vec<_>, _>>()?,
            span,
        })
    }

    fn resolve_match(
        &self,
        expr: Box<ParsedExpr>,
        match_arms: Vec<crate::parser::parsed_expr::ParsedMatchArm>,
        span: Span,
    ) -> FogResult<ResolvedExpr> {
        let scrutinee = self.resolve_expr(*expr)?;

        Ok(ResolvedExpr::Match {
            scrutinee: Box::new(scrutinee),
            match_arms: match_arms
                .into_iter()
                .map(|arm| {
                    Ok(ResolvedMatchArm {
                        pattern: Self::resolve_match_pattern(arm.pattern)?,
                        value_expr: self.resolve_expr(arm.value_expr)?,
                    })
                })
                .collect::<FogResult<Vec<_>>>()?,
            span,
        })
    }

    fn resolve_match_pattern(pattern: ParsedExpr) -> FogResult<ResolvedMatchArmPattern> {
        match pattern {
            ParsedExpr::Identifier { name, span } => {
                Ok(ResolvedMatchArmPattern::Identifier { name, span })
            }

            ParsedExpr::Literal { literal, span } => {
                Ok(ResolvedMatchArmPattern::Literal { literal, span })
            }

            ParsedExpr::Tuple { items, span } => Ok(ResolvedMatchArmPattern::Tuple {
                items: items
                    .into_iter()
                    .map(Self::resolve_match_pattern)
                    .collect::<FogResult<Vec<_>>>()?,
                span,
            }),

            // data constructors
            ParsedExpr::Collection { items: args, span } => {
                let first = args.first().ok_or_else(|| {
                    parse_error!(Some(span), "collection pattern items cannot be empty")
                })?;

                let ParsedExpr::Identifier { name, .. } = first else {
                    return Err(parse_error!(
                        Some(span),
                        "collection pattern's first element must be an identifier"
                    ));
                };

                let Some(first_char) = name.chars().next() else {
                    return Err(parse_error!(
                        Some(span),
                        "data constructor's name cannot be empty"
                    ));
                };

                if !first_char.is_uppercase() {
                    return Err(parse_error!(
                        Some(span),
                        "data constructor's name must starts with an uppercase letter"
                    ));
                }

                Ok(ResolvedMatchArmPattern::DataConstructor {
                    name: name.clone(),
                    args: args
                        .into_iter()
                        .skip(1)
                        .map(Self::resolve_match_pattern)
                        .collect::<FogResult<Vec<_>>>()?,
                    span,
                })
            }

            ParsedExpr::Block { span, .. }
            | ParsedExpr::Op { span, .. }
            | ParsedExpr::Lambda { span, .. }
            | ParsedExpr::Match { span, .. } => Err(parse_error!(
                Some(span),
                "invalid match arm pattern `{pattern}`"
            )),
        }
    }

    fn resolve_collection(
        &self,
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

            lhs = ResolvedExpr::FunctionAppl {
                fn_name: op_name,
                args: vec![lhs, rhs],
                span: lhs_span,
            };
        }

        Ok(lhs)
    }

    fn resolve_primary(
        &self,
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
            Ok(ResolvedExpr::FunctionAppl {
                fn_name: name,
                args,
                span,
            })
        }
    }

    fn resolve_atomic(
        &self,
        exprs: &Vec<ParsedExpr>,
        index: &mut usize,
    ) -> FogResult<ResolvedExpr> {
        let expr = exprs[*index].clone();
        *index += 1;

        match expr {
            ParsedExpr::Block { statements, span } => self.resolve_block(statements, span),

            ParsedExpr::Identifier { name, span } => Ok(ResolvedExpr::Identifier { name, span }),

            ParsedExpr::Literal { literal, span } => Ok(ResolvedExpr::Literal { literal, span }),

            ParsedExpr::Op {
                kind: OpKind::Minus,
                span,
            } => {
                let operand = self.resolve_atomic(exprs, index)?;

                Ok(ResolvedExpr::FunctionAppl {
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
            } => self.resolve_lambda(param_name, param_type, body, span),

            ParsedExpr::Tuple { items, span } => self.resolve_tuple(items, span),

            ParsedExpr::Collection { items: args, .. } => {
                let mut inner_index = 0;
                self.resolve_collection(&args, i32::MIN, &mut inner_index)
            }

            ParsedExpr::Op { .. } => Err(parse_error!(None, "unexpected infix operator")),

            ParsedExpr::Match {
                scrutinee,
                match_arms,
                span,
            } => self.resolve_match(scrutinee, match_arms, span),
        }
    }
}
