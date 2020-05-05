use actix_web::http::header::HeaderMap;
use regex::Regex;
use std::collections::HashMap;

///
/// Regular expression for extracting the query string key/value pairs
///
const QUERY_STRING_REGEX: &str = r"(?P<key>[^?=&]+)(=(?P<val>[^&]*))?";

///
/// Splits a query string into a HashMap<String, Option<Vec<String>>> at the '=' character.
///
/// # Examples
///
/// ```
/// let query_params = query_to_map("a=1&b=x&b=y");
/// assert_eq!(query_params["a"], Some(vec!["1".to_owned()]));
/// assert_eq!(query_params["b"], Some(vec!["x".to_owned(), "y".to_owned()]));
/// ```
pub async fn query_to_map(query_str: &str) -> HashMap<String, Option<Vec<String>>> {
  let re = Regex::new(QUERY_STRING_REGEX).unwrap();
  re.captures_iter(query_str)
    .fold(HashMap::new(), |mut hm, cap| {
      let k = &cap["key"];
      let v = cap["val"].to_owned();
      match hm.get_mut(k) {
        Some(val) => {
          let mut list_of_values = val.take().unwrap_or_default();
          list_of_values.push(v);
          (*val) = Some(list_of_values);
        }
        None => {
          hm.insert(k.to_owned(), Some(vec![v]));
        }
      };
      hm
    })
}

///
/// Creates a HashMap<String, Option<String>> from actix's HeaderMap.
///
pub async fn headers_to_map(headers: &HeaderMap) -> HashMap<String, Option<String>> {
  headers.iter().fold(HashMap::new(), |mut hm, (h, v)| {
    let val = v.to_str().ok().map(|v| v.to_owned());
    let name = h.as_str().to_owned();
    hm.insert(name, val);
    hm
  })
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
  use super::*;

  macro_rules! aw {
    ($e:expr) => {
      tokio_test::block_on($e)
    };
  }

  #[test]
  fn test_query_to_map__valid_scalars() {
    let query_params = aw!(query_to_map("a=1&b=x"));
    assert_eq!(query_params["a"], Some(vec!["1".to_owned()]));
    assert_eq!(query_params["b"], Some(vec!["x".to_owned()]));
  }
  #[test]
  fn test_query_to_map__lists() {
    let query_params = aw!(query_to_map("a=1&a=2&a=3"));
    assert_eq!(
      query_params["a"],
      Some(vec!["1".to_owned(), "2".to_owned(), "3".to_owned()])
    );
  }

  #[test]
  fn test_query_to_map__weird_values() {
    let query_params = aw!(query_to_map(
      "a=&a=2&a=3&b=a%20b&c=+++++{\"hello\": \"world\"}"
    ));
    assert_eq!(
      query_params["a"],
      Some(vec!["".to_owned(), "2".to_owned(), "3".to_owned()])
    );
    assert_eq!(query_params["b"], Some(vec!["a%20b".to_owned()]));
    assert_eq!(
      query_params["c"],
      Some(vec!["+++++{\"hello\": \"world\"}".to_owned()])
    );
  }
}
