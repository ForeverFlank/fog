use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::lexer::token::*;
use crate::parser::parsed_expr::*;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    pos: usize,
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

impl Parser<'_> {
    fn new(tokens: &'_ Vec<Token>) -> Parser<'_> {
        Parser { tokens, pos: 0 }
    }

    fn next(&mut self) {
        self.pos += 1;
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).expect("unexpected EOF")
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

                self.parse_expression()
                    .map(|expr: ParsedExpr| ParsedStatement::TypeAnnotation(name, expr, span))
            }
            TokenKind::Equal => {
                self.next();

                self.parse_expression()
                    .map(|expr: ParsedExpr| ParsedStatement::Declaration(name, expr, span))
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

    fn parse_expression(&mut self) -> FogResult<ParsedExpr> {
        let mut items: Vec<ParsedExpr> = Vec::new();

        loop {
            let atom: ParsedExpr = self.parse_atomic()?;
            items.push(atom);

            let token: &Token = self.peek();

            if let Some(kind) = get_op_kind(token) {
                items.push(ParsedExpr::Op(kind));
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
            Ok(ParsedExpr::Collection(items))
        }
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
            return Ok(ParsedExpr::Op(OpKind::Minus));
        }

        if let TokenKind::Identifier(name) = token.kind {
            // check if it's a lambda
            if let TokenKind::Colon = self.peek().kind {
                self.next();

                let param_type: ParsedExpr = self.parse_expression()?;

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

                let body: ParsedExpr = self.parse_expression()?;

                return Ok(ParsedExpr::Lambda(
                    name,
                    Box::new(param_type),
                    Box::new(body),
                ));
            }

            // if it's not, it's an identifier
            return Ok(ParsedExpr::Identifier(name));
        }

        if let TokenKind::LeftParenthesis = token.kind {
            if let TokenKind::RightParenthesis = self.peek().kind {
                self.next();
                return Ok(ParsedExpr::Tuple(Vec::new()));
            };

            let res: FogResult<ParsedExpr> = self.parse_expression();

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
