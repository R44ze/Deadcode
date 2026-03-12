use crate::ast::*;
use crate::error::{Error, Result};
use crate::lexer::{Token, TokenKind, TokenSpan};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
    }

    fn expect(&mut self, kind: TokenKind, expected: &str) -> Result<Token> {
        if self.check(&kind) {
            let token = self.current().clone();
            self.advance();
            Ok(token)
        } else {
            Err(Error::ExpectedToken(
                expected.to_string(),
                format!("{:?}", self.current().kind),
                self.current().span.line,
                self.current().span.column,
            ))
        }
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut items = Vec::new();

        while !self.check(&TokenKind::Eof) {
            // Пока просто парсим top-level выражения как "функцию main"
            let stmt = self.parse_statement()?;
            
            // Оборачиваем в функцию main для выполнения
            let func = Function {
                name: "main".to_string(),
                params: vec![],
                return_type: None,
                body: Block {
                    statements: vec![stmt],
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            };
            items.push(Item::Function(func));
        }

        Ok(Program { items })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        let expr = self.parse_expression()?;
        Ok(Statement::Expression(expr))
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        let token = self.current().clone();
        
        match &token.kind {
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
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(s), token.span.into()))
            }
            TokenKind::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Literal(Literal::Integer(n), token.span.into()))
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
