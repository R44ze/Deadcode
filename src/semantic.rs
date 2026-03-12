use crate::ast::*;
use crate::error::{Error, Result};
use std::collections::HashMap;

pub struct SemanticAnalyzer {
    symbols: HashMap<String, Type>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, _program: &Program) -> Result<()> {
        // Пока пропускаем проверки
        Ok(())
    }
}

pub fn analyze(program: Program) -> Result<Program> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program)?;
    Ok(program)
}
