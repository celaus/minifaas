use crate::triggers::Trigger;
use std::boxed::Box;
mod javascript;
use crate::errors::LanguageRuntimeError;
use crate::triggers::{FunctionInputs, FunctionOutputs};
use std::collections::HashMap;
use std::sync::Arc;

type Result<T> = std::result::Result<T, LanguageRuntimeError>;

pub enum SupportedToolchains {
  JavaScript
}

pub trait Compiler {
  type CompilerCode;
  fn compile(&self, code: &str) -> Result<Box<Self::CompilerCode>>;
}

pub trait Executor {
  type ByteCodeType;
  
  fn run(&self, func: Arc<Box<Self::ByteCodeType>>, inputs: Option<FunctionInputs>) -> Result<FunctionOutputs>;
}

pub trait CompiledFunction {
  type ByteCodeType;
  fn executable(&self) -> &Self::ByteCodeType;
}

pub fn load_toolchains() -> HashMap<SupportedToolchains, (impl Compiler, impl Executor)> {
  javascript::load_toolchain()
}