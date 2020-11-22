
use anyhow::Result;
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
pub fn read_config<T: Read + Sized>(mut f: T) -> Result<Settings> {
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;
    toml::from_str(&buffer).map_err(anyhow::Error::from)
}
