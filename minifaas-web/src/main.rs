use actix_web::{
    error::ErrorInternalServerError, middleware, web, App, HttpResponse, HttpServer, Responder,
};
use chrono::prelude::*;
use minifaas_rt::languages::{javascript, Compiler, Executor};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::boxed::Box;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use actix_files as fs;
use uuid::Uuid;
use askama::Template;

#[derive(Serialize, Deserialize)]
#[serde(tag = "lang")]
enum ProgrammingLanguage {
    JavaScript,
}

impl std::fmt::Display for ProgrammingLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            ProgrammingLanguage::JavaScript => "JavaScript".to_owned(),
        };
        write!(f, "{}", text)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "trigger")]
enum Trigger {
    Http { method: String },
}


impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            Trigger::Http { method } => format!("Http trigger ({})", method),
        };
        write!(f, "{}", text)
    }
}

#[derive(Serialize, Deserialize)]
struct UserFunctionDeclaration {
    id: String,
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
    triggers: Vec<Trigger>,
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
    IndexViewModel { functions, triggers: vec![Trigger::Http { method: "GET".to_owned() }] }
        .render()
        .map(|body| {
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(body)
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
            id: "id..".to_owned(),
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
            .service(fs::Files::new("/assets", "./minifaas-web/static").show_files_listing())
            .service(
                web::scope("/api/v1/")
                    .service(web::resource("/f").route(web::put().to(add_new_function)))
                    .service(web::resource("/functions").route(web::get().to(list_all_functions))),
            )
            .service(
                web::scope("/f/")
                    .service(web::resource("/call/{name}").to(call_function)),
            )
            .service(index)

    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
