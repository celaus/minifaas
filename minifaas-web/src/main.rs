mod config;
mod defaults;
mod utils;

mod routes;

use routes::*;

use anyhow::Result;
use clap::{App as ClApp, Arg};
use config::{read_config, Settings};
use minifaas_rt::RuntimeConnection;

use log::{debug, error, info, trace, warn};
use minifaas_common::*;
use minifaas_rt::{create_runtime, RuntimeConfiguration};

use std::fs::File;
use std::sync::Arc;

use tide;
const FUNC_CALL_PATH: &str = "f";
const API_VERSION: &str = "v1";

async fn start_runtime(settings: &Settings) -> Result<(Arc<FaaSDataStore>, RuntimeConnection)> {
    // set up connections to aux projects
    let _storage = Arc::new(
        create_or_load_storage(DataStoreConfig::new(&settings.runtime.db_path, true)).await?,
    );
    let predefined_envs = sync_environments(&settings.runtime.env_root, _storage.clone()).await?;

    let runtime_connection = create_runtime(
        RuntimeConfiguration::new(settings.runtime.no_threads, settings.runtime.timer_tick_ms),
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
    app.at("/assets").serve_dir("./minifaas-web/static")?;
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
    app.listen(settings.server.endpoint.to_owned()).await?;
    Ok(())
}

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
        .get_matches();

    let config_filename = matches.value_of("config").unwrap_or("config.toml");

    env_logger::init();

    info!("Using configuration file '{}'", config_filename);
    debug!(
        "serialized {}",
        serde_json::to_string_pretty(&UserFunctionDeclaration::default()).unwrap()
    );
    let mut f = File::open(config_filename).expect("Could not open config file.");
    let settings: Settings = read_config(&mut f).expect("Could not read config file.");

    let (storage, runtime_channel) = start_runtime(&settings).await?;

    start_web_server(&settings, storage, runtime_channel).await
}
