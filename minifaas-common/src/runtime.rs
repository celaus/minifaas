use crate::{errors::ExecutionError, FunctionCode, ProgrammingLanguage};
use crossbeam_channel::Sender;
use std::boxed::Box;
use std::collections::HashMap; // used for compatibility reasons
use std::sync::Arc;



///
/// Request some execution from the runtime.
/// 
#[derive(Debug, Clone)]
pub enum RuntimeRequest {

  ///
  /// Graceful shutdown
  /// 
  Shutdown,

  ///
  /// Call a function by shipping the code and a response channel.
  /// 
  FunctionCall(
    Arc<Box<FunctionCode>>,
    FunctionInputs,
    Sender<RuntimeResponse>,
  ),
}


///
/// Response message of the Function runtime
/// 
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum RuntimeResponse {

  ///
  /// Response of successful Function invocation. Contains whatever the appropriate outputs are.
  /// 
  FunctionResponse(FunctionOutputs),
  
  ///
  /// There is no runtime available for the language specified.
  /// 
  FunctionRuntimeUnavailable(ProgrammingLanguage),

  ///
  /// Something happened during the execution (e.g. a syntax error)
  /// 
  FunctionExecutionError {
    message: String,
    context: Vec<String>,
  },

  ///
  /// Some other runtime error happened during the execution (e.g. unresolved imports or so)
  /// 
  FunctionRuntimeError {
    context: Vec<String>,
  },
}

impl From<ExecutionError> for RuntimeResponse {
  fn from(error: ExecutionError) -> Self {
    match error {
      ExecutionError::CompilerError(message, context) => {
        RuntimeResponse::FunctionExecutionError { message, context }
      }
      ExecutionError::GeneralExecutionError(context) => {
        RuntimeResponse::FunctionRuntimeError { context }
      }
      _ => RuntimeResponse::FunctionRuntimeError {
        context: vec!["The runtime threw some error. Check the server logs.".to_owned()],
      },
    }
  }
}

///
/// Input parameters for functions. 
/// 
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FunctionInputs {

  ///
  /// Available fields for a HTTP trigger
  /// 
  Http {
    params: HashMap<String, Option<String>>,
    headers: HashMap<String, Option<String>>,
    body: Vec<u8>,
  },
}

///
/// Output parameters for functions that will be returned on successful invocation. 
/// 
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FunctionOutputs {

  ///
  /// Available fields for a HTTP response
  /// 
  Http {
    headers: HashMap<String, Option<String>>,
    body: String,
    status_code: u16,
  },

  ///
  /// No function output :(
  /// 
  None,
}
