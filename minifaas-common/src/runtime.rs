use crate::{errors::ExecutionError, FunctionCode, ProgrammingLanguage};
use crossbeam_channel::Sender;
use std::boxed::Box;
use std::collections::HashMap; // used for compatibility reasons
use std::sync::Arc;

pub enum RuntimeRequest {
  Shutdown,
  FunctionCall(
    Arc<Box<FunctionCode>>,
    FunctionInputs,
    Sender<RuntimeResponse>,
  ),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum RuntimeResponse {
  FunctionResponse(FunctionOutputs),
  FunctionRuntimeUnavailable(ProgrammingLanguage),
  FunctionExecutionError {
    message: String,
    context: Vec<String>,
  },
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

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FunctionInputs {
  Http {
    headers: HashMap<String, String>,
    body: String,
  },
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FunctionOutputs {
  Http {
    headers: HashMap<String, String>,
    body: String,
    status_code: u16,
  },
  None,
}
