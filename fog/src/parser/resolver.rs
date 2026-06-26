use std::collections::HashMap;
use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::parser::parsed_expr::OpKind;
use crate::parser::parsed_expr::ParsedExpr;
use crate::parser::parsed_expr::ParsedStatement;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::parser::resolved_expr::ResolvedStatement;

pub struct Resolver {
    infix_ops: HashMap<String, InfixOp>,
}

#[derive(Clone)]
pub enum OpAssociativity {
    Left,
    Right,
}

pub struct InfixOp {
    pub name: String,
    pub associativity: OpAssociativity,
    pub precedence: i32,
}

// fn token_op_key(token: &Token) -> Option<&str> {
//     match &token.kind {
//         TokenKind::Plus => Some("+"),
//         TokenKind::Minus => Some("-"),
//         TokenKind::Star => Some("*"),
//         TokenKind::Slash => Some("/"),
//         TokenKind::Arrow => Some("->"),
//         TokenKind::Identifier(name) => Some(name.as_str()),
//         _ => None,
//     }
// }

fn is_primary_starter(parsed_expr: &ParsedExpr) -> bool {
    match parsed_expr {
        ParsedExpr::Identifier(_) | ParsedExpr::Int32Literal(_) | ParsedExpr::Float32Literal(_) => {
            true
        }

        ParsedExpr::Op(kind) => match kind {
            OpKind::Minus => true,
            _ => false,
        },

        _ => false,
    }
}

impl Resolver {
    fn new() -> Self {
        Self {
            infix_ops: [
                ("*", OpAssociativity::Left, 3),
                ("/", OpAssociativity::Left, 3),
                ("+", OpAssociativity::Left, 2),
                ("-", OpAssociativity::Left, 2),
                ("->", OpAssociativity::Right, 1),
            ]
            .iter()
            .map(|(sym, assoc, prec)| {
                (
                    sym.to_string(),
                    InfixOp {
                        name: sym.to_string(),
                        associativity: assoc.clone(),
                        precedence: *prec,
                    },
                )
            })
            .collect(),
        }
    }

    fn get_binary_op(&self, parsed_expr: &ParsedExpr) -> Option<&InfixOp> {
        token_op_key(token).and_then(|key: &str| self.infix_ops.get(key))
    }

    pub fn resolve(
        parsed_statements: Vec<ParsedStatement>,
    ) -> (Vec<ResolvedStatement>, Vec<FogError>) {
        let mut resolved_statements: Vec<ResolvedStatement> = Vec::new();
        let mut errors: Vec<FogError> = Vec::new();

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
            ParsedStatement::TypeAnnotation(name, parsed_expr, span) => Ok(
                ResolvedStatement::TypeAnnotation(name, Self::resolve_expr(parsed_expr)?, span),
            ),
            ParsedStatement::Declaration(name, parsed_expr, span) => Ok(
                ResolvedStatement::Declaration(name, Self::resolve_expr(parsed_expr)?, span),
            ),
        }
    }

    fn resolve_expr(parsed_expr: ParsedExpr) -> FogResult<ResolvedExpr> {
        match parsed_expr {
            ParsedExpr::Identifier(name) => Ok(ResolvedExpr::Identifier(name)),
            ParsedExpr::Op(_) => unreachable!(),

            ParsedExpr::Int32Literal(value) => Ok(ResolvedExpr::Int32Literal(value)),
            ParsedExpr::Float32Literal(value) => Ok(ResolvedExpr::Float32Literal(value)),

            ParsedExpr::Lambda(param_name, param_type, body) => Ok(ResolvedExpr::Lambda(
                param_name,
                Box::new(Self::resolve_expr(*param_type)?),
                Rc::new(Self::resolve_expr(*body)?),
            )),

            ParsedExpr::Tuple(exprs) => Ok(ResolvedExpr::Tuple(
                exprs
                    .iter()
                    .map(|expr: &ParsedExpr| Ok(Self::resolve_expr(expr.clone())?.into()))
                    .collect::<Result<Vec<ResolvedExpr>, FogError>>()?,
            )),

            ParsedExpr::Collection(exprs) => Self::resolve_collection(exprs, i32::MIN),
        }
    }

    fn resolve_collection(parsed_exprs: Vec<ParsedExpr>, min_prec: i32) -> FogResult<ResolvedExpr> {
        let mut index: usize = 0;
        let mut lhs: ResolvedExpr = Self::resolve_prefix(parsed_exprs, &mut index)?;

        loop {
            let curr: ParsedExpr = parsed_exprs[index];

            let op: &InfixOp = match self.get_binary_op(curr) {
                Some(op) => op,
                None => break,
            };

            if op.precedence < min_prec {
                break;
            }

            let op_name: String = op.name.clone();
            let op_prec: i32 = op.precedence;
            let op_assoc: OpAssociativity = op.associativity.clone();

            index += 1;

            let rhs: ResolvedExpr = match op_assoc {
                OpAssociativity::Left => Self::resolve_collection(parsed_exprs[index], op_prec + 1),
                OpAssociativity::Right => Self::resolve_collection(parsed_exprs[index], op_prec),
            }?;

            lhs = ResolvedExpr::FuncAppl(op_name, vec![lhs, rhs]);
        }

        Ok(lhs)
    }

    fn resolve_prefix(parsed_exprs: Vec<ParsedExpr>, index: &mut usize) -> FogResult<ResolvedExpr> {
    }

    fn resolve_primary(&mut self) -> FogResult<ResolvedExpr> {
        let head: ResolvedExpr = self.resolve_atomic()?;

        // check for function application
        if let ResolvedExpr::Identifier(name) = &head {
            let mut args: Vec<ResolvedExpr> = Vec::new();

            while self.pos < self.tokens.len() && is_primary_starter(self.peek()) {
                let arg: ResolvedExpr = self.resolve_atomic()?;
                args.push(arg);
            }

            if !args.is_empty() {
                return Ok(ResolvedExpr::FuncAppl(name.clone(), args));
            }
        }

        Ok(head)
    }

    fn resolve_atomic(&mut self) -> FogResult<ResolvedExpr> {
        let token: Token = self.peek().clone();
        self.next();

        if let TokenKind::Int32Literal(value) = token.kind {
            return Ok(ResolvedExpr::Int32Literal(value));
        }

        if let TokenKind::Float32Literal(value) = token.kind {
            return Ok(ResolvedExpr::Float32Literal(value));
        }

        if let TokenKind::Minus = token.kind {
            let opnd: ResolvedExpr = self.parse_primary()?;

            return Ok(ResolvedExpr::FuncAppl("-".to_string(), vec![opnd]));
        }

        if let TokenKind::Identifier(name) = token.kind {
            // check if it's a lambda
            if let TokenKind::Colon = self.peek().kind {
                self.next();

                let param_type: ResolvedExpr = self.parse_expression(i32::MIN)?;

                let TokenKind::FatArrow = self.peek().kind else {
                    return Err(FogError::parse(
                        "expected `=>`".to_string(),
                        Some(Span {
                            pos: self.peek().pos,
                            line: self.peek().line,
                            column: self.peek().column,
                        }),
                    ));
                };
                self.next();

                let body: ResolvedExpr = self.parse_expression(i32::MIN)?;

                return Ok(ResolvedExpr::Lambda(
                    name,
                    Box::new(param_type),
                    Box::new(body),
                ));
            }

            // otherwise, it's an identifier
            return Ok(ResolvedExpr::Identifier(name));
        }

        if let TokenKind::LeftParenthesis = token.kind {
            if let TokenKind::RightParenthesis = self.peek().kind {
                self.next();
                return Ok(ResolvedExpr::Tuple(Vec::new()));
            };

            let res: FogResult<ResolvedExpr> = self.parse_expression(i32::MIN);

            return match self.peek().kind {
                TokenKind::RightParenthesis => {
                    self.next();
                    res
                }
                _ => Err(FogError::parse(
                    "expected `)`".to_string(),
                    Some(Span {
                        pos: token.pos,
                        line: token.line,
                        column: token.column,
                    }),
                )),
            };
        }

        Err(FogError::parse(
            "atomic expression parsing error".to_string(),
            Some(Span {
                pos: token.pos,
                line: token.line,
                column: token.column,
            }),
        ))
    }
}
