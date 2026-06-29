use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::lexer::token::*;
use crate::parse_error;
use crate::parser::parsed_expr::{MatchArm, *};

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

    fn peek_offset(&self, offset: i32) -> &Token {
        self.tokens
            .get((self.pos as i32 + offset) as usize)
            .unwrap_or(&self.eof_token)
    }

    pub fn parse(tokens: &Vec<Token>) -> (Vec<ParsedStatement>, Vec<FogError>) {
        let mut parser = Parser::new(&tokens);
        let mut statements = Vec::new();
        let mut errors = Vec::new();

        while let TokenKind::Newline = parser.peek().kind {
            parser.next();
        }

        loop {
            if let TokenKind::Eof = parser.peek().kind {
                break;
            }
            match parser.parse_block_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => errors.push(e),
            }
            while let TokenKind::Newline = parser.peek().kind {
                parser.next();
            }
        }

        (statements, errors)
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

        match token.kind {
            TokenKind::Int32Literal(value) => {
                self.next();

                Ok(ParsedExpr::Int32Literal { value })
            }

            TokenKind::Float32Literal(value) => {
                self.next();

                Ok(ParsedExpr::Float32Literal { value })
            }

            // unary minus (negation)
            TokenKind::Minus => {
                self.next();

                Ok(ParsedExpr::Op {
                    kind: OpKind::Minus,
                })
            }

            TokenKind::Identifier(name) => {
                self.next();

                // check for lambda with type annotation
                if let TokenKind::Colon = self.peek().kind {
                    self.next();

                    let param_type = self.parse_expression()?;

                    let TokenKind::FatArrow = self.peek().kind else {
                        return Err(parse_error!(Some(token_span(self.peek())), "expected `=>`"));
                    };
                    self.next();

                    let body = self.parse_expression()?;

                    return Ok(ParsedExpr::Lambda {
                        param_name: name,
                        param_type: param_type.into(),
                        body: body.into(),
                    });
                }

                Ok(ParsedExpr::Identifier { name })
            }

            // tuple
            TokenKind::LeftParenthesis => {
                self.next();

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
                            return Err(parse_error!(Some(token_span(&token)), "expected `)`"));
                        }
                    }
                }
            }

            // block statement
            TokenKind::LeftBrace => {
                self.next();

                let statements = self.parse_block()?;
                Ok(ParsedExpr::Block { statements })
            }

            // match
            TokenKind::Match => {
                self.next();

                let expr = self.parse_expression()?;

                let TokenKind::LeftBrace = self.peek().kind else {
                    return Err(parse_error!(Some(token_span(self.peek())), "expected `{{`"));
                };
                self.next();

                let match_arms = self.parse_match_arms()?;

                Ok(ParsedExpr::Match {
                    expr: expr.into(),
                    match_arms,
                })
            }

            _ => Err(parse_error!(
                Some(token_span(&token)),
                "atomic expression parsing error"
            )),
        }
    }

    fn parse_match_arms(&mut self) -> FogResult<Vec<MatchArm>> {
        let mut arms = Vec::new();

        while let TokenKind::Newline = self.peek().kind {
            self.next();
        }

        loop {
            if let TokenKind::RightBrace = self.peek().kind {
                self.next();
                break;
            }
            if let TokenKind::Eof = self.peek().kind {
                return Err(parse_error!(None, "unclosed match"));
            }

            let pattern = self.parse_expression()?;

            let TokenKind::FatArrow = self.peek().kind else {
                return Err(parse_error!(Some(token_span(self.peek())), "expected `=>`"));
            };
            self.next();

            let value_expr = self.parse_expression()?;

            arms.push(MatchArm {
                pattern,
                value_expr,
            });

            while let TokenKind::Newline = self.peek().kind {
                self.next();
            }
        }

        Ok(arms)
    }

    fn parse_block(&mut self) -> FogResult<Vec<ParsedStatement>> {
        let mut statements = Vec::new();

        while let TokenKind::Newline = self.peek().kind {
            self.next();
        }

        loop {
            if let TokenKind::RightBrace = self.peek().kind {
                self.next();
                break;
            }
            if let TokenKind::Eof = self.peek().kind {
                return Err(parse_error!(None, "unclosed block"));
            }

            let stmt = self.parse_block_statement()?;
            statements.push(stmt);

            while let TokenKind::Newline = self.peek().kind {
                self.next();
            }
        }

        Ok(statements)
    }

    fn parse_block_statement(&mut self) -> FogResult<ParsedStatement> {
        let span = token_span(self.peek());

        let ahead = self.peek_offset(1).clone();

        match (&self.peek().kind, ahead.kind) {
            (TokenKind::Identifier(name), TokenKind::Colon) => {
                let name = name.clone();

                self.next(); // name
                self.next(); // :

                let expr = self.parse_expression()?;

                Ok(ParsedStatement::TypeAnnotation { name, expr, span })
            }

            (TokenKind::Identifier(name), TokenKind::Equal) => {
                let name = name.clone();

                self.next(); // name
                self.next(); // =

                let expr = self.parse_expression()?;

                Ok(ParsedStatement::Declaration { name, expr, span })
            }

            _ => {
                let expr = self.parse_expression()?;

                Ok(ParsedStatement::Expression { expr, span })
            }
        }
    }
}
