
use anyhow::Result;
use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Settings {

    #[envconfig(from = "MF_ADDR", default = "0.0.0.0:6200")]
    pub endpoint: String,

    #[envconfig(from = "MF_WEB_STATIC_DIR", default = "static")]
    pub static_dir_path: String,

    #[envconfig(from = "MF_DB_PATH", default = "functions.db")]
    pub functions_db_path: String,

    #[envconfig(from = "MF_ENV_ROOT_PATH", default = "/tmp")]
    pub env_root: String,

    #[envconfig(from = "MF_NO_RUNTIME_THREADS", default = "15")]
    no_threads_raw: String,

    #[envconfig(from = "MF_TICK_EVERY_MS", default = "1000")]
    timer_tick_ms_raw: String,

    #[envconfig(from = "MF_MAX_FUNCTION_RUNTIME_SECS", default = "300")]
    max_runtime_secs_raw: String,
}

impl Settings {
    
    pub fn timer_tick_ms(&self) -> Result<i64> { self.timer_tick_ms_raw.parse().map_err(anyhow::Error::from) }
    
    pub fn max_runtime_secs(&self) -> Result<u64> { self.max_runtime_secs_raw.parse().map_err(anyhow::Error::from) }

    pub fn no_threads(&self) -> Result<usize> { self.no_threads_raw.parse().map_err(anyhow::Error::from) }
}