use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    UnexpectedCharacter(char, usize, usize),
    UnterminatedString(usize, usize),
    InvalidNumber(String, usize, usize),
    UnexpectedToken(String, usize, usize),
    ExpectedToken(String, String, usize, usize),
    UnexpectedEof,
    UndefinedVariable(String, usize, usize),
    TypeMismatch { expected: String, found: String, line: usize, column: usize },
    DuplicateDefinition(String, usize, usize),
    InvalidOperation(String, usize, usize),
    StackOverflow,
    OutOfMemory,
    DivisionByZero(usize, usize),
    IoError(std::io::Error),
    UnsupportedOperation(String),
    Internal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UnexpectedCharacter(ch, line, col) => 
                write!(f, "Unexpected character '{}' at {}:{}", ch, line, col),
            Error::UnterminatedString(line, col) => 
                write!(f, "Unterminated string at {}:{}", line, col),
            Error::InvalidNumber(num, line, col) => 
                write!(f, "Invalid number '{}' at {}:{}", num, line, col),
            Error::UnexpectedToken(token, line, col) => 
                write!(f, "Unexpected token '{}' at {}:{}", token, line, col),
            Error::ExpectedToken(expected, found, line, col) => 
                write!(f, "Expected '{}', found '{}' at {}:{}", expected, found, line, col),
            Error::UnexpectedEof => write!(f, "Unexpected end of file"),
            Error::UndefinedVariable(name, line, col) => 
                write!(f, "Undefined variable '{}' at {}:{}", name, line, col),
            Error::TypeMismatch { expected, found, line, column } => 
                write!(f, "Type mismatch: expected {}, found {} at {}:{}", expected, found, line, column),
            Error::DuplicateDefinition(name, line, col) => 
                write!(f, "Duplicate definition of '{}' at {}:{}", name, line, col),
            Error::InvalidOperation(op, line, col) => 
                write!(f, "Invalid operation '{}' at {}:{}", op, line, col),
            Error::StackOverflow => write!(f, "Stack overflow"),
            Error::OutOfMemory => write!(f, "Out of memory"),
            Error::DivisionByZero(line, col) => 
                write!(f, "Division by zero at {}:{}", line, col),
            Error::IoError(err) => write!(f, "I/O error: {}", err),
            Error::UnsupportedOperation(op) => 
                write!(f, "Unsupported operation: {}", op),
            Error::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}
