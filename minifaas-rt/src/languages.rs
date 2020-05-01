mod javascript;
use crate::errors::LanguageRuntimeError;
pub use javascript::JavaScript;

type Result<T> = std::result::Result<T, LanguageRuntimeError>;

pub enum SupportedToolchains {
  JavaScript,
}

pub struct Runtime {}

pub struct FunctionCode {
  code: String,
}

impl FunctionCode {
  pub fn new(code: String) -> FunctionCode {
    FunctionCode { code }
  }
}

pub trait CompiledCode {}
pub trait SourceCode {
  fn str_source(&self) -> &str;
}

impl SourceCode for FunctionCode {
  fn str_source(&self) -> &str {
    &self.code
  }
}
