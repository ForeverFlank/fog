use std::collections::HashMap;
use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::lexer::token::*;
use crate::parser::parsed_expr::*;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    pos: usize,
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

fn token_op_key(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Plus => Some("+"),
        TokenKind::Minus => Some("-"),
        TokenKind::Star => Some("*"),
        TokenKind::Slash => Some("/"),
        TokenKind::Arrow => Some("->"),
        TokenKind::Identifier(name) => Some(name.as_str()),
        _ => None,
    }
}

fn is_primary_starter(token: &Token) -> bool {
    match token.kind {
        TokenKind::Identifier(_)
        | TokenKind::Int32Literal(_)
        | TokenKind::Float32Literal(_)
        | TokenKind::LeftParenthesis
        | TokenKind::Minus => true,
        _ => false,
    }
}

impl Parser<'_> {
    fn new(tokens: &'_ Vec<Token>) -> Parser<'_> {
        Parser {
            tokens,
            pos: 0,
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

    fn next(&mut self) {
        self.pos += 1;
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).expect("unexpected EOF")
    }

    fn get_binary_op(&self, token: &Token) -> Option<&InfixOp> {
        token_op_key(token).and_then(|key| self.infix_ops.get(key))
    }

    pub fn parse(tokens: &Vec<Token>) -> (Vec<ParsedStatement>, Vec<FogError>) {
        let mut parser: Parser = Parser::new(&tokens);
        let mut statements: Vec<ParsedStatement> = Vec::new();
        let mut errors: Vec<FogError> = Vec::new();

        while parser.pos < parser.tokens.len() {
            if let Some(result) = parser.parse_statement() {
                match result {
                    Ok(stmt) => statements.push(stmt),
                    Err(error) => errors.push(error),
                }
            }
            parser.next();
        }

        (statements, errors)
    }

    fn parse_statement(&mut self) -> Option<FogResult<ParsedStatement>> {
        let name: String = match &self.peek().kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => return None,
        };

        let span: Span = Span {
            pos: self.peek().pos,
            line: self.peek().line,
            column: self.peek().column,
        };

        self.next();

        let result: FogResult<ParsedStatement> = match self.peek().kind {
            TokenKind::Colon => {
                self.next();

                self.parse_expression(i32::MIN)
                    .map(|expr| ParsedStatement::TypeAnnotation(name, expr, span))
            }
            TokenKind::Equal => {
                self.next();

                self.parse_expression(i32::MIN)
                    .map(|expr| ParsedStatement::Declaration(name, expr, span))
            }
            _ => Err(FogError::parse(
                "expected `:` or `=`".to_string(),
                Some(Span {
                    pos: self.peek().pos,
                    line: self.peek().line,
                    column: self.peek().column,
                }),
            )),
        };

        Some(result)
    }

    fn parse_expression(&mut self, min_prec: i32) -> FogResult<ParsedExpr> {
        let mut lhs: ParsedExpr = self.parse_primary()?;

        while self.pos < self.tokens.len() {
            let token: &Token = self.peek();

            let op: &InfixOp = match self.get_binary_op(token) {
                Some(op) => op,
                None => break,
            };

            if op.precedence < min_prec {
                break;
            }

            let op_name: String = op.name.clone();
            let op_prec: i32 = op.precedence;
            let op_assoc: OpAssociativity = op.associativity.clone();

            self.next();

            let rhs: ParsedExpr = match op_assoc {
                OpAssociativity::Left => self.parse_expression(op_prec + 1),
                OpAssociativity::Right => self.parse_expression(op_prec),
            }?;

            lhs = ParsedExpr::FuncAppl(op_name, vec![lhs, rhs]);
        }

        Ok(lhs)
    }

    fn parse_primary(&mut self) -> FogResult<ParsedExpr> {
        let head: ParsedExpr = self.parse_atomic()?;

        // check for function application
        if let ParsedExpr::Identifier(name) = &head {
            let mut args: Vec<ParsedExpr> = Vec::new();

            while self.pos < self.tokens.len() && is_primary_starter(self.peek()) {
                let arg: ParsedExpr = self.parse_atomic()?;
                args.push(arg);
            }

            if !args.is_empty() {
                return Ok(ParsedExpr::FuncAppl(name.clone(), args));
            }
        }

        Ok(head)
    }

    fn parse_atomic(&mut self) -> FogResult<ParsedExpr> {
        let token: Token = self.peek().clone();
        self.next();

        if let TokenKind::Int32Literal(value) = token.kind {
            return Ok(ParsedExpr::Int32Literal(value));
        }

        if let TokenKind::Float32Literal(value) = token.kind {
            return Ok(ParsedExpr::Float32Literal(value));
        }

        if let TokenKind::Minus = token.kind {
            let opnd: ParsedExpr = self.parse_primary()?;

            return Ok(ParsedExpr::FuncAppl("-".to_string(), vec![opnd]));
        }

        if let TokenKind::Identifier(name) = token.kind {
            // check if it's a lambda
            if let TokenKind::Colon = self.peek().kind {
                self.next();

                let param_type: ParsedExpr = self.parse_expression(i32::MIN)?;

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

                let body: ParsedExpr = self.parse_expression(i32::MIN)?;

                return Ok(ParsedExpr::Lambda(
                    name,
                    Box::new(param_type),
                    Rc::new(body),
                ));
            }

            // otherwise, it's an identifier
            return Ok(ParsedExpr::Identifier(name));
        }

        if let TokenKind::LeftParenthesis = token.kind {
            if let TokenKind::RightParenthesis = self.peek().kind {
                self.next();
                return Ok(ParsedExpr::Tuple(Vec::new()));
            };

            let res: FogResult<ParsedExpr> = self.parse_expression(i32::MIN);

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
