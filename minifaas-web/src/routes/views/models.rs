use crate::UserFunctionType;
use minifaas_common::{ProgrammingLanguage, Trigger};
use askama::*;
use serde::{Deserialize, Serialize};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexViewModel {
    pub functions: Vec<UserFunctionType>,
    pub http_triggers: Vec<Trigger>,
    pub programming_languages: Vec<ProgrammingLanguage>,
    pub selected: Option<usize>,
    pub base_url: String,
    pub fn_base_path: String 
}



#[derive(Template, Serialize, Deserialize)]
#[template(source = "{{ logs|linebreaks }}", ext = "html", escape = "none")]
pub struct LogViewModel {
    pub logs: String,
}

