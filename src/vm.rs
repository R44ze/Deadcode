// src/vm.rs - Полноценная виртуальная машина DeadCode
use crate::error::{Error, Result};
use crate::runtime::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Константы и литералы
    LoadConst(Value),
    LoadNull,
    LoadTrue,
    LoadFalse,
    
    // Переменные
    LoadVar(String),
    StoreVar(String),
    
    // Арифметика
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    
    // Сравнение
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    
    // Логика
    And,
    Or,
    Not,
    
    // Управление потоком
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    Call(String, usize), // имя функции, количество аргументов
    Return,
    
    // Работа с данными
    MakeArray(usize),
    MakeStruct(String, usize),
    GetField(String),
    SetField(String),
    GetIndex,
    SetIndex,
    
    // Вывод
    Say,
    
    // Стек
    Pop,
    Dup,
    
    // Halt
    Halt,
}

pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    locals: Vec<HashMap<String, Value>>,
    call_stack: Vec<CallFrame>,
    ip: usize,
    bytecode: Vec<Instruction>,
    functions: HashMap<String, Function>,
}

#[derive(Debug, Clone)]
struct CallFrame {
    return_ip: usize,
    function_name: String,
}

#[derive(Debug, Clone)]
struct Function {
    params: Vec<String>,
    code_start: usize,
    code_end: usize,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
            locals: vec![HashMap::new()],
            call_stack: Vec::new(),
            ip: 0,
            bytecode: Vec::new(),
            functions: HashMap::new(),
        }
    }

    pub fn load_bytecode(&mut self, bytecode: Vec<Instruction>) {
        self.bytecode = bytecode;
    }

    pub fn execute(&mut self, bytecode: &[u8]) -> Result<Value> {
        // Десериализация байткода в инструкции
        let instructions = self.deserialize_bytecode(bytecode)?;
        self.load_bytecode(instructions);
        
        // Выполнение
        self.run()
    }

    fn deserialize_bytecode(&self, _bytecode: &[u8]) -> Result<Vec<Instruction>> {
        // TODO: Реальная десериализация
        // Пока возвращаем пустой вектор, так как у нас пока нет генератора байткода
        Ok(vec![])
    }

    pub fn execute_instructions(&mut self, instructions: Vec<Instruction>) -> Result<Value> {
        self.load_bytecode(instructions);
        self.run()
    }

    fn run(&mut self) -> Result<Value> {
        self.ip = 0;

        while self.ip < self.bytecode.len() {
            let instruction = self.bytecode[self.ip].clone();
            
            match instruction {
                Instruction::LoadConst(value) => {
                    self.stack.push(value);
                }
                Instruction::LoadNull => {
                    self.stack.push(Value::Null);
                }
                Instruction::LoadTrue => {
                    self.stack.push(Value::Bool(true));
                }
                Instruction::LoadFalse => {
                    self.stack.push(Value::Bool(false));
                }
                Instruction::LoadVar(name) => {
                    let value = self.get_variable(&name)?;
                    self.stack.push(value);
                }
                Instruction::StoreVar(name) => {
                    let value = self.pop()?;
                    self.set_variable(name, value);
                }
                Instruction::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(self.add(a, b)?);
                }
                Instruction::Sub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(self.sub(a, b)?);
                }
                Instruction::Mul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(self.mul(a, b)?);
                }
                Instruction::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(self.div(a, b)?);
                }
                Instruction::Mod => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(self.modulo(a, b)?);
                }
                Instruction::Neg => {
                    let a = self.pop()?;
                    self.stack.push(self.negate(a)?);
                }
                Instruction::Eq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(a == b));
                }
                Instruction::Ne => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(a != b));
                }
                Instruction::Lt => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(self.less_than(&a, &b)?));
                }
                Instruction::Le => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(self.less_than(&a, &b)? || a == b));
                }
                Instruction::Gt => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(self.less_than(&b, &a)?));
                }
                Instruction::Ge => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(self.less_than(&b, &a)? || a == b));
                }
                Instruction::And => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(self.to_bool(&a) && self.to_bool(&b)));
                }
                Instruction::Or => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(self.to_bool(&a) || self.to_bool(&b)));
                }
                Instruction::Not => {
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(!self.to_bool(&a)));
                }
                Instruction::Jump(addr) => {
                    self.ip = addr;
                    continue;
                }
                Instruction::JumpIfFalse(addr) => {
                    let cond = self.pop()?;
                    if !self.to_bool(&cond) {
                        self.ip = addr;
                        continue;
                    }
                }
                Instruction::JumpIfTrue(addr) => {
                    let cond = self.pop()?;
                    if self.to_bool(&cond) {
                        self.ip = addr;
                        continue;
                    }
                }
                Instruction::Call(name, arg_count) => {
                    self.call_function(&name, arg_count)?;
                    continue;
                }
                Instruction::Return => {
                    if let Some(frame) = self.call_stack.pop() {
                        self.ip = frame.return_ip;
                        self.locals.pop();
                    } else {
                        // Конец программы
                        break;
                    }
                }
                Instruction::MakeArray(size) => {
                    let mut elements = Vec::new();
                    for _ in 0..size {
                        elements.push(self.pop()?);
                    }
                    elements.reverse();
                    self.stack.push(Value::Array(elements));
                }
                Instruction::MakeStruct(name, field_count) => {
                    let mut fields = HashMap::new();
                    for _ in 0..field_count {
                        let value = self.pop()?;
                        let field_name = self.pop()?;
                        if let Value::String(n) = field_name {
                            fields.insert(n, value);
                        }
                    }
                    self.stack.push(Value::Struct(name, fields));
                }
                Instruction::GetField(field) => {
                    let obj = self.pop()?;
                    if let Value::Struct(_, fields) = obj {
                        let value = fields.get(&field)
                            .ok_or_else(|| Error::Internal(format!("Field '{}' not found", field)))?
                            .clone();
                        self.stack.push(value);
                    } else {
                        return Err(Error::Internal("GetField on non-struct".to_string()));
                    }
                }
                Instruction::SetField(field) => {
                    let value = self.pop()?;
                    let obj = self.pop()?;
                    if let Value::Struct(name, mut fields) = obj {
                        fields.insert(field, value);
                        self.stack.push(Value::Struct(name, fields));
                    } else {
                        return Err(Error::Internal("SetField on non-struct".to_string()));
                    }
                }
                Instruction::GetIndex => {
                    let index = self.pop()?;
                    let arr = self.pop()?;
                    if let (Value::Array(elements), Value::Integer(i)) = (arr, index) {
                        let value = elements.get(i as usize)
                            .ok_or_else(|| Error::Internal("Index out of bounds".to_string()))?
                            .clone();
                        self.stack.push(value);
                    } else {
                        return Err(Error::Internal("GetIndex on non-array".to_string()));
                    }
                }
                Instruction::SetIndex => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let arr = self.pop()?;
                    if let (Value::Array(mut elements), Value::Integer(i)) = (arr, index) {
                        if i >= 0 && (i as usize) < elements.len() {
                            elements[i as usize] = value;
                            self.stack.push(Value::Array(elements));
                        } else {
                            return Err(Error::Internal("Index out of bounds".to_string()));
                        }
                    } else {
                        return Err(Error::Internal("SetIndex on non-array".to_string()));
                    }
                }
                Instruction::Say => {
                    let value = self.pop()?;
                    println!("{}", self.value_to_string(&value));
                }
                Instruction::Pop => {
                    self.pop()?;
                }
                Instruction::Dup => {
                    let value = self.peek()?;
                    self.stack.push(value);
                }
                Instruction::Halt => {
                    break;
                }
            }

            self.ip += 1;
        }

        // Возвращаем последнее значение со стека или Null
        Ok(self.stack.pop().unwrap_or(Value::Null))
    }

    fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or_else(|| Error::StackOverflow)
    }

    fn peek(&self) -> Result<Value> {
        self.stack.last().cloned().ok_or_else(|| Error::StackOverflow)
    }

    fn get_variable(&self, name: &str) -> Result<Value> {
        // Сначала ищем в локальных областях видимости
        for scope in self.locals.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        
        // Затем в глобальных
        self.globals.get(name)
            .cloned()
            .ok_or_else(|| Error::UndefinedVariable(name.to_string(), 0, 0))
    }

    fn set_variable(&mut self, name: String, value: Value) {
        // Устанавливаем в текущей локальной области видимости
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name, value);
        } else {
            self.globals.insert(name, value);
        }
    }

    fn call_function(&mut self, name: &str, arg_count: usize) -> Result<()> {
        // Встроенные функции
        match name {
            "rgb" => {
                if arg_count != 3 {
                    return Err(Error::Internal("rgb() требует 3 аргумента".to_string()));
                }
                let b = self.pop()?;
                let g = self.pop()?;
                let r = self.pop()?;
                
                if let (Value::Integer(r), Value::Integer(g), Value::Integer(b)) = (r, g, b) {
                    self.stack.push(Value::Color(r as u8, g as u8, b as u8));
                    return Ok(());
                }
                return Err(Error::Internal("rgb() требует целочисленные аргументы".to_string()));
            }
            _ => {}
        }

        // Пользовательские функции
        if let Some(func) = self.functions.get(name).cloned() {
            // Собираем аргументы
            let mut args = Vec::new();
            for _ in 0..arg_count {
                args.push(self.pop()?);
            }
            args.reverse();

            // Создаем новую локальную область видимости
            let mut local_scope = HashMap::new();
            for (param, arg) in func.params.iter().zip(args.iter()) {
                local_scope.insert(param.clone(), arg.clone());
            }
            self.locals.push(local_scope);

            // Сохраняем текущий IP
            self.call_stack.push(CallFrame {
                return_ip: self.ip + 1,
                function_name: name.to_string(),
            });

            // Переходим к коду функции
            self.ip = func.code_start;
            
            Ok(())
        } else {
            Err(Error::UndefinedVariable(name.to_string(), 0, 0))
        }
    }

    fn add(&self, a: Value, b: Value) -> Result<Value> {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            _ => Err(Error::Internal("Cannot add these types".to_string())),
        }
    }

    fn sub(&self, a: Value, b: Value) -> Result<Value> {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - b as f64)),
            _ => Err(Error::Internal("Cannot subtract these types".to_string())),
        }
    }

    fn mul(&self, a: Value, b: Value) -> Result<Value> {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * b as f64)),
            _ => Err(Error::Internal("Cannot multiply these types".to_string())),
        }
    }

    fn div(&self, a: Value, b: Value) -> Result<Value> {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    return Err(Error::DivisionByZero(0, 0));
                }
                Ok(Value::Integer(a / b))
            }
            (Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    return Err(Error::DivisionByZero(0, 0));
                }
                Ok(Value::Float(a / b))
            }
            (Value::Integer(a), Value::Float(b)) => {
                if b == 0.0 {
                    return Err(Error::DivisionByZero(0, 0));
                }
                Ok(Value::Float(a as f64 / b))
            }
            (Value::Float(a), Value::Integer(b)) => {
                if b == 0 {
                    return Err(Error::DivisionByZero(0, 0));
                }
                Ok(Value::Float(a / b as f64))
            }
            _ => Err(Error::Internal("Cannot divide these types".to_string())),
        }
    }

    fn modulo(&self, a: Value, b: Value) -> Result<Value> {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    return Err(Error::DivisionByZero(0, 0));
                }
                Ok(Value::Integer(a % b))
            }
            _ => Err(Error::Internal("Modulo only works on integers".to_string())),
        }
    }

    fn negate(&self, a: Value) -> Result<Value> {
        match a {
            Value::Integer(n) => Ok(Value::Integer(-n)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(Error::Internal("Cannot negate this type".to_string())),
        }
    }

    fn less_than(&self, a: &Value, b: &Value) -> Result<bool> {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a < b),
            (Value::Float(a), Value::Float(b)) => Ok(a < b),
            (Value::Integer(a), Value::Float(b)) => Ok((*a as f64) < *b),
            (Value::Float(a), Value::Integer(b)) => Ok(*a < (*b as f64)),
            _ => Err(Error::Internal("Cannot compare these types".to_string())),
        }
    }

    fn to_bool(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Integer(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            _ => true,
        }
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Char(c) => c.to_string(),
            Value::Array(elements) => {
                let strings: Vec<String> = elements.iter()
                    .map(|v| self.value_to_string(v))
                    .collect();
                format!("[{}]", strings.join(", "))
            }
            Value::Struct(name, fields) => {
                let field_strings: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.value_to_string(v)))
                    .collect();
                format!("{} {{ {} }}", name, field_strings.join(", "))
            }
            Value::Color(r, g, b) => format!("rgb({}, {}, {})", r, g, b),
            Value::Function(name) => format!("<function {}>", name),
        }
    }

    pub fn register_function(&mut self, name: String, params: Vec<String>, code_start: usize, code_end: usize) {
        self.functions.insert(name.clone(), Function {
            params,
            code_start,
            code_end,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        let mut vm = VM::new();
        let instructions = vec![
            Instruction::LoadConst(Value::Integer(5)),
            Instruction::LoadConst(Value::Integer(3)),
            Instruction::Add,
            Instruction::Halt,
        ];
        
        let result = vm.execute_instructions(instructions).unwrap();
        assert_eq!(result, Value::Integer(8));
    }

    #[test]
    fn test_variables() {
        let mut vm = VM::new();
        let instructions = vec![
            Instruction::LoadConst(Value::Integer(42)),
            Instruction::StoreVar("x".to_string()),
            Instruction::LoadVar("x".to_string()),
            Instruction::Halt,
        ];
        
        let result = vm.execute_instructions(instructions).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_say() {
        let mut vm = VM::new();
        let instructions = vec![
            Instruction::LoadConst(Value::String("Hello, DeadCode!".to_string())),
            Instruction::Say,
            Instruction::Halt,
        ];
        
        vm.execute_instructions(instructions).unwrap();
    }
}
