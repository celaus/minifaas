pub mod traits;
pub mod languages;
pub mod triggers;

use std::{env};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use std::boxed::Box;
use languages::{Compiler, Executor};
use serde_json::{Result, Value};

#[derive(Serialize,Deserialize)]
#[serde(tag = "lang")]
enum ProgrammingLanguage {
    JavaScript,
}

#[derive(Serialize,Deserialize)]
#[serde(tag = "trigger")]
enum Trigger {
    Http { method: String },
}


#[derive(Serialize,Deserialize)]
struct UserFunctionDeclaration {
    name: String,
    code: String,
    
    #[serde(flatten)]
    trigger: Trigger,

    #[serde(flatten)]
    language: ProgrammingLanguage,
}


// async fn call_function(storage: web::Data<Mutex<HashMap<String, Box<UserFunctionDeclaration>>>>, name: web::Path<String>) -> HttpResponse {
//     let name = name.to_string();
//     if let Some(func) = (*storage).lock().unwrap().get(&name) {
//         let svc = languages::javascript::DuccJS {};
    
//         let compiled = svc.compile(&func.code).unwrap();
//         svc.run(compiled, None);
//         HttpResponse::Ok().finish()
//     }
//     else{
//         HttpResponse::BadRequest().finish()
//     }
// }

// async fn add_new_function(storage: web::Data<Mutex<HashMap<String, Box<UserFunctionDeclaration>>>>, item: web::Json<UserFunctionDeclaration>) -> HttpResponse {
//     let name = item.name.clone();
//     (*storage).lock().unwrap().insert(name, Box::new(item.into_inner()));
//     HttpResponse::Ok().finish()
// }

// async fn list_all_functions(storage: web::Data<Mutex<HashMap<String, Box<UserFunctionDeclaration>>>>) -> HttpResponse {

//     let mut functions = vec![];
//     let raw = (*storage).lock().unwrap();
//     for function in raw.values() {
//         functions.push(function.clone())
//     }
//     HttpResponse::Ok().json(functions)
// }

