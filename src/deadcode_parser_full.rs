// src/parser.rs - Continuation
    fn parse_postfix(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;

        loop {
            match &self.current().kind {
                TokenKind::LParen => {
                    // Function call
                    self.advance();
                    let mut args = Vec::new();
                    while !self.check(&TokenKind::RParen) {
                        args.push(self.parse_expression()?);
                        if !self.check(&TokenKind::RParen) {
                            self.expect(TokenKind::Comma, ",")?;
                        }
                    }
                    let end = self.current().clone();
                    self.expect(TokenKind::RParen, ")")?;
                    let span = Span::new(
                        expr.span().start,
                        end.span.end,
                        expr.span().line,
                        expr.span().column,
                    );
                    expr = Expression::Call {
                        func: Box::new(expr),
                        args,
                        span,
                    };
                }
                TokenKind::Dot => {
                    // Field access
                    self.advance();
                    let field = if let TokenKind::Ident(name) = &self.current().kind {
                        let name = name.clone();
                        self.advance();
                        name
                    } else {
                        return Err(Error::ExpectedToken(
                            "field name".to_string(),
                            format!("{:?}", self.current().kind),
                            self.current().span.line,
                            self.current().span.column,
                        ));
                    };
                    let span = Span::new(
                        expr.span().start,
                        self.tokens[self.pos - 1].span.end,
                        expr.span().line,
                        expr.span().column,
                    );
                    expr = Expression::FieldAccess {
                        object: Box::new(expr),
                        field,
                        span,
                    };
                }
                TokenKind::LBracket => {
                    // Index access
                    self.advance();
                    let index = self.parse_expression()?;
                    let end = self.current().clone();
                    self.expect(TokenKind::RBracket, "]")?;
                    let span = Span::new(
                        expr.span().start,
                        end.span.end,
                        expr.span().line,
                        expr.span().column,
                    );
                    expr = Expression::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                        span,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        let token = self.current().clone();
        
        match &token.kind {
            TokenKind::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Literal(Literal::Integer(n), token.span.into()))
            }
            TokenKind::Float(f) => {
                let f = *f;
                self.advance();
                Ok(Expression::Literal(Literal::Float(f), token.span.into()))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(s), token.span.into()))
            }
            TokenKind::Char(c) => {
                let c = *c;
                self.advance();
                Ok(Expression::Literal(Literal::Char(c), token.span.into()))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(true), token.span.into()))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(false), token.span.into()))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expression::Literal(Literal::Null, token.span.into()))
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                
                // Check for struct initialization
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut fields = Vec::new();
                    while !self.check(&TokenKind::RBrace) {
                        let field_name = if let TokenKind::Ident(n) = &self.current().kind {
                            let n = n.clone();
                            self.advance();
                            n
                        } else {
                            return Err(Error::ExpectedToken(
                                "field name".to_string(),
                                format!("{:?}", self.current().kind),
                                self.current().span.line,
                                self.current().span.column,
                            ));
                        };
                        self.expect(TokenKind::Colon, ":")?;
                        let value = self.parse_expression()?;
                        fields.push((field_name, value));
                        if !self.check(&TokenKind::RBrace) {
                            self.expect(TokenKind::Comma, ",")?;
                        }
                    }
                    let end = self.current().clone();
                    self.expect(TokenKind::RBrace, "}")?;
                    Ok(Expression::StructInit {
                        name,
                        fields,
                        span: Span::new(token.span.start, end.span.end, token.span.line, token.span.column),
                    })
                } else {
                    Ok(Expression::Variable(name, token.span.into()))
                }
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen, ")")?;
                Ok(expr)
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                while !self.check(&TokenKind::RBracket) {
                    elements.push(self.parse_expression()?);
                    if !self.check(&TokenKind::RBracket) {
                        self.expect(TokenKind::Comma, ",")?;
                    }
                }
                let end = self.current().clone();
                self.expect(TokenKind::RBracket, "]")?;
                Ok(Expression::Array(elements, Span::new(
                    token.span.start,
                    end.span.end,
                    token.span.line,
                    token.span.column,
                )))
            }
            TokenKind::Say => {
                self.advance();
                self.expect(TokenKind::LBrace, "{")?;
                let message = Box::new(self.parse_expression()?);
                let end = self.current().clone();
                self.expect(TokenKind::RBrace, "}")?;
                Ok(Expression::Say {
                    message,
                    span: Span::new(token.span.start, end.span.end, token.span.line, token.span.column),
                })
            }
            _ => Err(Error::UnexpectedToken(
                format!("expression, got {:?}", token.kind),
                token.span.line,
                token.span.column,
            )),
        }
    }
}

impl From<TokenSpan> for Span {
    fn from(ts: TokenSpan) -> Self {
        Span::new(ts.start, ts.end, ts.line, ts.column)
    }
}

pub fn parse(tokens: &[Token]) -> Result<Program> {
    let mut parser = Parser::new(tokens.to_vec());
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_parse_simple() {
        let tokens = tokenize(r#"let x = 42"#).unwrap();
        let ast = parse(&tokens);
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_function() {
        let tokens = tokenize(r#"
            func add(a: i32, b: i32) -> i32 {
                return a + b
            }
        "#).unwrap();
        let ast = parse(&tokens);
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_struct() {
        let tokens = tokenize(r#"
            struct Point {
                x: f32,
                y: f32
            }
        "#).unwrap();
        let ast = parse(&tokens);
        assert!(ast.is_ok());
    }
}