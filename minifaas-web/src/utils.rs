use tide::http::headers::{Headers, HeaderValues, HeaderName};
use tide::StatusCode;
use regex::Regex;
use std::collections::HashMap;


///
/// Creates a HashMap<String, Option<String>> from actix's HeaderMap.
///
pub async fn headers_to_map<'a, I: Iterator<Item=(&'a HeaderName, &'a HeaderValues)>>(headers: &'a mut I) -> HashMap<String, Option<String>> {
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

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;  

}
