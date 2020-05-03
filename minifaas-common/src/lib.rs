mod types;
pub mod errors;
mod runtime;

pub use crate::types::*;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::boxed::Box;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

pub use runtime::{FunctionInputs, FunctionOutputs, RuntimeRequest, RuntimeResponse};

pub struct DataStoreConfig {
    path: PathBuf,
}

impl DataStoreConfig {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        DataStoreConfig { path: path.into() }
    }
}

#[derive(Debug)]
pub struct FaaSDataStore {
    store: RwLock<HashMap<String, Arc<Box<FunctionCode>>>>,
}

impl FaaSDataStore {
    pub fn new() -> FaaSDataStore {
        FaaSDataStore {
            store: RwLock::new(HashMap::new()),
        }
    }

    pub fn from_path(path: &Path, create_if_missing: bool) -> std::io::Result<FaaSDataStore> {
        // TODO deserialize
        Ok(FaaSDataStore::new())
    }

    pub fn set(&self, key: String, value: FunctionCode) {
        let mut writer = self.store.write().unwrap();
        writer.insert(key, Arc::new(Box::new(value)));
    }

    pub fn get(&self, key: &str) -> Option<Arc<Box<FunctionCode>>> {
        let reader = self.store.read().unwrap();
        reader.get(key).map(|v| v.clone())
    }

    pub fn values(&self) -> Vec<Arc<Box<FunctionCode>>> {
        let reader = self.store.read().unwrap();
        reader.values().map(|v| v.clone()).collect()
    }

    pub fn keys(&self) -> Vec<String> {
        let reader = self.store.read().unwrap();
        reader.keys().map(|v| v.clone()).collect()
    }

    pub fn items(&self) -> Vec<(String, Arc<Box<FunctionCode>>)> {
        let reader = self.store.read().unwrap();
        reader.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}

pub fn create_or_load_storage(config: DataStoreConfig) -> std::io::Result<FaaSDataStore> {
    FaaSDataStore::from_path(&config.path, true)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
