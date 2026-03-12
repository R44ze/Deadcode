use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: TokenSpan,
    pub lexeme: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Ident(String),
    Say, Let, Const, Func, Return, If, Else, While, For, In,
    Break, Continue, Struct, Enum, Sprite, Window, Render,
    Draw, Clear, Import, As, This, Null, True, False,
    I8, I16, I32, I64, U8, U16, U32, U64, F32, F64,
    Bool_, Char_, String_, Void,
    Plus, Minus, Star, Slash, Percent, Amp, Pipe, Caret, Bang,
    Eq, EqEq, Ne, Lt, Le, Gt, Ge, AmpAmp, PipePipe,
    PlusEq, MinusEq, StarEq, SlashEq, Arrow,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Dot, Colon, DoubleColon, Semicolon,
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    start_pos: usize,
    start_line: usize,
    start_column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            start_pos: 0,
            start_line: 1,
            start_column: 1,
        }
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<TokenKind> {
        let mut num_str = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            num_str.parse::<f64>()
                .map(TokenKind::Float)
                .map_err(|_| Error::InvalidNumber(num_str, self.start_line, self.start_column))
        } else {
            num_str.parse::<i64>()
                .map(TokenKind::Integer)
                .map_err(|_| Error::InvalidNumber(num_str, self.start_line, self.start_column))
        }
    }

    fn read_string(&mut self) -> Result<TokenKind> {
        self.advance();
        let mut string = String::new();

        while let Some(ch) = self.current() {
            if ch == '"' {
                self.advance();
                return Ok(TokenKind::String(string));
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current() {
                    match escaped {
                        'n' => string.push('\n'),
                        't' => string.push('\t'),
                        _ => string.push(escaped),
                    }
                    self.advance();
                }
            } else {
                string.push(ch);
                self.advance();
            }
        }

        Err(Error::UnterminatedString(self.start_line, self.start_column))
    }

    fn read_ident(&mut self) -> String {
        let mut ident = String::new();
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    fn keyword_or_ident(&self, ident: &str) -> TokenKind {
        match ident {
            "say" => TokenKind::Say,
            "let" => TokenKind::Let,
            "func" => TokenKind::Func,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "i32" => TokenKind::I32,
            "string" => TokenKind::String_,
            _ => TokenKind::Ident(ident.to_string()),
        }
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        let lexeme = self.input[self.start_pos..self.pos].iter().collect();
        Token {
            kind,
            span: TokenSpan {
                start: self.start_pos,
                end: self.pos,
                line: self.start_line,
                column: self.start_column,
            },
            lexeme,
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        self.start_pos = self.pos;
        self.start_line = self.line;
        self.start_column = self.column;

        let kind = match self.current() {
            None => TokenKind::Eof,
            Some(ch) => match ch {
                '0'..='9' => self.read_number()?,
                '"' => self.read_string()?,
                'a'..='z' | 'A'..='Z' | '_' => {
                    let ident = self.read_ident();
                    self.keyword_or_ident(&ident)
                }
                '+' => { self.advance(); TokenKind::Plus }
                '-' => { self.advance(); TokenKind::Minus }
                '*' => { self.advance(); TokenKind::Star }
                '/' => { self.advance(); TokenKind::Slash }
                '=' => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        TokenKind::EqEq
                    } else {
                        TokenKind::Eq
                    }
                }
                '<' => { self.advance(); TokenKind::Lt }
                '>' => { self.advance(); TokenKind::Gt }
                '(' => { self.advance(); TokenKind::LParen }
                ')' => { self.advance(); TokenKind::RParen }
                '{' => { self.advance(); TokenKind::LBrace }
                '}' => { self.advance(); TokenKind::RBrace }
                ',' => { self.advance(); TokenKind::Comma }
                ';' => { self.advance(); TokenKind::Semicolon }
                ':' => { self.advance(); TokenKind::Colon }
                _ => {
                    return Err(Error::UnexpectedCharacter(ch, self.line, self.column));
                }
            }
        };

        Ok(self.make_token(kind))
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token()?;
        let is_eof = matches!(token.kind, TokenKind::Eof);
        tokens.push(token);
        if is_eof {
            break;
        }
    }

    Ok(tokens)
}

