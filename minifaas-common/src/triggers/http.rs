use crate::types::HttpMethod;
use std::collections::HashMap;

#[xactor::message(result = "anyhow::Result<HttpTriggerOutputs>")]
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct HttpTrigger {
    pub route: String,
    pub method: HttpMethod,
    pub params: HashMap<String, Option<Vec<String>>>,
    pub headers: HashMap<String, Option<String>>,
    pub body: Vec<u8>,
}

#[xactor::message]
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct HttpTriggerOutputs {
    pub headers: HashMap<String, Option<String>>,
    pub body: Vec<u8>,
    pub status_code: u16,
}
