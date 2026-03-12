use crate::ast::*;
use crate::vm::Instruction;
use crate::runtime::Value;
use crate::error::Result;

pub struct BytecodeGenerator {
    instructions: Vec<Instruction>,
}

impl BytecodeGenerator {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }

    fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
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
                // Для функции main просто генерируем её тело
                if func.name == "main" {
                    for stmt in &func.body.statements {
                        self.generate_statement(stmt)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Expression(expr) => {
                self.generate_expression(expr)?;
                // Pop результат выражения если это не say
                if !matches!(expr, Expression::Say { .. }) {
                    self.emit(Instruction::Pop);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_expression(&mut self, expr: &Expression) -> Result<()> {
        match expr {
            Expression::Literal(lit, _) => {
                match lit {
                    Literal::Integer(n) => self.emit(Instruction::LoadConst(Value::Integer(*n))),
                    Literal::String(s) => self.emit(Instruction::LoadConst(Value::String(s.clone()))),
                    Literal::Bool(b) => self.emit(Instruction::LoadConst(Value::Bool(*b))),
                    Literal::Float(f) => self.emit(Instruction::LoadConst(Value::Float(*f))),
                    Literal::Char(c) => self.emit(Instruction::LoadConst(Value::Char(*c))),
                    Literal::Null => self.emit(Instruction::LoadConst(Value::Null)),
                }
            }
            Expression::Say { message, .. } => {
                self.generate_expression(message)?;
                self.emit(Instruction::Say);
            }
            _ => {}
        }
        Ok(())
    }
}

pub fn generate(program: &Program) -> Result<Vec<Instruction>> {
    let mut generator = BytecodeGenerator::new();
    generator.generate(program)
}
