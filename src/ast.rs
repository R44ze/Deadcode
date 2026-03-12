// src/ast.rs - Abstract Syntax Tree для DeadCode
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(EnumDef),
    Sprite(Sprite),
    Window(Window),
    Import(Import),
}

// ==================== ФУНКЦИИ ====================

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

// ==================== СТРУКТУРЫ ====================

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

// ==================== ПЕРЕЧИСЛЕНИЯ ====================

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<String>,
    pub span: Span,
}

// ==================== ГРАФИКА ====================

#[derive(Debug, Clone, PartialEq)]
pub struct Sprite {
    pub name: String,
    pub properties: Vec<Property>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    pub name: String,
    pub properties: Vec<Property>,
    pub render_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: String,
    pub value: Expression,
    pub span: Span,
}

// ==================== ИМПОРТЫ ====================

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: String,
    pub alias: Option<String>,
    pub span: Span,
}

// ==================== ВЫРАЖЕНИЯ ====================

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        name: String,
        ty: Option<Type>,
        value: Expression,
        is_const: bool,
        span: Span,
    },
    Expression(Expression),
    Return(Option<Expression>, Span),
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
        span: Span,
    },
    While {
        condition: Expression,
        body: Block,
        span: Span,
    },
    For {
        var: String,
        iter: Expression,
        body: Block,
        span: Span,
    },
    Break(Span),
    Continue(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal, Span),
    Variable(String, Span),
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
        span: Span,
    },
    Call {
        func: Box<Expression>,
        args: Vec<Expression>,
        span: Span,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
        span: Span,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
    StructInit {
        name: String,
        fields: Vec<(String, Expression)>,
        span: Span,
    },
    Array(Vec<Expression>, Span),
    Say {
        message: Box<Expression>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Char(char),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

// ==================== ТИПЫ ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Char,
    String,
    Void,
    Array(Box<Type>, Option<usize>),
    Custom(String),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::Char => write!(f, "char"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),
            Type::Array(inner, Some(size)) => write!(f, "[{}; {}]", inner, size),
            Type::Array(inner, None) => write!(f, "[{}]", inner),
            Type::Custom(name) => write!(f, "{}", name),
        }
    }
}

// ==================== SPAN (для отслеживания позиции в коде) ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    pub fn dummy() -> Self {
        Self {
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        }
    }
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Literal(_, span) => *span,
            Expression::Variable(_, span) => *span,
            Expression::Binary { span, .. } => *span,
            Expression::Unary { span, .. } => *span,
            Expression::Call { span, .. } => *span,
            Expression::FieldAccess { span, .. } => *span,
            Expression::Index { span, .. } => *span,
            Expression::StructInit { span, .. } => *span,
            Expression::Array(_, span) => *span,
            Expression::Say { span, .. } => *span,
        }
    }
}