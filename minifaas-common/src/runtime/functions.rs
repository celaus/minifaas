use crate::triggers::http::HttpMethod;
use crate::triggers::{http::HttpTriggerOutputs, timer::TimerTrigger};
use crate::{triggers::http::HttpTrigger, ProgrammingLanguage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap; // used for compatibility reasons
use uuid::Uuid;

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
    Timer(TimerTrigger),
}

impl From<HttpTrigger> for FunctionInputs {
    fn from(t: HttpTrigger) -> Self {
        FunctionInputs::Http(t)
    }
}

impl From<TimerTrigger> for FunctionInputs {
    fn from(t: TimerTrigger) -> Self {
        FunctionInputs::Timer(t)
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
pub struct RawFunctionOutputWrapper(HashMap<String, Vec<u8>>);

impl From<HashMap<String, Vec<u8>>> for RawFunctionOutputWrapper {
    fn from(map: HashMap<String, Vec<u8>>) -> Self {
        RawFunctionOutputWrapper(map)
    }
}

impl From<RawFunctionOutputWrapper> for HttpTriggerOutputs {
    fn from(raw: RawFunctionOutputWrapper) -> Self {
        HttpTriggerOutputs::from(raw.0)
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

#[xactor::message(result = "anyhow::Result<RawFunctionOutputWrapper>")]
#[derive(Default, Clone)]
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

impl From<TimerTrigger> for RawFunctionInput {
    fn from(input: TimerTrigger) -> Self {
        let map: HashMap<String, FnInputValue> = vec![(
            String::from("when"),
            FnInputValue::Str(input.when.format("%s").to_string()),
        )]
        .into_iter()
        .collect();
        RawFunctionInput(map)
    }
}

///
/// Representation of a Function in code.
///
#[xactor::message]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FunctionCode {
    pub code: String,
    pub language: ProgrammingLanguage,
}

impl FunctionCode {
    pub fn new(code: String, language: ProgrammingLanguage) -> Self {
        FunctionCode::existing(code, language, Uuid::new_v4())
    }

    pub fn existing(code: String, language: ProgrammingLanguage, uuid: Uuid) -> Self {
        FunctionCode { code, language }
    }
}

impl std::fmt::Display for FunctionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}
