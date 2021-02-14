use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl HttpMethod {
    pub fn available() -> Vec<Self> {
        vec![
            HttpMethod::ALL,
            HttpMethod::CONNECT,
            HttpMethod::GET,
            HttpMethod::POST,
            HttpMethod::OPTIONS,
            HttpMethod::HEAD,
            HttpMethod::PATCH,
            HttpMethod::DELETE,
            HttpMethod::PUT,
            HttpMethod::TRACE,
        ]
    }
}

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

impl From<HashMap<String, Vec<u8>>> for HttpTriggerOutputs {
    fn from(map: HashMap<String, Vec<u8>>) -> Self {
        let mut map = map;
        let headers: HashMap<String, Option<String>> = map
            .get("headers")
            .and_then(|s| serde_json::from_slice(s).ok())
            .unwrap_or_default();
        let body = map.remove("body").unwrap_or_default();
        let status_code = map
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
