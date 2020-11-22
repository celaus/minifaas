use minifaas_common::triggers::http::HttpMethod;
use std::collections::HashMap;
use tide::http::headers::{HeaderName, HeaderValues};
use tide::StatusCode;

///
/// Creates a HashMap<String, Option<String>> from actix's HeaderMap.
///
pub async fn headers_to_map<'a, I: Iterator<Item = (&'a HeaderName, &'a HeaderValues)>>(
    headers: &'a mut I,
) -> HashMap<String, Option<String>> {
    headers.fold(HashMap::new(), |mut hm, (h, v)| {
        let val = Some(v.as_str().to_owned());
        let name = h.as_str().to_owned();
        hm.insert(name, val);
        hm
    })
}

pub async fn _500<S: Into<String>>(msg: S) -> tide::Error {
    tide::Error::from_str(StatusCode::InternalServerError, msg.into())
}

pub async fn _400<S: Into<String>>(msg: S) -> tide::Error {
    tide::Error::from_str(StatusCode::BadRequest, msg.into())
}

pub fn convert_http_method(other: tide::http::Method) -> HttpMethod {
    match other {
        tide::http::Method::Get => HttpMethod::GET,
        tide::http::Method::Head => HttpMethod::HEAD,
        tide::http::Method::Post => HttpMethod::POST,
        tide::http::Method::Put => HttpMethod::PUT,
        tide::http::Method::Delete => HttpMethod::DELETE,
        tide::http::Method::Connect => HttpMethod::CONNECT,
        tide::http::Method::Options => HttpMethod::OPTIONS,
        tide::http::Method::Trace => HttpMethod::TRACE,
        tide::http::Method::Patch => HttpMethod::PATCH,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
}
