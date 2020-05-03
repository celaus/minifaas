
#[derive(Clone, Debug)]
pub enum ExecutionError {
  GeneralExecutionError(Vec<String>),
  CompilerError(String, Vec<String>),
  RuntimeDeadError
}

