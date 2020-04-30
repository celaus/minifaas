pub enum HttpMethod {
  POST, 
  GET, 
  HEAD,
  OPTION, 
  ALL
}

pub struct HttpTrigger {
  route: String,
  method: HttpMethod,
}