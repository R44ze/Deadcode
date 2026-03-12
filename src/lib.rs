pub mod lexer;
pub mod parser;
pub mod ast;
pub mod semantic;
pub mod bytecode;
pub mod vm;
pub mod runtime;
pub mod error;

use std::path::Path;
pub use error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Interpret,
}

pub struct DeadCode {
    mode: ExecutionMode,
}

impl DeadCode {
    pub fn new(mode: ExecutionMode) -> Self {
        Self { mode }
    }

    pub fn run_string(&self, source: &str) -> Result<runtime::Value> {
        let tokens = lexer::tokenize(source)?;
        let ast = parser::parse(&tokens)?;
        let validated_ast = semantic::analyze(ast)?;
        let bytecode = bytecode::generate(&validated_ast)?;
        
        let mut vm = vm::VM::new();
        vm.execute_instructions(bytecode)
    }

    pub fn run_file(&self, path: &Path) -> Result<runtime::Value> {
        let source = std::fs::read_to_string(path)?;
        self.run_string(&source)
    }
}

impl Default for DeadCode {
    fn default() -> Self {
        Self::new(ExecutionMode::Interpret)
    }
}
