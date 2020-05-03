use actix_files as fs;
use actix_web::{
    error::ErrorBadRequest, error::ErrorInternalServerError, http::header::HeaderName,
    http::header::HeaderValue, http::StatusCode, middleware, web, App, HttpResponse, HttpServer,
    Responder,
};
use askama::Template;
use chrono::prelude::*;
use crossbeam_channel::{bounded, Sender};
use minifaas_common::*;
use minifaas_rt::{create_runtime, RuntimeConfiguration};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::boxed::Box;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexViewModel {
    functions: Vec<String>,
    triggers: Vec<Trigger>,
}

async fn call_function(
    storage: web::Data<FaaSDataStore>,
    runtime: web::Data<Sender<RuntimeRequest>>,
    name: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let name = name.to_string();
    if let Some(user_func) = storage.as_ref().get(&name) {
        let timeout = Duration::from_secs(300); // 5 minutes timeout
        let (tx, rx) = bounded::<RuntimeResponse>(1); // create a channel for a single message
        let _ = runtime.as_ref().send(RuntimeRequest::FunctionCall(
            user_func,
            FunctionInputs::Http {
                headers: HashMap::<String, String>::new(),
                body: "".to_owned(),
            },
            tx,
        ));
        let func_output = web::block(move || {
            let result = rx.recv_timeout(timeout);
            drop(rx);
            result
        })
        .await
        .unwrap();
        match func_output {
            RuntimeResponse::FunctionResponse(resp) => {
                if let FunctionOutputs::Http {
                    headers,
                    body,
                    status_code,
                } = resp
                {
                    let mut builder = HttpResponse::build(
                        StatusCode::from_u16(status_code).unwrap_or(StatusCode::BAD_REQUEST),
                    );
                    let mut response = headers.iter().fold(&mut builder, |out, (n, v)| {
                        let val = HeaderValue::from_str(v).ok();
                        let name = HeaderName::from_lowercase(n.to_lowercase().as_bytes()).ok();
                        if val.is_some() && name.is_some() {
                            out.header(name.unwrap(), val.unwrap())
                        } else {
                            out
                        }
                    });
                    Ok(response.body(body))
                } else {
                    Err(ErrorBadRequest("Weird response"))
                }
            }
            RuntimeResponse::FunctionRuntimeUnavailable(lang) => Err(ErrorBadRequest(lang)),
            RuntimeResponse::FunctionExecutionError { message, context } => {
                Err(ErrorBadRequest(context.join("\n")))
            } // <- find a good way to return execution errors (stack traces etc)>
            _ => Err(ErrorInternalServerError("Some error message")),
        }
    } else {
        Err(ErrorInternalServerError("Some error message"))
    }
}

async fn add_new_function(
    storage: web::Data<FaaSDataStore>,
    item: web::Json<UserFunctionDeclaration>,
) -> HttpResponse {
    let name = item.name.clone();
    storage.as_ref().set(name, item.into_inner().code);
    HttpResponse::Ok().finish()
}

async fn get_function_code(
    storage: web::Data<FaaSDataStore>,
    name: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let name = name.to_string();
    if let Some(user_func) = storage.as_ref().get(&name) {
        Ok(HttpResponse::Ok().json(user_func))
    } else {
        Err(actix_web::error::ErrorNotFound(format!(
            "{} not found",
            name
        )))
    }
}

async fn list_all_functions(storage: web::Data<FaaSDataStore>) -> HttpResponse {
    HttpResponse::Ok().json(storage.as_ref().values())
}

#[actix_web::get("/")]
async fn index(storage: web::Data<FaaSDataStore>) -> actix_web::Result<HttpResponse> {
    let functions = storage.as_ref().keys();
    IndexViewModel {
        functions,
        triggers: vec![Trigger::Http {
            method: "GET".to_owned(),
        }],
    }
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
        serde_json::to_string_pretty(&UserFunctionDeclaration::default()).unwrap()
    );

    let storage = create_or_load_storage(DataStoreConfig::new("functions.db"))?;

    let runtime = create_runtime(RuntimeConfiguration::new(10));
    //let mut functions_db: HashMap<String, Box<UserFunctionDeclaration>> = HashMap::new();
    //let storage = web::Data::new(Mutex::new(functions_db));
    let storage = web::Data::new(storage);
    let runtime = web::Data::new(runtime);
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(storage.clone()) // add shared state
            .app_data(runtime.clone())
            .data(web::JsonConfig::default()) // <- limit size of the payload (global configuration)
            .service(fs::Files::new("/assets", "./minifaas-web/static").show_files_listing())
            .service(
                web::scope("/api/v1/")
                    .service(web::resource("/f").route(web::put().to(add_new_function)))
                    .service(web::resource("/functions").route(web::get().to(list_all_functions))),
            )
            .service(
                web::scope("/f/")
                    .service(web::resource("/call/{name}").to(call_function))
                    .service(web::resource("/impl/{name}").route(web::get().to(get_function_code))),
            )
            .service(index)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
