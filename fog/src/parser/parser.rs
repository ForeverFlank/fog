use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::lexer::token::*;
use crate::parser::parsed_expr::*;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    pos: usize,
    eof_token: Token,
}

fn get_op_kind(token: &Token) -> Option<OpKind> {
    match &token.kind {
        TokenKind::Plus => Some(OpKind::Plus),
        TokenKind::Minus => Some(OpKind::Minus),
        TokenKind::Star => Some(OpKind::Star),
        TokenKind::Slash => Some(OpKind::Slash),
        TokenKind::Arrow => Some(OpKind::Arrow),
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

fn token_span(token: &Token) -> Span {
    Span {
        pos: token.pos,
        line: token.line,
        column: token.column,
    }
}

impl Parser<'_> {
    fn new(tokens: &'_ Vec<Token>) -> Parser<'_> {
        let eof_token = Token {
            kind: TokenKind::Eof,
            pos: tokens.last().map_or(0, |t| t.pos + 1),
            line: tokens.last().map_or(1, |t| t.line),
            column: tokens.last().map_or(1, |t| t.column + 1),
        };
        Parser {
            tokens,
            pos: 0,
            eof_token,
        }
    }

    fn next(&mut self) {
        self.pos += 1;
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.eof_token)
    }

    pub fn parse(tokens: &Vec<Token>) -> (Vec<ParsedStatement>, Vec<FogError>) {
        let mut parser = Parser::new(&tokens);
        let mut statements = Vec::new();
        let mut errors = Vec::new();

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
        let name = match &self.peek().kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => return None,
        };

        let span = token_span(self.peek());

        self.next();

        let result: FogResult<ParsedStatement> = match self.peek().kind {
            TokenKind::Colon => {
                self.next();

                self.parse_expression()
                    .map(|expr| ParsedStatement::TypeAnnotation { name, expr, span })
            }
            TokenKind::Equal => {
                self.next();

                self.parse_expression()
                    .map(|expr| ParsedStatement::Declaration { name, expr, span })
            }
            _ => Err(FogError::parse(
                "expected `:` or `=`".to_string(),
                Some(token_span(self.peek())),
            )),
        };

        Some(result)
    }

    fn parse_expression(&mut self) -> FogResult<ParsedExpr> {
        let mut items = Vec::new();

        loop {
            let atom = self.parse_atomic()?;
            items.push(atom);

            let token = self.peek();

            if let Some(kind) = get_op_kind(token) {
                items.push(ParsedExpr::Op { kind });
                self.next();
            } else if is_primary_starter(token) {
                continue;
            } else {
                break;
            }
        }

        if items.len() == 1 {
            Ok(items[0].clone())
        } else {
            Ok(ParsedExpr::Collection { items })
        }
    }

    fn parse_atomic(&mut self) -> FogResult<ParsedExpr> {
        let token = self.peek().clone();
        self.next();

        match token.kind {
            TokenKind::Int32Literal(value) => Ok(ParsedExpr::Int32Literal { value }),
            TokenKind::Float32Literal(value) => Ok(ParsedExpr::Float32Literal { value }),
            TokenKind::Minus => Ok(ParsedExpr::Op {
                kind: OpKind::Minus,
            }),

            TokenKind::Identifier(name) => {
                if let TokenKind::Colon = self.peek().kind {
                    self.next();

                    let param_type = self.parse_expression()?;

                    let TokenKind::FatArrow = self.peek().kind else {
                        return Err(FogError::parse(
                            "expected `=>`".to_string(),
                            Some(token_span(self.peek())),
                        ));
                    };
                    self.next();

                    let body = self.parse_expression()?;

                    return Ok(ParsedExpr::Lambda {
                        param_name: name,
                        param_type: Box::new(param_type),
                        body: Box::new(body),
                    });
                }

                Ok(ParsedExpr::Identifier { name })
            }

            TokenKind::LeftParenthesis => {
                if let TokenKind::RightParenthesis = self.peek().kind {
                    self.next();

                    return Ok(ParsedExpr::Tuple { items: Vec::new() });
                }

                let mut items: Vec<ParsedExpr> = Vec::new();

                loop {
                    let expr = self.parse_expression()?;
                    items.push(expr);

                    match self.peek().kind {
                        TokenKind::RightParenthesis => {
                            self.next();

                            if items.len() == 1 {
                                return Ok(items[0].clone());
                            } else {
                                return Ok(ParsedExpr::Tuple { items });
                            }
                        }

                        TokenKind::Comma => {
                            self.next();

                            continue;
                        }

                        _ => {
                            return Err(FogError::parse(
                                "expected `)`".to_string(),
                                Some(token_span(&token)),
                            ));
                        }
                    }
                }
            }

            _ => Err(FogError::parse(
                "atomic expression parsing error".to_string(),
                Some(token_span(&token)),
            )),
        }
    }
}
