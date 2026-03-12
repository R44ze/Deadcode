// src/semantic.rs - Семантический анализ
use crate::ast::*;
use crate::error::{Error, Result};
use std::collections::HashMap;

pub struct SemanticAnalyzer {
    symbols: HashMap<String, Type>,
    scopes: Vec<HashMap<String, Type>>,
    current_function_return_type: Option<Type>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            scopes: vec![HashMap::new()],
            current_function_return_type: None,
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<()> {
        // Первый проход - собираем определения функций и структур
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    let func_type = func.return_type.clone().unwrap_or(Type::Void);
                    self.define(func.name.clone(), func_type)?;
                }
                Item::Struct(s) => {
                    self.define(s.name.clone(), Type::Custom(s.name.clone()))?;
                }
                Item::Enum(e) => {
                    self.define(e.name.clone(), Type::Custom(e.name.clone()))?;
                }
                _ => {}
            }
        }

        // Второй проход - проверяем тела функций
        for item in &program.items {
            if let Item::Function(func) = item {
                self.check_function(func)?;
            }
        }

        Ok(())
    }

    fn check_function(&mut self, func: &Function) -> Result<()> {
        self.enter_scope();
        
        self.current_function_return_type = func.return_type.clone();

        // Добавляем параметры в область видимости
        for param in &func.params {
            self.define(param.name.clone(), param.ty.clone())?;
        }

        // Проверяем тело функции
        self.check_block(&func.body)?;

        self.current_function_return_type = None;
        self.leave_scope();

        Ok(())
    }

    fn check_block(&mut self, block: &Block) -> Result<()> {
        for stmt in &block.statements {
            self.check_statement(stmt)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Let { name, ty, value, .. } => {
                let value_type = self.infer_expression(value)?;
                
                if let Some(declared_type) = ty {
                    if !self.types_compatible(declared_type, &value_type) {
                        return Err(Error::TypeMismatch {
                            expected: declared_type.to_string(),
                            found: value_type.to_string(),
                            line: 0,
                            column: 0,
                        });
                    }
                    self.define(name.clone(), declared_type.clone())?;
                } else {
                    self.define(name.clone(), value_type)?;
                }
            }
            Statement::Expression(expr) => {
                self.infer_expression(expr)?;
            }
            Statement::Return(value, _) => {
                if let Some(expected) = &self.current_function_return_type {
                    if let Some(expr) = value {
                        let actual = self.infer_expression(expr)?;
                        if !self.types_compatible(expected, &actual) {
                            return Err(Error::TypeMismatch {
                                expected: expected.to_string(),
                                found: actual.to_string(),
                                line: 0,
                                column: 0,
                            });
                        }
                    } else if *expected != Type::Void {
                        return Err(Error::TypeMismatch {
                            expected: expected.to_string(),
                            found: "void".to_string(),
                            line: 0,
                            column: 0,
                        });
                    }
                }
            }
            Statement::If { condition, then_block, else_block, .. } => {
                let cond_type = self.infer_expression(condition)?;
                if cond_type != Type::Bool {
                    return Err(Error::TypeMismatch {
                        expected: "bool".to_string(),
                        found: cond_type.to_string(),
                        line: 0,
                        column: 0,
                    });
                }
                self.enter_scope();
                self.check_block(then_block)?;
                self.leave_scope();
                
                if let Some(else_b) = else_block {
                    self.enter_scope();
                    self.check_block(else_b)?;
                    self.leave_scope();
                }
            }
            Statement::While { condition, body, .. } => {
                let cond_type = self.infer_expression(condition)?;
                if cond_type != Type::Bool {
                    return Err(Error::TypeMismatch {
                        expected: "bool".to_string(),
                        found: cond_type.to_string(),
                        line: 0,
                        column: 0,
                    });
                }
                self.enter_scope();
                self.check_block(body)?;
                self.leave_scope();
            }
            Statement::For { var, iter, body, .. } => {
                let iter_type = self.infer_expression(iter)?;
                let element_type = match iter_type {
                    Type::Array(inner, _) => *inner,
                    _ => return Err(Error::TypeMismatch {
                        expected: "array".to_string(),
                        found: iter_type.to_string(),
                        line: 0,
                        column: 0,
                    }),
                };
                
                self.enter_scope();
                self.define(var.clone(), element_type)?;
                self.check_block(body)?;
                self.leave_scope();
            }
            Statement::Break(_) | Statement::Continue(_) => {}
        }
        Ok(())
    }

    fn infer_expression(&self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Literal(lit, _) => Ok(self.literal_type(lit)),
            Expression::Variable(name, span) => {
                self.lookup(name)
                    .ok_or_else(|| Error::UndefinedVariable(name.clone(), span.line, span.column))
            }
            Expression::Binary { op, left, right, .. } => {
                let left_type = self.infer_expression(left)?;
                let right_type = self.infer_expression(right)?;
                self.binary_op_type(*op, &left_type, &right_type)
            }
            Expression::Unary { op, expr, .. } => {
                let expr_type = self.infer_expression(expr)?;
                self.unary_op_type(*op, &expr_type)
            }
            Expression::Call { func, args, .. } => {
                if let Expression::Variable(name, _) = &**func {
                    // Встроенные функции
                    if name == "rgb" {
                        return Ok(Type::Custom("Color".to_string()));
                    }
                    
                    // Пользовательские функции
                    self.lookup(name)
                        .ok_or_else(|| Error::UndefinedVariable(name.clone(), 0, 0))
                } else {
                    Err(Error::Internal("Complex function calls not yet supported".to_string()))
                }
            }
            Expression::FieldAccess { object, field, .. } => {
                let obj_type = self.infer_expression(object)?;
                match obj_type {
                    Type::Custom(name) => {
                        // TODO: Lookup field type in struct definition
                        Ok(Type::I32) // Временная заглушка
                    }
                    _ => Err(Error::Internal("Field access on non-struct".to_string())),
                }
            }
            Expression::Index { object, .. } => {
                let obj_type = self.infer_expression(object)?;
                match obj_type {
                    Type::Array(inner, _) => Ok(*inner),
                    Type::String => Ok(Type::Char),
                    _ => Err(Error::Internal("Index on non-indexable type".to_string())),
                }
            }
            Expression::StructInit { name, .. } => {
                Ok(Type::Custom(name.clone()))
            }
            Expression::Array(elements, _) => {
                if elements.is_empty() {
                    Ok(Type::Array(Box::new(Type::I32), Some(0)))
                } else {
                    let first_type = self.infer_expression(&elements[0])?;
                    Ok(Type::Array(Box::new(first_type), Some(elements.len())))
                }
            }
            Expression::Say { .. } => Ok(Type::Void),
        }
    }

    fn literal_type(&self, lit: &Literal) -> Type {
        match lit {
            Literal::Integer(_) => Type::I32,
            Literal::Float(_) => Type::F64,
            Literal::String(_) => Type::String,
            Literal::Bool(_) => Type::Bool,
            Literal::Char(_) => Type::Char,
            Literal::Null => Type::Custom("Null".to_string()),
        }
    }

    fn binary_op_type(&self, op: BinaryOp, left: &Type, right: &Type) -> Result<Type> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if self.is_numeric(left) && self.is_numeric(right) {
                    if left == &Type::F64 || right == &Type::F64 {
                        Ok(Type::F64)
                    } else if left == &Type::F32 || right == &Type::F32 {
                        Ok(Type::F32)
                    } else {
                        Ok(Type::I32)
                    }
                } else if op == BinaryOp::Add && left == &Type::String && right == &Type::String {
                    Ok(Type::String)
                } else {
                    Err(Error::Internal(format!("Cannot apply {:?} to {:?} and {:?}", op, left, right)))
                }
            }
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                Ok(Type::Bool)
            }
            BinaryOp::And | BinaryOp::Or => {
                if left == &Type::Bool && right == &Type::Bool {
                    Ok(Type::Bool)
                } else {
                    Err(Error::Internal(format!("Logical operators require bool operands")))
                }
            }
            BinaryOp::Assign | BinaryOp::AddAssign | BinaryOp::SubAssign |
            BinaryOp::MulAssign | BinaryOp::DivAssign => Ok(left.clone()),
        }
    }

    fn unary_op_type(&self, op: UnaryOp, expr_type: &Type) -> Result<Type> {
        match op {
            UnaryOp::Neg => {
                if self.is_numeric(expr_type) {
                    Ok(expr_type.clone())
                } else {
                    Err(Error::Internal("Negation requires numeric type".to_string()))
                }
            }
            UnaryOp::Not => {
                if expr_type == &Type::Bool {
                    Ok(Type::Bool)
                } else {
                    Err(Error::Internal("Logical NOT requires bool".to_string()))
                }
            }
        }
    }

    fn is_numeric(&self, ty: &Type) -> bool {
        matches!(ty, 
            Type::I8 | Type::I16 | Type::I32 | Type::I64 |
            Type::U8 | Type::U16 | Type::U32 | Type::U64 |
            Type::F32 | Type::F64
        )
    }

    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }

        // Автоприведение типов
        match (expected, actual) {
            (Type::F64, Type::I32) | (Type::F32, Type::I32) => true,
            (Type::I64, Type::I32) => true,
            _ => false,
        }
    }

    fn define(&mut self, name: String, ty: Type) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        self.symbols.get(name).cloned()
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
    }
}

pub fn analyze(program: Program) -> Result<Program> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program)?;
    Ok(program)
}

// src/bytecode.rs - Генератор байткода
use crate::ast::*;
use crate::vm::Instruction;
use crate::runtime::Value;
use crate::error::Result;

pub struct BytecodeGenerator {
    instructions: Vec<Instruction>,
    label_counter: usize,
}

impl BytecodeGenerator {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            label_counter: 0,
        }
    }

    fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    fn current_position(&self) -> usize {
        self.instructions.len()
    }

    fn next_label(&mut self) -> usize {
        let label = self.label_counter;
        self.label_counter += 1;
        label
    }

    pub fn generate(&mut self, program: &Program) -> Result<Vec<Instruction>> {
        for item in &program.items {
            self.generate_item(item)?;
        }
        self.emit(Instruction::Halt);
        Ok(self.instructions.clone())
    }

    fn generate_item(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::Function(func) => {
                // Регистрация функции будет происходить в VM
                let start = self.current_position();
                self.generate_block(&func.body)?;
                self.emit(Instruction::LoadNull);
                self.emit(Instruction::Return);
                let end = self.current_position();
                
                // TODO: Передать информацию о функции в VM
            }
            Item::Sprite(_) | Item::Window(_) => {
                // Графические элементы обрабатываются отдельно
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_block(&mut self, block: &Block) -> Result<()> {
        for stmt in &block.statements {
            self.generate_statement(stmt)?;
        }
        Ok(())
    }

    fn generate_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Let { name, value, .. } => {
                self.generate_expression(value)?;
                self.emit(Instruction::StoreVar(name.clone()));
            }
            Statement::Expression(expr) => {
                self.generate_expression(expr)?;
                if !matches!(expr, Expression::Say { .. }) {
                    self.emit(Instruction::Pop);
                }
            }
            Statement::Return(value, _) => {
                if let Some(expr) = value {
                    self.generate_expression(expr)?;
                } else {
                    self.emit(Instruction::LoadNull);
                }
                self.emit(Instruction::Return);
            }
            Statement::If { condition, then_block, else_block, .. } => {
                self.generate_expression(condition)?;
                
                let else_label = self.next_label();
                let end_label = self.next_label();
                
                self.emit(Instruction::JumpIfFalse(0)); // Placeholder
                let else_jump_pos = self.current_position() - 1;
                
                self.generate_block(then_block)?;
                self.emit(Instruction::Jump(0)); // Placeholder
                let end_jump_pos = self.current_position() - 1;
                
                // Patch else jump
                let else_pos = self.current_position();
                if let Instruction::JumpIfFalse(ref mut addr) = self.instructions[else_jump_pos] {
                    *addr = else_pos;
                }
                
                if let Some(else_b) = else_block {
                    self.generate_block(else_b)?;
                }
                
                // Patch end jump
                let end_pos = self.current_position();
                if let Instruction::Jump(ref mut addr) = self.instructions[end_jump_pos] {
                    *addr = end_pos;
                }
            }
            Statement::While { condition, body, .. } => {
                let start_pos = self.current_position();
                
                self.generate_expression(condition)?;
                self.emit(Instruction::JumpIfFalse(0)); // Placeholder
                let exit_jump_pos = self.current_position() - 1;
                
                self.generate_block(body)?;
                self.emit(Instruction::Jump(start_pos));
                
                let exit_pos = self.current_position();
                if let Instruction::JumpIfFalse(ref mut addr) = self.instructions[exit_jump_pos] {
                    *addr = exit_pos;
                }
            }
            Statement::For { var, iter, body, .. } => {
                // TODO: Реализовать for loop
                // Пока что заглушка
            }
            Statement::Break(_) | Statement::Continue(_) => {
                // TODO: Реализовать break/continue
            }
        }
        Ok(())
    }

    fn generate_expression(&mut self, expr: &Expression) -> Result<()> {
        match expr {
            Expression::Literal(lit, _) => {
                match lit {
                    Literal::Integer(n) => self.emit(Instruction::LoadConst(Value::Integer(*n))),
                    Literal::Float(f) => self.emit(Instruction::LoadConst(Value::Float(*f))),
                    Literal::String(s) => self.emit(Instruction::LoadConst(Value::String(s.clone()))),
                    Literal::Bool(b) => {
                        if *b {
                            self.emit(Instruction::LoadTrue);
                        } else {
                            self.emit(Instruction::LoadFalse);
                        }
                    }
                    Literal::Char(c) => self.emit(Instruction::LoadConst(Value::Char(*c))),
                    Literal::Null => self.emit(Instruction::LoadNull),
                }
            }
            Expression::Variable(name, _) => {
                self.emit(Instruction::LoadVar(name.clone()));
            }
            Expression::Binary { op, left, right, .. } => {
                self.generate_expression(left)?;
                self.generate_expression(right)?;
                
                match op {
                    BinaryOp::Add => self.emit(Instruction::Add),
                    BinaryOp::Sub => self.emit(Instruction::Sub),
                    BinaryOp::Mul => self.emit(Instruction::Mul),
                    BinaryOp::Div => self.emit(Instruction::Div),
                    BinaryOp::Mod => self.emit(Instruction::Mod),
                    BinaryOp::Eq => self.emit(Instruction::Eq),
                    BinaryOp::Ne => self.emit(Instruction::Ne),
                    BinaryOp::Lt => self.emit(Instruction::Lt),
                    BinaryOp::Le => self.emit(Instruction::Le),
                    BinaryOp::Gt => self.emit(Instruction::Gt),
                    BinaryOp::Ge => self.emit(Instruction::Ge),
                    BinaryOp::And => self.emit(Instruction::And),
                    BinaryOp::Or => self.emit(Instruction::Or),
                    BinaryOp::Assign => {
                        if let Expression::Variable(name, _) = &**left {
                            self.emit(Instruction::StoreVar(name.clone()));
                        }
                    }
                    _ => {}
                }
            }
            Expression::Unary { op, expr, .. } => {
                self.generate_expression(expr)?;
                match op {
                    UnaryOp::Neg => self.emit(Instruction::Neg),
                    UnaryOp::Not => self.emit(Instruction::Not),
                }
            }
            Expression::Call { func, args, .. } => {
                for arg in args {
                    self.generate_expression(arg)?;
                }
                if let Expression::Variable(name, _) = &**func {
                    self.emit(Instruction::Call(name.clone(), args.len()));
                }
            }
            Expression::Array(elements, _) => {
                for elem in elements {
                    self.generate_expression(elem)?;
                }
                self.emit(Instruction::MakeArray(elements.len()));
            }
            Expression::StructInit { name, fields, .. } => {
                for (field_name, value) in fields {
                    self.emit(Instruction::LoadConst(Value::String(field_name.clone())));
                    self.generate_expression(value)?;
                }
                self.emit(Instruction::MakeStruct(name.clone(), fields.len()));
            }
            Expression::Say { message, .. } => {
                self.generate_expression(message)?;
                self.emit(Instruction::Say);
            }
            Expression::FieldAccess { object, field, .. } => {
                self.generate_expression(object)?;
                self.emit(Instruction::GetField(field.clone()));
            }
            Expression::Index { object, index, .. } => {
                self.generate_expression(object)?;
                self.generate_expression(index)?;
                self.emit(Instruction::GetIndex);
            }
        }
        Ok(())
    }
}

pub fn generate(program: &Program) -> Result<Vec<u8>> {
    let mut generator = BytecodeGenerator::new();
    let instructions = generator.generate(program)?;
    
    // Сериализация инструкций в байты
    // TODO: Реальная сериализация
    Ok(vec![])
}