mod config;
mod errors;
mod utils;

use crate::utils::convert_http_method;
use anyhow::{Error as AnyError, Result};
use askama::Template;
use clap::{App as ClApp, Arg};
use config::{read_config, Settings};
use minifaas_rt::RuntimeConnection;
//use crossbeam_channel::{bounded, Sender};
use log::{debug, error, info, trace, warn};
use minifaas_common::triggers::http::HttpTrigger;
use minifaas_common::*;
use minifaas_rt::{create_runtime, RuntimeConfiguration};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tide;
use tide::http::headers::{HeaderName, HeaderValue};
use tide::{Body, Request, Response, StatusCode};
use uuid::Uuid;

type AppSate = (Arc<FaaSDataStore>, RuntimeConnection);

const MAX_RUNTIME_SECS: u64 = 300;
const NO_RUNTIME_THREADS: usize = 10;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexViewModel {
    functions: Vec<String>,
    triggers: Vec<Trigger>,
}

async fn call_function(mut req: Request<AppSate>) -> tide::Result {
    let bytes = req.body_bytes().await?;

    let (storage, runtime) = req.state();
    let name: String = req.param("name")?;
    info!("Calling function '{}'", name);
    if let Some(user_func) = storage.get(&name).await {
        let query_params: HashMap<String, Option<Vec<String>>> = req.query().unwrap_or_default();
        let req_headers = utils::headers_to_map(&mut req.iter()).await;

        let func_output = runtime
            .send(RuntimeRequest::FunctionCall(
                user_func,
                FunctionInputs::Http(HttpTrigger {
                    route: name,
                    params: query_params,
                    body: bytes.to_vec(),
                    headers: req_headers,
                    method: convert_http_method(req.method()),
                }),
            ))
            .await?;
        info!("Function output: {:?}", func_output);
        match func_output {
            RuntimeResponse::FunctionResponse(resp) => {
                if let FunctionOutputs::Http(http) = resp {
                    let mut builder = Response::new(
                        StatusCode::try_from(http.status_code).unwrap_or(StatusCode::BadRequest),
                    );
                    builder.set_body(http.body);

                    let response = http.headers.iter().fold(builder, |mut out, (n, v)| {
                        let val = HeaderValue::from_str(v.as_ref().unwrap_or(&"".to_owned())).ok();
                        let name =
                            HeaderName::from_bytes(n.to_lowercase().as_bytes().to_vec()).ok();
                        if val.is_some() && name.is_some() {
                            out.insert_header(name.unwrap(), val.unwrap());
                        }
                        out
                    });
                    Ok(response)
                } else {
                    Err(utils::_400(format!("{:?}", resp)).await)
                }
            }
            RuntimeResponse::FunctionRuntimeUnavailable(lang) => {
                Err(utils::_400(format!("{}", lang)).await)
            }
            RuntimeResponse::FunctionExecutionError { message, context } => {
                Err(utils::_400(context.join("\n")).await)
            } // <- find a good way to return execution errors (stack traces etc)>
            _ => Err(utils::_500("Some error message").await),
        }
    } else {
        Err(utils::_500("Some error message").await)
    }
}

async fn add_new_function(mut req: Request<AppSate>) -> tide::Result {
    let item: UserFunctionDeclaration = req.body_json().await?;
    let name = &item.name;
    info!(
        "Name: {}, Trigger: {:?}, Code: {}",
        name, item.trigger, item.code
    );
    if !name.trim().is_empty() {
        let (storage, connection) = req.state();
        storage
            .set(name.clone(), UserFunctionRecord::from(item.clone()))
            .await;
        let code = storage
            .get(&name)
            .await
            .ok_or_else(|| AnyError::msg(format!("Function couldn't be found: {}", name)))?;

        connection.send(RuntimeRequest::NewFunction(code)).await?;
        Ok(Response::new(StatusCode::Ok))
    } else {
        info!("ERROR :(");

        Err(tide::Error::from_str(
            StatusCode::BadRequest,
            format!("Name '{}' is invalid", name),
        ))
    }
}

async fn remove_function(req: Request<AppSate>) -> tide::Result {
    let (storage, _) = req.state();
    let name: String = req.param("name")?;
    if !name.trim().is_empty() {
        storage.delete(&name).await;
        Ok(Response::new(StatusCode::Ok))
    } else {
        Err(tide::Error::from_str(
            StatusCode::BadRequest,
            format!("Name '{}' is invalid", name),
        ))
    }
}

async fn get_function_code(req: Request<AppSate>) -> tide::Result {
    let (storage, _) = req.state();
    let name: String = req.param("name")?;
    if let Some(user_func) = storage.get(&name).await {
        let mut response = Response::new(StatusCode::Ok);
        response.set_body(Body::from_json(&user_func)?);
        Ok(response)
    } else {
        Err(tide::Error::from_str(
            StatusCode::BadRequest,
            format!("{} not found", name),
        ))
    }
}

async fn list_all_functions(req: Request<AppSate>) -> tide::Result {
    let (storage, _) = req.state();
    let mut resp = Response::new(StatusCode::Ok);
    resp.set_body(Body::from_json(&storage.values().await)?);
    Ok(resp)
}

async fn index(req: Request<AppSate>) -> tide::Result {
    let (storage, _) = req.state();
    let functions = storage.keys().await;
    IndexViewModel {
        functions,
        triggers: vec![Trigger::Http(HttpMethod::ALL)],
    }
    .render()
    .map(|body| {
        let mut resp = Response::new(StatusCode::Ok);
        resp.set_body(body);
        resp.set_content_type(http_types::Mime::from_str("text/html;charset=utf-8").unwrap());
        resp
    })
    .map_err(|_| tide::Error::from_str(StatusCode::InternalServerError, ":("))
}

// #[message(result = "String")]
// struct ToUppercase(String);

// struct MyActor;

// impl Actor for MyActor {}

// #[async_trait::async_trait]
// impl Handler<ToUppercase> for MyActor {
//     async fn handle(&mut self, _ctx: &Context<Self>, msg: ToUppercase) -> String {
//         msg.0.to_uppercase()
//     }
// }

#[async_std::main]
async fn main() -> Result<()> {
    let matches = ClApp::new("MiniFaaS")
        .version("0.1.0")
        .author("Claus Matzinger. <claus.matzinger+kb@gmail.com>")
        .about("A no-fluff Function-as-a-Service runtime for home use.")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Sets a custom config file [default: config.toml]")
                .value_name("config.toml")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("logging")
                .short("l")
                .long("logging-conf")
                .value_name("logging.yml")
                .takes_value(true)
                .help("Sets the logging configuration [default: logging.yml]"),
        )
        .arg(
          Arg::with_name("environment")
              .short("e")
              .long("environment-root")
              .value_name("/tmp")
              .takes_value(true)
              .help("Sets the configuration where the executables and file system for functions (i.e. their enviornments) should reside. Default: /tmp"),
      )
        .get_matches();

    let config_filename = matches.value_of("config").unwrap_or("config.toml");
    let logging_filename = matches.value_of("logging").unwrap_or("logging.yml");
    let env_root = matches.value_of("environment").unwrap_or("/tmp");
    info!(
        "Using configuration file '{}' and logging config '{}'",
        config_filename, logging_filename
    );

    log4rs::init_file(logging_filename, Default::default()).expect("Could not initialize log4rs.");
    let mut f = File::open(config_filename).expect("Could not open config file.");
    let settings: Settings = read_config(&mut f).expect("Could not read config file.");

    debug!(
        "serialized {}",
        serde_json::to_string_pretty(&UserFunctionDeclaration::default()).unwrap()
    );
    // set up connections to aux projects
    let _storage =
        Arc::new(create_or_load_storage(DataStoreConfig::new("functions.db", true)).await?);
    let predefined_envs = sync_environments(env_root, _storage.clone()).await?;
    let runtime_channel = create_runtime(
        RuntimeConfiguration::new(NO_RUNTIME_THREADS),
        predefined_envs,
        _storage.clone(),
    )
    .await?;

    let mut app = tide::with_state((_storage.clone(), runtime_channel.clone()));
    app.with(tide::log::LogMiddleware::new());
    app.at("/assets").serve_dir("./minifaas-web/static")?;
    app.at("/").get(index);
    app.at("/api").nest({
        let mut f = tide::with_state((_storage.clone(), runtime_channel.clone()));
        f.at("v1/f").put(add_new_function);
        f.at("v1/f/:name").delete(remove_function);
        f.at("v1/functions").delete(list_all_functions);
        f
    });
    app.at("/f/").nest({
        let mut f = tide::with_state((_storage.clone(), runtime_channel.clone()));
        f.at("/call/:name").all(call_function);
        f.at("/impl/:name").get(get_function_code);
        f
    });
    app.listen(settings.server.endpoint).await?;
    Ok(())
}
