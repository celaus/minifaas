mod javascript;
pub use javascript::JavaScript;
use minifaas_common::{FunctionCode, errors::ExecutionError};

type Result<T> = std::result::Result<T, ExecutionError>;

pub enum SupportedToolchains {
    JavaScript,
}

pub struct Runtime {}
impl Runtime {
    pub fn new() -> Self {
        Runtime {}
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
