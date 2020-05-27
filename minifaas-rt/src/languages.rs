mod javascript;
pub use javascript::JavaScript;
use minifaas_common::{FunctionCode, errors::ExecutionError};

type Result<T> = std::result::Result<T, ExecutionError>;


///
/// Which toolchains (language runtimes) are currently supported
/// 
pub enum SupportedToolchains {
    JavaScript,
}


///
/// A language runtime.
/// 
/// 
pub struct Runtime {}

impl Runtime {
    pub fn new() -> Self {
        Runtime {}
    }
}

///
/// Bytecode suitable for execution.
/// 
pub trait CompiledCode {}

///
/// A trait for retrieving the source code of a function as string.
/// 
pub trait SourceCode {
    fn str_source(&self) -> &str;
}


impl SourceCode for FunctionCode {
    fn str_source(&self) -> &str {
        &self.code
    }
}
