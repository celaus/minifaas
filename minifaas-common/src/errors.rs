use thiserror::Error;

///
/// Errors that come out of executing code inside the runtime. 
/// 
#[derive(Clone, Debug, Error)]
pub enum ExecutionError {

  ///
  /// Catch-all error
  /// 
  #[error("Code execution failed :(")]
  GeneralExecutionError(Vec<String>),

  ///
  /// Errors during compilation, such as syntax errors.
  ///
  #[error("Can't compile code")] 
  CompilerError(String, Vec<String>),

  ///
  /// The runtime had some issue and died. WIP
  /// 
  #[error("The runtime somehow died. Sorry")]
  RuntimeDeadError
}


///
/// Errors that come out of executing code inside the runtime. 
/// 
#[derive(Debug, Error)]
pub enum PreparationError {

  ///
  /// Catch-all error
  /// 
  #[error("Code execution failed :(")]
  EnvironmentSetupFailed(#[from] std::io::Error),

  ///
  /// Catch-all error
  /// 
  #[error("Can't add another environment at: {0}")]
  EnvironmentAddFailed(String),

  ///
  /// The runtime had some issue and died. 
  /// 
  #[error("Environment setup failed somehow.")]
  Unknown
}

