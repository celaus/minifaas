use serde::{Deserialize, Serialize};
use uuid::Uuid;

///
/// The programming language the FaaS function is created with. There should be a runtime available for each of the variants except `Unknown`.
///
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "lang")]
pub enum ProgrammingLanguage {
    /// Vanilla JavaScript
    JavaScript,

    /// No known Programming language
    Unknown,
}

impl std::fmt::Display for ProgrammingLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            ProgrammingLanguage::JavaScript => "JavaScript".to_owned(),
            ProgrammingLanguage::Unknown => "Unknown".to_owned(),
        };
        write!(f, "{}", text)
    }
}

impl Default for ProgrammingLanguage {
    fn default() -> Self {
        ProgrammingLanguage::Unknown
    }
}

///
/// Represents a trigger for the Function as a Service function. Declares the required parameters and so on. Defaults to `None` which means disabled.
///
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(tag = "type", content = "when")]
pub enum Trigger {
    /// Execute on a specified HTTP call
    Http(HttpMethod),

    /// Disable a function
    None,
}

impl Default for Trigger {
    fn default() -> Self {
        Trigger::Http(HttpMethod::default())
    }
}

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            Trigger::Http(method) => format!("Http trigger ({:?})", method),
            Trigger::None => "No trigger".to_owned(),
        };
        write!(f, "{}", text)
    }
}

///
/// A user's declaration of a function. This is intended to be used as data transmission object.
///
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct UserFunctionDeclaration {
    pub name: String,
    #[serde(flatten)]
    pub code: FunctionCode,
    pub trigger: Trigger,
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum HttpMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
    ALL,
}

impl Default for HttpMethod {
    fn default() -> Self {
        HttpMethod::ALL
    }
}
