use std::collections::HashMap;

use crate::{ast_nodes::*, lexer::*};

struct ASTParser {
    tokens: Vec<Token>,
    pos: usize,
    binary_ops: HashMap<String, BinaryOp>,
}

pub struct ASTParserError {
    pub message: &'static str,
    pub token: Token,
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
        | TokenKind::IntLiteral(_)
        | TokenKind::FloatLiteral(_)
        | TokenKind::LeftParenthesis
        | TokenKind::Minus => true,
        _ => false,
    }
}

pub fn parse_program(tokens: Vec<Token>) -> (Program, Vec<ASTParserError>) {
    ASTParser::parse_program(tokens)
}

impl ASTParser {
    fn new(tokens: Vec<Token>) -> ASTParser {
        ASTParser {
            tokens: tokens,
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
        self.tokens.get(self.pos).expect("Unexpected EOF")
    }

    fn get_binary_op(&self, token: &Token) -> Option<&BinaryOp> {
        token_op_key(token).and_then(|key| self.binary_ops.get(key))
    }

    fn parse_program(tokens: Vec<Token>) -> (Program, Vec<ASTParserError>) {
        let mut parser: ASTParser = ASTParser::new(tokens);
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

        let program: Program = Program { statements };

        (program, errors)
    }

    fn parse_statement(&mut self) -> Option<Result<Statement, ASTParserError>> {
        let identifier: Identifier = match &self.peek().kind {
            TokenKind::Identifier(name) => Identifier(name.clone()),
            _ => return None,
        };

        self.next();

        let result: Result<Statement, ASTParserError> = match self.peek().kind {
            TokenKind::Colon => {
                self.next();

                let result: Result<Expr, ASTParserError> = self.parse_expression(i32::MIN);

                match result {
                    Ok(expr) => Ok(Statement::TypeAnnotation(identifier, expr)),
                    Err(error) => Err(error),
                }
            }
            TokenKind::Equal => {
                self.next();

                let result: Result<Expr, ASTParserError> = self.parse_expression(i32::MIN);

                match result {
                    Ok(expr) => Ok(Statement::Declaration(identifier, expr)),
                    Err(error) => Err(error),
                }
            }
            _ => Err(ASTParserError {
                message: "Expected ':' or '='",
                token: self.peek().clone(),
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

            let rhs_result: Result<Expr, ASTParserError> = match op_assoc {
                BinaryOpAssociativity::Left => self.parse_expression(op_prec + 1),
                BinaryOpAssociativity::Right => self.parse_expression(op_prec),
            };

            let rhs: Box<Expr> = Box::new(match rhs_result {
                Ok(expr) => expr,
                Err(error) => return Err(error),
            });

            lhs = Expr::FuncAppl(FuncAppl {
                function: Identifier(op_name),
                arguments: vec![Box::new(lhs), rhs],
            });
        }

        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<Expr, ASTParserError> {
        let token: Token = self.peek().clone();
        self.next();

        if let TokenKind::IntLiteral(value) = token.kind {
            return Ok(Expr::IntLiteral(value));
        }

        if let TokenKind::FloatLiteral(value) = token.kind {
            return Ok(Expr::FloatLiteral(value));
        }

        // TODO: partial application
        // -2 should be negative two
        // - 2 should be x => x - 2
        if let TokenKind::Minus = token.kind {
            let opnd: Expr = match self.parse_primary() {
                Ok(opnd) => opnd,
                Err(error) => return Err(error),
            };
            return Ok(Expr::FuncAppl(FuncAppl {
                function: Identifier::new("-"),
                arguments: vec![Box::new(opnd)],
            }));
        }

        if let TokenKind::Identifier(name) = token.kind {
            // check for lambda
            if let TokenKind::FatArrow = self.peek().kind {
                self.next();

                let body: Expr = match self.parse_expression(0) {
                    Ok(body) => body,
                    Err(error) => return Err(error),
                };

                return Ok(Expr::Lambda(Lambda {
                    parameter: Identifier(name),
                    body: Box::new(body),
                }));
            }

            // check for arguments
            let mut args: Vec<Box<Expr>> = Vec::new();
            while self.pos < self.tokens.len() {
                if !is_primary_starter(self.peek()) {
                    break;
                }

                let arg: Expr = match self.parse_primary() {
                    Ok(arg) => arg,
                    Err(error) => return Err(error),
                };

                args.push(Box::new(arg))
            }

            if args.is_empty() {
                return Ok(Expr::Identifier(Identifier(name)));
            } else {
                return Ok(Expr::FuncAppl(FuncAppl {
                    function: Identifier(name),
                    arguments: args,
                }));
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
                    message: "Expected ')'",
                    token,
                }),
            };
        }

        Err(ASTParserError {
            message: "Primary expression parsing error",
            token,
        })
    }
}
