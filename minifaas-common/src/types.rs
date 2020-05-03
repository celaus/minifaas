use serde::{Deserialize, Serialize};

///
/// The programming language the FaaS function is created with. There should be a runtime available for each of the variants except `Unknown`.
/// 
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Serialize, Deserialize)]
#[serde(tag = "trigger")]
pub enum Trigger {

    /// Execute on a specified HTTP call
    Http { method: String },

    /// Disable a function
    None,
}

impl Default for Trigger {
    fn default() -> Self {
        Trigger::None
    }
}

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            Trigger::Http { method } => format!("Http trigger ({})", method),
            Trigger::None => "No trigger".to_owned(),
        };
        write!(f, "{}", text)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct UserFunctionDeclaration {
    pub name: String,
    #[serde(flatten)]
    pub code: FunctionCode,
    #[serde(flatten)]
    pub trigger: Trigger,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FunctionCode {
    pub code: String,
    #[serde(flatten)]
    pub language: ProgrammingLanguage,
}

impl FunctionCode {
    pub fn new(code: String, language: ProgrammingLanguage) -> FunctionCode {
        FunctionCode { code, language }
    }
}

impl std::fmt::Display for FunctionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}
