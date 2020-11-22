mod models;
pub use models::LogViewModel;

use askama::Template;
use minifaas_rt::RuntimeConnection;

use log::{debug, error, info, trace, warn};
use minifaas_common::*;
use serde::Deserialize;

use std::str::FromStr;
use std::sync::Arc;

use tide;
use tide::{Request, Response, StatusCode};

type AppSate = (Arc<FaaSDataStore>, RuntimeConnection);

use models::IndexViewModel;

#[derive(Deserialize)]
struct MainPageShowFunction {
    show: String,
}

///
/// The main page showing all active functions.
///
///
pub async fn index(req: Request<AppSate>) -> tide::Result {
    let (storage, _) = req.state();
    let which: Option<MainPageShowFunction> = req.query().ok();
    let functions = storage.values().await;
    let selected = which
        .map(|w| functions.iter().position(|f| f.name() == &w.show))
        .flatten();
    IndexViewModel {
        functions,
        http_triggers: Trigger::all_http(),
        programming_languages: ProgrammingLanguage::available(),
        selected,
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
