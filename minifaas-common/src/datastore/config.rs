pub use crate::types::*;
use async_std::sync::{Arc, RwLock};
use log::{debug, error, info, trace, warn};
use std::path::PathBuf;


///
/// DataStore's configuration options
///
pub struct DataStoreConfig {
    pub path: PathBuf,
    pub serialize_on_write: bool,
}

impl DataStoreConfig {
    pub fn new(path: impl Into<PathBuf>, serialize_on_write: bool) -> Self {
        DataStoreConfig {
            path: path.into(),
            serialize_on_write,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use minifaas_test::get_empty_tmp_dir;
   
}
