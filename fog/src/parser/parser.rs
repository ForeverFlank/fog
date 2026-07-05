use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Pos;
use crate::error::Span;
use crate::lexer::token::*;
use crate::parse_error;
use crate::parser::Literal;
use crate::parser::parsed_expr::ParsedMatchArm;
use crate::parser::parsed_expr::*;

pub fn parse(tokens: &Vec<Token>) -> (Vec<ParsedStatement>, Vec<FogError>) {
    Parser::parse(tokens)
}

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    pos: usize,
    eof_token: Token,
}

impl Parser<'_> {
    fn new(tokens: &'_ Vec<Token>) -> Parser<'_> {
        let eof_pos = tokens.last().map_or(
            Pos {
                line: 1,
                column: 1,
            },
            |t| t.span.end,
        );
        let eof_token = Token {
            kind: TokenKind::Eof,
            span: Span::new(eof_pos, eof_pos),
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

    fn parse(tokens: &Vec<Token>) -> (Vec<ParsedStatement>, Vec<FogError>) {
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

            let prev_pos = parser.pos;

            match parser.parse_block_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    errors.push(e);

                    if parser.pos == prev_pos {
                        parser.next();
                    }
                }
            }

            while let TokenKind::Newline = parser.peek().kind {
                parser.next();
            }
        }

        (statements, errors)
    }

    fn parse_block_statement(&mut self) -> FogResult<ParsedStatement> {
        let start_span = self.peek().span;

        let ahead = self.peek_offset(1).clone();

        match (&self.peek().kind, ahead.kind) {
            (TokenKind::Identifier(name), TokenKind::Colon) => {
                let name = name.clone();

                self.next(); // name
                self.next(); // :

                let expr = self.parse_expression()?;
                let span = Span::merge(start_span, expr.span());

                Ok(ParsedStatement::TypeAnnotation { name, expr, span })
            }

            (TokenKind::Identifier(name), TokenKind::Equal) => {
                let name = name.clone();

                self.next(); // name
                self.next(); // =

                // forgiving newline
                if let TokenKind::Newline = self.peek().kind {
                    self.next();
                }

                let expr = self.parse_expression()?;
                let span = Span::merge(start_span, expr.span());

                Ok(ParsedStatement::Declaration {
                    pattern: ParsedDeclPattern::Identifier {
                        name,
                        span: start_span,
                    },
                    expr,
                    span,
                })
            }

            // either a tuple assignemt, a function clause,
            // or an final operand expression
            _ => {
                let expr = self.parse_expression()?;

                if let TokenKind::Equal = self.peek().kind {
                    self.next();

                    let pattern = expr.into_decl_pattern()?;
                    let expr = self.parse_expression()?;
                    let span = Span::merge(pattern.span(), expr.span());

                    Ok(ParsedStatement::Declaration {
                        pattern,
                        expr,
                        span,
                    })
                } else {
                    let span = expr.span();
                    Ok(ParsedStatement::Expression { expr, span })
                }
            }
        }
    }

    fn parse_expression(&mut self) -> FogResult<ParsedExpr> {
        let mut args = Vec::new();
        let start_span = self.peek().span;

        loop {
            let atom = self.parse_atomic()?;
            args.push(atom);

            let token = self.peek();

            if let Some(kind) = OpKind::from_token(token) {
                let op_span = token.span;
                args.push(ParsedExpr::Op {
                    kind,
                    span: op_span,
                });
                self.next();
            } else if token.kind.is_primary_starter() {
                continue;
            } else {
                break;
            }
        }

        if args.len() == 1 {
            Ok(args[0].clone())
        } else {
            let span = Span::merge(start_span, args.last().unwrap().span());
            Ok(ParsedExpr::Collection { items: args, span })
        }
    }

    fn parse_atomic(&mut self) -> FogResult<ParsedExpr> {
        let token = self.peek().clone();
        let span = token.span;

        match token.kind {
            TokenKind::Int32Literal(value) => {
                self.next();
                Ok(ParsedExpr::Literal {
                    literal: Literal::Int32(value),
                    span,
                })
            }

            TokenKind::Float32Literal(value) => {
                self.next();
                Ok(ParsedExpr::Literal {
                    literal: Literal::Float32(value),
                    span,
                })
            }

            // unary minus (negation)
            TokenKind::Minus => {
                self.next();
                Ok(ParsedExpr::Op {
                    kind: OpKind::Minus,
                    span,
                })
            }

            TokenKind::Identifier(name) => {
                self.next();

                // check for lambda with type annotation
                if let TokenKind::Colon = self.peek().kind {
                    self.next();

                    let param_type = self.parse_expression()?;

                    let TokenKind::FatArrow = self.peek().kind else {
                        return Err(parse_error!(Some(self.peek().span), "expected `=>`"));
                    };
                    self.next();

                    // forgiving newline
                    if let TokenKind::Newline = self.peek().kind {
                        self.next();
                    }

                    let body = self.parse_expression()?;
                    let span = Span::merge(span, body.span());

                    return Ok(ParsedExpr::Lambda {
                        param_name: name,
                        param_type: param_type.into(),
                        body: body.into(),
                        span,
                    });
                }

                Ok(ParsedExpr::Identifier { name, span })
            }

            // tuple
            TokenKind::LeftParenthesis => {
                self.next();

                if let TokenKind::RightParenthesis = self.peek().kind {
                    let close_span = self.peek().span;
                    self.next();
                    return Ok(ParsedExpr::Tuple {
                        items: Vec::new(),
                        span: Span::merge(span, close_span),
                    });
                }

                let mut items: Vec<ParsedExpr> = Vec::new();

                loop {
                    let expr = self.parse_expression()?;
                    items.push(expr);

                    match self.peek().kind {
                        TokenKind::RightParenthesis => {
                            let close_span = self.peek().span;
                            self.next();

                            if items.len() == 1 {
                                return Ok(items[0].clone());
                            } else {
                                return Ok(ParsedExpr::Tuple {
                                    items,
                                    span: Span::merge(span, close_span),
                                });
                            }
                        }

                        TokenKind::Comma => {
                            self.next();
                            continue;
                        }

                        _ => {
                            return Err(parse_error!(Some(token.span), "expected `)`"));
                        }
                    }
                }
            }

            // block statement
            TokenKind::LeftBrace => {
                self.next();
                let (statements, close_span) = self.parse_block()?;
                Ok(ParsedExpr::Block {
                    statements,
                    span: Span::merge(span, close_span),
                })
            }

            // match
            TokenKind::Match => {
                self.next();

                let scrutinee = Box::new(self.parse_expression()?);

                let TokenKind::LeftBrace = self.peek().kind else {
                    return Err(parse_error!(Some(self.peek().span), "expected `{{`"));
                };
                self.next();

                let (match_arms, close_span) = self.parse_match_arms()?;

                Ok(ParsedExpr::Match {
                    scrutinee,
                    match_arms,
                    span: Span::merge(span, close_span),
                })
            }

            _ => Err(parse_error!(Some(span), "atomic expression parsing error")),
        }
    }

    fn parse_match_arms(&mut self) -> FogResult<(Vec<ParsedMatchArm>, Span)> {
        let mut arms = Vec::new();

        while let TokenKind::Newline = self.peek().kind {
            self.next();
        }

        loop {
            if let TokenKind::Eof = self.peek().kind {
                return Err(parse_error!(None, "unclosed match"));
            }

            if let TokenKind::RightBrace = self.peek().kind {
                let close_span = self.peek().span;
                self.next();
                return Ok((arms, close_span));
            }

            let pattern = self.parse_expression()?;

            let TokenKind::FatArrow = self.peek().kind else {
                return Err(parse_error!(Some(self.peek().span), "expected `=>`"));
            };
            self.next();

            // forgive newline
            if let TokenKind::Newline = self.peek().kind {
                self.next();
            }

            let value_expr = self.parse_expression()?;

            arms.push(ParsedMatchArm {
                pattern,
                value_expr,
            });

            while let TokenKind::Newline = self.peek().kind {
                self.next();
            }
        }
    }

    fn parse_block(&mut self) -> FogResult<(Vec<ParsedStatement>, Span)> {
        let mut statements = Vec::new();

        while let TokenKind::Newline = self.peek().kind {
            self.next();
        }

        loop {
            if let TokenKind::RightBrace = self.peek().kind {
                let close_span = self.peek().span;
                self.next();
                return Ok((statements, close_span));
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
    }
}
