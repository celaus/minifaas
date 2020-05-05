pub use crate::types::*;
use hashbrown::HashMap;
use std::boxed::Box;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;

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

///
/// A key-value store for the user-defined functions. Uses an RwLock for multi-threaded reads/writes. Can serialize itself to disk.
///
#[derive(Debug)]
pub struct FaaSDataStore {
    store: RwLock<HashMap<String, Arc<Box<FunctionCode>>>>,
    path: PathBuf,
    serialize_on_write: bool,
}

impl FaaSDataStore {
    pub fn new<P: Into<PathBuf>>(path: P, serialize_on_write: bool) -> Self {
        FaaSDataStore::with(HashMap::new(), path, serialize_on_write)
    }

    pub fn with<P: Into<PathBuf>>(
        map: HashMap<String, Arc<Box<FunctionCode>>>,
        path: P,
        serialize_on_write: bool,
    ) -> Self {
        FaaSDataStore {
            store: RwLock::new(map),
            path: path.into(),
            serialize_on_write: serialize_on_write,
        }
    }

    pub fn set(&self, key: String, value: FunctionCode) {
        let val = Arc::new(Box::new(value));
        self.store.write().map(|mut writer| writer.insert(key, val));
        if self.serialize_on_write {
            match self.write_to_disk() {
                Ok(_) => {}
                Err(e) => {}
            }
        }
    }

    pub fn delete(&self, key: &str) {
        self.store.write().map(|mut writer| writer.remove(key));
        if self.serialize_on_write {
            match self.write_to_disk() {
                Ok(_) => {}
                Err(e) => {}
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<Arc<Box<FunctionCode>>> {
        self.store
            .read()
            .ok()
            .and_then(|reader| reader.get(key).cloned())
    }

    pub fn values(&self) -> Vec<Arc<Box<FunctionCode>>> {
        self.store
            .read()
            .ok()
            .map(|reader| reader.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn keys(&self) -> Vec<String> {
        self.store
            .read()
            .ok()
            .map(|reader| reader.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn items(&self) -> Vec<(String, Arc<Box<FunctionCode>>)> {
        self.store
            .read()
            .map(|reader| reader.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default()
    }

    fn serialize(&self, mut writer: impl Write) -> std::io::Result<()> {
        let db = &*(self.store.read().unwrap());
        let buf = rmp_serde::to_vec(db).expect("Couldn't serialize data store");
        writer.write_all(&buf)
    }

    pub fn write_to_disk(&self) -> std::io::Result<()> {
        let mut file = File::create(&self.path)?;
        self.serialize(&mut file)
    }

    pub fn from_path<P: Into<PathBuf>>(path: P) -> std::io::Result<Self> {
        let p = path.into();
        let file = File::open(&p).or_else(|_| File::create(&p))?;
        let buf_reader = BufReader::new(file);
        let store = rmp_serde::from_read(buf_reader).unwrap_or_default();
        Ok(FaaSDataStore::with(store, p, true))
    }
}
