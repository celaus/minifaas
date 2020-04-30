use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder, error::ErrorInternalServerError};
use chrono::prelude::*;
use minifaas_rt::languages::{Compiler, Executor, javascript};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::boxed::Box;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;

use askama::Template;

#[derive(Serialize, Deserialize)]
#[serde(tag = "lang")]
enum ProgrammingLanguage {
    JavaScript,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "trigger")]
enum Trigger {
    Http { method: String },
}

#[derive(Serialize, Deserialize)]
struct UserFunctionDeclaration {
    name: String,
    code: String,
    #[serde(flatten)]
    trigger: Trigger,
    #[serde(flatten)]
    language: ProgrammingLanguage,
    timestamp: DateTime<Utc>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexViewModel<'a> {
    functions: Vec<&'a Box<UserFunctionDeclaration>>,
}

async fn call_function(
    storage: web::Data<Mutex<HashMap<String, Box<UserFunctionDeclaration>>>>,
    name: web::Path<String>,
) -> HttpResponse {
    let name = name.to_string();
    if let Some(func) = (*storage).lock().unwrap().get(&name) {
        let svc = javascript::DuccJS {};
        let compiled = svc.compile(&func.code).unwrap();
        svc.run(compiled, None);
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::BadRequest().finish()
    }
}

async fn add_new_function(
    storage: web::Data<Mutex<HashMap<String, Box<UserFunctionDeclaration>>>>,
    item: web::Json<UserFunctionDeclaration>,
) -> HttpResponse {
    let name = item.name.clone();
    (*storage)
        .lock()
        .unwrap()
        .insert(name, Box::new(item.into_inner()));
    HttpResponse::Ok().finish()
}

async fn list_all_functions(
    storage: web::Data<Mutex<HashMap<String, Box<UserFunctionDeclaration>>>>,
) -> HttpResponse {
    let mut functions = vec![];
    let raw = (*storage).lock().unwrap();
    for function in raw.values() {
        functions.push(function.clone())
    }
    HttpResponse::Ok().json(functions)
}

#[actix_web::get("/")]
async fn index(
    storage: web::Data<Mutex<HashMap<String, Box<UserFunctionDeclaration>>>>,
) -> actix_web::Result<HttpResponse> {
    let mut functions = vec![];
    let raw = (*storage).lock().unwrap();
    for function in raw.values() {
        functions.push(function.clone())
    }
    IndexViewModel { functions }.render().map(|body|{
        HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body)
    })
    .map_err(|_| ErrorInternalServerError("Some error message"))

}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();
    println!(
        "serialized {}",
        serde_json::to_string_pretty(&UserFunctionDeclaration {
            name: "abc".to_owned(),
            code: "more abc".to_owned(),
            trigger: Trigger::Http {
                method: "GET".to_owned()
            },
            language: ProgrammingLanguage::JavaScript,
            timestamp: Utc::now()
        })
        .unwrap()
    );

    let mut functions_db: HashMap<String, Box<UserFunctionDeclaration>> = HashMap::new();
    let storage = web::Data::new(Mutex::new(functions_db));

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(storage.clone()) // add shared state
            .data(web::JsonConfig::default()) // <- limit size of the payload (global configuration)
            .service(
                web::scope("/api/v1/")
                    .service(web::resource("/f").route(web::put().to(add_new_function)))
                    .service(web::resource("/functions").route(web::get().to(list_all_functions))),
            )
            .service(
                web::scope("/f/")
                    .service(web::resource("/call/{name}").route(web::get().to(call_function))),
            )
            .service(index)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
