use crate::UserFunctionRecord;
use crate::HttpMethod;
use crate::triggers::http::HttpTrigger;
use crate::triggers::http::HttpTriggerOutputs;
use crate::{errors::ExecutionError, FunctionCode, ProgrammingLanguage};
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
    FunctionCall(Arc<Box<UserFunctionRecord>>, FunctionInputs),

    ///
    /// Compile the code, and start a new executor
    ///
    NewFunction(Arc<Box<UserFunctionRecord>>),
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

///
/// Input parameters for functions.
///
#[xactor::message(result = "anyhow::Result<FunctionOutputs>")]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FunctionInputs {
    ///
    /// Available fields for a HTTP trigger
    ///
    Http(HttpTrigger),
}

impl From<HttpTrigger> for FunctionInputs {
    fn from(t: HttpTrigger) -> Self {
        FunctionInputs::Http(t)
    }
}

///
/// Output parameters for functions that will be returned on successful invocation.
///
#[xactor::message]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FunctionOutputs {
    ///
    /// Available fields for a HTTP response
    ///
    Http(HttpTriggerOutputs),

    ///
    /// No function output :(
    ///
    None,
}


#[xactor::message]
#[derive(Default, Debug)]
pub struct RawFunctionOutput(HashMap<String, Vec<u8>>);

impl From<HashMap<String, Vec<u8>>> for RawFunctionOutput {
    fn from(map: HashMap<String, Vec<u8>>) -> Self {
        RawFunctionOutput(map)
    }
}
impl RawFunctionOutput {
    pub fn into_http(&mut self) -> HttpTriggerOutputs {
        let headers: HashMap<String, Option<String>> = self
            .0
            .get("headers")
            .and_then(|s| serde_json::from_slice(s).ok())
            .unwrap_or_default();
            
        let body = self.0.remove("body").unwrap_or_default();
        let status_code = self
            .0
            .get("status_code")
            .and_then(|c| std::str::from_utf8(c).map(|r| str::parse(r)).ok())
            .unwrap_or(Ok(200))
            .unwrap();

        HttpTriggerOutputs {
            headers,
            body,
            status_code,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FnInputValue {
    Coll(Vec<String>),
    Raw(Vec<u8>),
    Str(String),
    Map(HashMap<String, Option<String>>),
    MapColl(HashMap<String, Option<Vec<String>>>),
    Type(HttpMethod),
}

#[xactor::message(result = "anyhow::Result<RawFunctionOutput>")]
#[derive(Default)]
pub struct RawFunctionInput(HashMap<String, FnInputValue>);

impl From<HttpTrigger> for RawFunctionInput {
    fn from(input: HttpTrigger) -> Self {
        let map: HashMap<String, FnInputValue> = vec![
            (String::from("body"), FnInputValue::Raw(input.body)),
            (String::from("headers"), FnInputValue::Map(input.headers)),
            (String::from("params"), FnInputValue::MapColl(input.params)),
            (String::from("method"), FnInputValue::Type(input.method)),
            (String::from("route"), FnInputValue::Str(input.route)),
        ]
        .into_iter()
        .collect();
        RawFunctionInput(map)
    }
}
