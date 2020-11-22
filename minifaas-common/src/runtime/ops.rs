

use crate::{FunctionOutputs, FunctionInputs};
use crate::triggers::http::HttpTriggerOutputs;
use crate::UserFunctionRecord;
use crate::{errors::ExecutionError, ProgrammingLanguage};
use std::boxed::Box;
use std::sync::Arc;
use uuid::Uuid;

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
    /// Disable a specific function
    ///
    Disable(Arc<Box<UserFunctionRecord>>),

    ///
    /// Call a function by shipping the code and a response channel.
    ///
    FunctionCall(Arc<Box<UserFunctionRecord>>, FunctionInputs),

    ///
    /// Start a new executor
    ///
    NewFunction(Arc<Box<UserFunctionRecord>>),

    ///
    /// Deletes environment and cleans up
    ///
    DeleteFunction(Arc<Box<UserFunctionRecord>>),

    FetchLogs {
        env_id: Uuid,
        start_line: usize,
        lines: usize,
    },
}

///
/// Response message of the Function runtime
///
#[xactor::message]
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

    LogResponse(String),

    Ok,
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

impl From<HttpTriggerOutputs> for RuntimeResponse {
    fn from(fnout: HttpTriggerOutputs) -> Self {
        RuntimeResponse::FunctionResponse(FunctionOutputs::Http(fnout))
    }
}
