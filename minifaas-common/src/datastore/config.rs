use crate::triggers::Trigger;
pub use crate::types::*;
use anyhow::Result;
use async_std::fs::File;
use async_std::io::prelude::*;
use async_std::sync::{Arc, RwLock};
use async_std::task;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::boxed::Box;
use std::collections::HashMap;
use std::fmt;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::path::PathBuf;
use uuid::Uuid;

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
