use crate::defaults::{
    default_db_name, default_env_path, default_max_runtime_secs, default_no_runtime_threads,
    default_timer_resolution_ms,
};
use anyhow::Result;
use serde;
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
pub struct Settings {
    pub server: Server,
    pub runtime: Runtime,
}

#[derive(Deserialize)]
pub struct Server {
    pub endpoint: String,
}
#[derive(Deserialize)]
pub struct Runtime {
    #[serde(default = "default_db_name")]
    pub db_path: String,

    #[serde(default = "default_env_path")]
    pub env_root: String,

    #[serde(default = "default_no_runtime_threads")]
    pub no_threads: usize,

    #[serde(default = "default_timer_resolution_ms")]
    pub timer_tick_ms: i64,

    #[serde(default = "default_max_runtime_secs")]
    pub max_runtime_secs: u64,
}

///
/// Reads a TOML-based configuration from a Read object into a Settings object.
///
pub fn read_config<T: Read + Sized>(mut f: T) -> Result<Settings> {
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;
    toml::from_str(&buffer).map_err(anyhow::Error::from)
}
