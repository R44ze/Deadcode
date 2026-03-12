// src/runtime.rs - Типы значений во время выполнения
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Array(Vec<Value>),
    Struct(String, HashMap<String, Value>),
    Color(u8, u8, u8),
    Function(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Char(c) => write!(f, "{}", c),
            Value::Array(elements) => {
                let strings: Vec<String> = elements.iter()
                    .map(|v| v.to_string())
                    .collect();
                write!(f, "[{}]", strings.join(", "))
            }
            Value::Struct(name, fields) => {
                let field_strings: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{} {{ {} }}", name, field_strings.join(", "))
            }
            Value::Color(r, g, b) => write!(f, "rgb({}, {}, {})", r, g, b),
            Value::Function(name) => write!(f, "<function {}>", name),
        }
    }
}