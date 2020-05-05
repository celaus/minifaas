
///
/// Errors that come out of executing code inside the runtime. 
/// 
#[derive(Clone, Debug)]
pub enum ExecutionError {

  ///
  /// Catch-all error
  /// 
  GeneralExecutionError(Vec<String>),

  ///
  /// Errors during compilation, such as syntax errors.
  /// 
  CompilerError(String, Vec<String>),

  ///
  /// The runtime had some issue and died. WIP
  /// 
  RuntimeDeadError
}

