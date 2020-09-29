
use crate::errors::ConfigError;
use std::io::Read;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct Settings {
    pub server: Server,
}

#[derive(Deserialize)]
pub struct Server {
    pub endpoint: String,
}


/// 
/// Reads a TOML-based configuration from a Read object into a Settings object.
/// 
pub fn read_config<T: Read + Sized>(mut f: T) -> Result<Settings, ConfigError> {
    let mut buffer = String::new();
    f.read_to_string(&mut buffer).map_err(ConfigError::Io)?;
    toml::from_str(&buffer).map_err(ConfigError::Parse)
}
