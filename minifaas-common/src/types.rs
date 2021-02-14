use crate::runtime::FunctionCode;
use crate::triggers::Trigger;
use serde::{Deserialize, Serialize};

///
/// The programming language the FaaS function is created with. There should be a runtime available for each of the variants except `Unknown`.
///
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "lang")]
pub enum ProgrammingLanguage {
    /// Vanilla JavaScript
    JavaScript,

    /// Good old bash scripts
    Bash,

    /// No known Programming language
    Unknown,
}

impl std::fmt::Display for ProgrammingLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            ProgrammingLanguage::JavaScript => "JavaScript".to_owned(),
            ProgrammingLanguage::Bash => "Bash".to_owned(),
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

impl ProgrammingLanguage {
    pub fn available() -> Vec<Self> {
        vec![ProgrammingLanguage::JavaScript, ProgrammingLanguage::Bash]
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
