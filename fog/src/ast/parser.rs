use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::nodes::*;
use crate::lexer::token::*;

pub struct ASTParser<'a> {
    tokens: &'a Vec<Token>,
    pos: usize,
    binary_ops: HashMap<String, BinaryOp>,
}

pub struct ASTParserError {
    pub message: &'static str,
    pub token_pos: usize,
}

#[derive(Clone)]
pub enum BinaryOpAssociativity {
    Left,
    Right,
}

pub struct BinaryOp {
    pub name: String,
    pub associativity: BinaryOpAssociativity,
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

impl ASTParser<'_> {
    fn new(tokens: &'_ Vec<Token>) -> ASTParser<'_> {
        ASTParser {
            tokens,
            pos: 0,
            binary_ops: [
                ("*", BinaryOpAssociativity::Left, 3),
                ("/", BinaryOpAssociativity::Left, 3),
                ("+", BinaryOpAssociativity::Left, 2),
                ("-", BinaryOpAssociativity::Left, 2),
                ("->", BinaryOpAssociativity::Left, 1),
            ]
            .iter()
            .map(|(sym, assoc, prec)| {
                (
                    sym.to_string(),
                    BinaryOp {
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

    fn get_binary_op(&self, token: &Token) -> Option<&BinaryOp> {
        token_op_key(token).and_then(|key| self.binary_ops.get(key))
    }

    pub fn parse_program(tokens: &Vec<Token>) -> (Box<Program>, Vec<ASTParserError>) {
        let mut parser: ASTParser = ASTParser::new(&tokens);
        let mut statements: Vec<Statement> = Vec::new();
        let mut errors: Vec<ASTParserError> = Vec::new();

        while parser.pos < parser.tokens.len() {
            if let Some(result) = parser.parse_statement() {
                match result {
                    Ok(statement) => statements.push(statement),
                    Err(error) => errors.push(error),
                }
            }
            parser.next();
        }

        let program: Box<Program> = Box::new(Program { statements });

        (program, errors)
    }

    fn parse_statement(&mut self) -> Option<Result<Statement, ASTParserError>> {
        let name: String = match &self.peek().kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => return None,
        };

        self.next();

        let result: Result<Statement, ASTParserError> = match self.peek().kind {
            TokenKind::Colon => {
                self.next();

                self.parse_expression(i32::MIN)
                    .map(|expr| Statement::TypeAnnotation(name, expr))
            }
            TokenKind::Equal => {
                self.next();

                self.parse_expression(i32::MIN)
                    .map(|expr| Statement::Declaration(name, expr))
            }
            _ => Err(ASTParserError {
                message: "expected `:` or `=`",
                token_pos: self.peek().pos,
            }),
        };

        Some(result)
    }

    fn parse_expression(&mut self, min_prec: i32) -> Result<Expr, ASTParserError> {
        let mut lhs: Expr = self.parse_primary()?;

        while self.pos < self.tokens.len() {
            let token: &Token = self.peek();

            let op: &BinaryOp = match self.get_binary_op(token) {
                Some(op) => op,
                None => break,
            };

            if op.precedence < min_prec {
                break;
            }

            let op_name: String = op.name.clone();
            let op_prec: i32 = op.precedence;
            let op_assoc: BinaryOpAssociativity = op.associativity.clone();

            self.next();

            let rhs: Box<Expr> = Box::new(match op_assoc {
                BinaryOpAssociativity::Left => self.parse_expression(op_prec + 1),
                BinaryOpAssociativity::Right => self.parse_expression(op_prec),
            }?);

            lhs = Expr::FuncAppl {
                function: op_name,
                args: vec![Box::new(lhs), rhs],
            };
        }

        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<Expr, ASTParserError> {
        let token: Token = self.peek().clone();
        self.next();

        if let TokenKind::Int32Literal(value) = token.kind {
            return Ok(Expr::Int32Literal(value));
        }

        if let TokenKind::Float32Literal(value) = token.kind {
            return Ok(Expr::Float32Literal(value));
        }

        // TODO: partial application
        // -2 should be negative two
        // - 2 should be x => x - 2

        // update: maybe not?
        if let TokenKind::Minus = token.kind {
            let opnd: Expr = match self.parse_primary() {
                Ok(opnd) => opnd,
                Err(error) => return Err(error),
            };
            return Ok(Expr::FuncAppl {
                function: "-".to_string(),
                args: vec![Box::new(opnd)],
            });
        }

        if let TokenKind::Identifier(name) = token.kind {
            // check for lambda
            if let TokenKind::Colon = self.peek().kind {
                self.next();

                let param_type: Expr = self.parse_expression(i32::MIN)?;

                let TokenKind::FatArrow = self.peek().kind else {
                    return Err(ASTParserError {
                        message: "expected `=>`",
                        token_pos: self.pos,
                    });
                };

                let body: Expr = self.parse_expression(0)?;

                return Ok(Expr::Lambda {
                    param: name,
                    param_type: Box::new(param_type),
                    body: Rc::new(body),
                });
            }

            // check for function application arguments
            let mut args: Vec<Box<Expr>> = Vec::new();
            while self.pos < self.tokens.len() {
                if !is_primary_starter(self.peek()) {
                    break;
                }

                let arg: Expr = self.parse_primary()?;

                args.push(Box::new(arg))
            }

            if args.is_empty() {
                return Ok(Expr::Identifier(name));
            } else {
                return Ok(Expr::FuncAppl {
                    function: name,
                    args,
                });
            }
        }

        if let TokenKind::LeftParenthesis = token.kind {
            let res: Result<Expr, ASTParserError> = self.parse_expression(i32::MIN);

            return match self.peek().kind {
                TokenKind::RightParenthesis => {
                    self.next();
                    res
                }
                _ => Err(ASTParserError {
                    message: "expected `)`",
                    token_pos: token.pos,
                }),
            };
        }

        Err(ASTParserError {
            message: "primary expression parsing error",
            token_pos: token.pos,
        })
    }
}
