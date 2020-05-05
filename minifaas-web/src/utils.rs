use actix_web::http::header::HeaderMap;
use std::collections::HashMap;

pub async fn query_to_map(query_str: &str) -> HashMap<String, Option<String>> {
  let parts: Vec<&str> = query_str.split("=").collect();
  parts.chunks(2).fold(HashMap::new(), |mut hm, kv| {
    if kv.len() > 0 {
      let k = kv[0];
      let v = kv.get(1).and_then(|val| Some((*val).to_owned()));
      hm.insert((*k).to_owned(), v);
    }
    hm
  })
}

pub async fn headers_to_map(headers: &HeaderMap) -> HashMap<String, Option<String>> {
  headers.iter().fold(HashMap::new(), |mut hm, (h, v)| {
    let val = v.to_str().ok().and_then(|v| Some(v.to_owned()));
    let name = h.as_str().to_owned();
    hm.insert(name, val);
    hm
  })
}