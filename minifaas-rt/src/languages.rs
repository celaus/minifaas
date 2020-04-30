use crate::triggers::Trigger;
use std::boxed::Box;
pub mod javascript;

type Result<T> = std::result::Result<T, LanguageRuntimeError>;

pub trait Compiler {
  type CompilerCode;
  fn compile(&self, code: &str) -> Result<Box<Self::CompilerCode>>;
}

pub trait Executor {
  type ByteCodeType;
  
  fn run(&self, func: Box<Self::ByteCodeType>, inputs: Option<FunctionInputs>) -> Result<FunctionOutputs>;
}

pub trait CompiledFunction {
  type ByteCodeType;
  fn executable(&self) -> &Self::ByteCodeType;
}

pub enum FunctionInputs {
  Http { headers: Vec<(String, String)>, body: String }
}

pub enum FunctionOutputs {
  Http { headers: Vec<(String, String)>, body: String, status_code: u16 },
  None
}

#[derive(Debug)]
pub enum LanguageRuntimeError {

}