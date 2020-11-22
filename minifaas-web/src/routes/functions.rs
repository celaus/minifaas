use crate::utils::convert_http_method;
use minifaas_rt::RuntimeConnection;

use crate::utils;
use log::{debug, error, info, trace, warn};
use minifaas_common::triggers::http::HttpTrigger;
use minifaas_common::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;

use tide;
use tide::http::headers::{HeaderName, HeaderValue};
use tide::{Request, Response, StatusCode};

type AppSate = (Arc<FaaSDataStore>, RuntimeConnection);

///
/// Call a function to
///
pub async fn call_function(mut req: Request<AppSate>) -> tide::Result {
    let bytes = req.body_bytes().await?;

    let (storage, runtime) = req.state();
    let name = req.param("name")?.trim();
    info!("Calling function '{}'", name);
    if let Some(user_func) = storage.get(&name).await {
        let query_params: HashMap<String, Option<Vec<String>>> = req.query().unwrap_or_default();
        let req_headers = utils::headers_to_map(&mut req.iter()).await;
        let func_output = runtime
            .send(RuntimeRequest::FunctionCall(
                user_func,
                FunctionInputs::Http(HttpTrigger {
                    route: name.to_owned(),
                    params: query_params,
                    body: bytes.to_vec(),
                    headers: req_headers,
                    method: convert_http_method(req.method()),
                }),
            ))
            .await?;
        debug!("Function output: {:?}", func_output);
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
        error!("Function with name '{}' not found", name);
        Err(utils::_500("Some error message").await)
    }
}
