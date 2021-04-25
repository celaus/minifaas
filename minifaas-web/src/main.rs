mod config;
mod utils;

mod routes;

use routes::*;

use anyhow::Result;
use clap::{App as ClApp, Arg};
use config::Settings;
use minifaas_rt::RuntimeConnection;
use envconfig::Envconfig;
use log::{debug, info};
use minifaas_common::*;
use minifaas_rt::{create_runtime, RuntimeConfiguration};
use std::sync::Arc;

use tide;
const FUNC_CALL_PATH: &str = "f";
const API_VERSION: &str = "v1";
const VERSION: &str = "0.1.0";

async fn start_runtime(settings: &Settings) -> Result<(Arc<FaaSDataStore>, RuntimeConnection)> {
    // set up connections to aux projects
    let _storage = Arc::new(
        create_or_load_storage(DataStoreConfig::new(&settings.functions_db_path, true)).await?,
    );
    let predefined_envs = sync_environments(&settings.env_root, _storage.clone()).await?;

    let runtime_connection = create_runtime(
        RuntimeConfiguration::new(settings.no_threads()?, settings.timer_tick_ms()?),
        predefined_envs,
        _storage.clone(),
    )
    .await?;
    Ok((_storage, runtime_connection))
}

pub async fn start_web_server(
    settings: &Settings,
    storage: Arc<FaaSDataStore>,
    runtime_channel: RuntimeConnection,
) -> Result<()> {
    //
    // Set up routing and start the web server
    //
    let mut app = tide::with_state((storage.clone(), runtime_channel.clone()));
    app.with(tide::log::LogMiddleware::new());
    app.at("/assets").serve_dir(&settings.static_dir_path)?;
    app.at("/").get(index);
    app.at("/api").nest({
        let mut f = tide::with_state((storage.clone(), runtime_channel.clone()));
        f.at(&format!("{}/{}", API_VERSION, FUNC_CALL_PATH))
            .put(save_function);
        f.at(&format!("{}/{}/:name", API_VERSION, FUNC_CALL_PATH))
            .delete(remove_function);
        f.at(&format!("{}/{}", API_VERSION, FUNC_CALL_PATH))
            .get(list_all_functions);
        f.at(&format!("{}/logs/:name/:from/:lines", API_VERSION))
            .get(get_logs);
        f
    });
    app.at("/f/").nest({
        let mut f = tide::with_state((storage.clone(), runtime_channel.clone()));
        f.at("/call/:name").all(call_function);
        f
    });
    app.listen(settings.endpoint.to_owned()).await?;
    Ok(())
}

#[async_std::main]

async fn main() -> Result<()> {
    let matches = ClApp::new("MiniFaaS")
        .version(VERSION)
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
        .get_matches();

    env_logger::init();

    info!(".:::MINIFAAS v{} :::.", VERSION);
    debug!(
        "serialized {}",
        serde_json::to_string_pretty(&UserFunctionDeclaration::default()).unwrap()
    );
    let settings= Settings::init_from_env()?;

    let (storage, runtime_channel) = start_runtime(&settings).await?;

    start_web_server(&settings, storage, runtime_channel).await
}
