use crate::error::FogError;
use crate::error::FogResult;
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

fn expr_to_decl_pattern(expr: ParsedExpr) -> FogResult<ParsedDeclPattern> {
    match expr {
        ParsedExpr::Identifier { name, span } => Ok(ParsedDeclPattern::Identifier { name, span }),

        ParsedExpr::Tuple { items, span } => Ok(ParsedDeclPattern::Tuple {
            items: items
                .into_iter()
                .map(expr_to_decl_pattern)
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),

        ParsedExpr::Collection { items, span } => Ok(ParsedDeclPattern::Collection {
            items: items
                .into_iter()
                .map(expr_to_decl_pattern)
                .collect::<Result<Vec<_>, _>>()?,
            span,
        }),

        ParsedExpr::Op { kind } => Ok(ParsedDeclPattern::Op { kind }),

        ParsedExpr::Int32Literal { value, span } => {
            Ok(ParsedDeclPattern::Int32Literal { value, span })
        }
        ParsedExpr::Float32Literal { value, span } => {
            Ok(ParsedDeclPattern::Float32Literal { value, span })
        }

        _ => Err(parse_error!(Some(expr.span()), "invalid pattern")),
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

                // forgiving newline
                if let TokenKind::Newline = self.peek().kind {
                    self.next();
                }

                let expr = self.parse_expression()?;

                Ok(ParsedStatement::Declaration {
                    pattern: ParsedDeclPattern::Identifier { name, span: span },
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

                    let pattern = Self::expr_to_decl_pattern(expr)?;
                    let expr = self.parse_expression()?;

                    Ok(ParsedStatement::Declaration {
                        pattern,
                        expr,
                        span,
                    })
                } else {
                    Ok(ParsedStatement::Expression { expr, span })
                }
            }
        }
    }

    fn expr_to_decl_pattern(expr: ParsedExpr) -> FogResult<ParsedDeclPattern> {
        match expr {
            ParsedExpr::Identifier { name, span } => {
                Ok(ParsedDeclPattern::Identifier { name, span })
            }

            ParsedExpr::Literal { literal, span } => {
                Ok(ParsedDeclPattern::Literal { literal, span })
            }

            ParsedExpr::Tuple { items, span } => Ok(ParsedDeclPattern::Tuple {
                items: items
                    .into_iter()
                    .map(|item| Self::expr_to_decl_pattern(item))
                    .collect::<Result<Vec<_>, _>>()?,
                span,
            }),

            ParsedExpr::Collection { args, span } => Ok(ParsedDeclPattern::Collection {
                items: args
                    .into_iter()
                    .map(|item| Self::expr_to_decl_pattern(item))
                    .collect::<Result<Vec<_>, _>>()?,
                span,
            }),

            ParsedExpr::Block { .. }
            | ParsedExpr::Op { .. }
            | ParsedExpr::Lambda { .. }
            | ParsedExpr::Match { .. } => Err(parse_error!(
                Some(expr.span()),
                "invalid declaration pattern"
            )),
        }
    }

    fn parse_expression(&mut self) -> FogResult<ParsedExpr> {
        let mut args = Vec::new();
        let span = token_span(self.peek());

        loop {
            let atom = self.parse_atomic()?;
            args.push(atom);

            let token = self.peek();

            if let Some(kind) = get_op_kind(token) {
                args.push(ParsedExpr::Op { kind, span: span });
                self.next();
            } else if is_primary_starter(token) {
                continue;
            } else {
                break;
            }
        }

        if args.len() == 1 {
            Ok(args[0].clone())
        } else {
            Ok(ParsedExpr::Collection { args, span })
        }
    }

    fn parse_atomic(&mut self) -> FogResult<ParsedExpr> {
        let token = self.peek().clone();
        let span = token_span(&token);

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
                        return Err(parse_error!(Some(token_span(self.peek())), "expected `=>`"));
                    };
                    self.next();

                    // forgiving newline
                    if let TokenKind::Newline = self.peek().kind {
                        self.next();
                    }

                    let body = self.parse_expression()?;

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
                    self.next();
                    return Ok(ParsedExpr::Tuple {
                        items: Vec::new(),
                        span,
                    });
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
                                return Ok(ParsedExpr::Tuple { items, span });
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
                Ok(ParsedExpr::Block { statements, span })
            }

            // match
            TokenKind::Match => {
                self.next();

                let scrutinee = Box::new(self.parse_expression()?);

                let TokenKind::LeftBrace = self.peek().kind else {
                    return Err(parse_error!(Some(token_span(self.peek())), "expected `{{`"));
                };
                self.next();

                let match_arms = self.parse_match_arms()?;

                Ok(ParsedExpr::Match {
                    scrutinee,
                    match_arms,
                    span,
                })
            }

            _ => Err(parse_error!(Some(span), "atomic expression parsing error")),
        }
    }

    fn parse_match_arms(&mut self) -> FogResult<Vec<ParsedMatchArm>> {
        let mut arms = Vec::new();

        while let TokenKind::Newline = self.peek().kind {
            self.next();
        }

        loop {
            if let TokenKind::Eof = self.peek().kind {
                return Err(parse_error!(None, "unclosed match"));
            }

            if let TokenKind::RightBrace = self.peek().kind {
                self.next();
                break;
            }

            let pattern = self.parse_expression()?;

            let TokenKind::FatArrow = self.peek().kind else {
                return Err(parse_error!(Some(token_span(self.peek())), "expected `=>`"));
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
}
