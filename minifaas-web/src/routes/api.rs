use anyhow::Error as AnyError;
use askama::Template;
use minifaas_rt::RuntimeConnection;

use log::{debug, error, info, trace, warn};
use minifaas_common::*;
use serde::Deserialize;

use std::str::FromStr;
use std::sync::Arc;

use tide;
use tide::{Body, Request, Response, StatusCode};

type AppSate = (Arc<FaaSDataStore>, RuntimeConnection);

use super::views::LogViewModel;

#[derive(Deserialize)]
struct ReturnTypeOptions {
    format: String,
}

impl ReturnTypeOptions {
    pub fn format(&self) -> String {
        self.format.to_lowercase()
    }
}

///
/// API call to save a function using a JSON object.
///
pub async fn save_function(mut req: Request<AppSate>) -> tide::Result {
    let item: UserFunctionDeclaration = req.body_json().await?;
    let name = &item.name;
    debug!(
        "Name: {}, Trigger: {:?}, Code: {}",
        name, item.trigger, item.code
    );
    if !name.trim().is_empty() {
        let (storage, connection) = req.state();

        let new_record = match storage.get(&name).await {
            Some(f) => {
                // if a function is already saved it needs to be updated
                let env_id = f.environment_id.clone();

                // ... and for that we have to disable the function
                connection.send(RuntimeRequest::Disable(f)).await?;

                // ... and create a new record with the same environment as the old one
                UserFunctionRecord::new(item.clone(), env_id)
            }
            None => UserFunctionRecord::from(item.clone()),
        };

        // replace the exisiting function
        storage.set(name.clone(), new_record).await;
        let code = storage
            .get(&name)
            .await
            .ok_or_else(|| AnyError::msg(format!("Function couldn't be found: {}", name)))?;

        connection.send(RuntimeRequest::NewFunction(code)).await?;
        Ok(Response::new(StatusCode::Ok))
    } else {
        error!("ERROR :(");

        Err(tide::Error::from_str(
            StatusCode::BadRequest,
            format!("Name '{}' is invalid", name),
        ))
    }
}

pub async fn remove_function(req: Request<AppSate>) -> tide::Result {
    let (storage, _) = req.state();
    let name = req.param("name")?;
    if !name.trim().is_empty() {
        storage.delete(name).await;
        Ok(Response::new(StatusCode::Ok))
    } else {
        Err(tide::Error::from_str(
            StatusCode::BadRequest,
            format!("Name '{}' is invalid", name),
        ))
    }
}

pub async fn get_logs(req: Request<AppSate>) -> tide::Result {
    let (storage, connection) = req.state();
    let name = req.param("name")?;
    let output_format: ReturnTypeOptions = req.query().unwrap_or(ReturnTypeOptions {
        format: "json".to_owned(),
    });
    let from = req.param("from")?.parse::<usize>()?;
    let lines = req.param("lines")?.parse::<usize>()?;

    if let Some(user_func) = storage.get(name).await {
        let logs = connection
            .send(RuntimeRequest::FetchLogs {
                env_id: user_func.environment_id,
                start_line: from,
                lines: lines,
            })
            .await
            .map(|r| {
                if let RuntimeResponse::LogResponse(s) = r {
                    s
                } else {
                    panic!("The Runtime returned the wrong response")
                }
            })?;
        let view_model = LogViewModel { logs };
        let mut resp = Response::new(StatusCode::Ok);
        match output_format.format().as_str() {
            "json" => {
                resp.set_body(Body::from_json(&view_model)?);
                Ok(resp)
            }
            "html" => {
                let rendered = view_model.render()?;
                resp.set_body(Body::from_string(rendered));
                resp.set_content_type(http_types::Mime::from_str("text/html;charset=utf-8")?);
                Ok(resp)
            }
            _ => {
                resp.set_status(StatusCode::BadRequest);
                Ok(resp)
            }
        }
    } else {
        Err(tide::Error::from_str(
            StatusCode::BadRequest,
            format!("{} not found", name),
        ))
    }
}

pub async fn list_all_functions(req: Request<AppSate>) -> tide::Result {
    let (storage, _) = req.state();
    let mut resp = Response::new(StatusCode::Ok);
    resp.set_body(Body::from_json(&storage.values().await)?);
    Ok(resp)
}
