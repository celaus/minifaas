mod http;


pub trait Trigger {

}

pub enum FunctionInputs {
  Http { headers: Vec<(String, String)>, body: String }
}

pub enum FunctionOutputs {
  Http { headers: Vec<(String, String)>, body: String, status_code: u16 },
  None
}
