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

pub type UserFunctionType = Arc<Box<UserFunctionRecord>>;
type InnerStorageType = HashMap<String, UserFunctionType>;

///
/// A DB record to store a user function and the corresponding trigger/env id.
///
///
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct UserFunctionRecord {
    func: UserFunctionDeclaration,
    pub environment_id: Uuid,
}

impl UserFunctionRecord {
    pub fn new(func: UserFunctionDeclaration, env_id: Uuid) -> Self {
        UserFunctionRecord {
            func,
            environment_id: env_id,
        }
    }

    pub fn language(&self) -> ProgrammingLanguage {
        self.func.code.language
    }

    pub fn code(&self) -> &FunctionCode {
        &self.func.code
    }

    pub fn name(&self) -> &String {
        &self.func.name
    }

    pub fn trigger(&self) -> Trigger {
        self.func.trigger
    }

    pub fn update_function(
        &mut self,
        new_func: UserFunctionDeclaration,
    ) -> UserFunctionDeclaration {
        std::mem::replace(&mut self.func, new_func)
    }
}

impl fmt::Display for UserFunctionRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Function(Name: {}, Language: {}, Trigger: {}, Environment: {})",
            self.name(),
            self.language(),
            self.trigger(),
            self.environment_id
        )
    }
}

impl From<UserFunctionDeclaration> for UserFunctionRecord {
    fn from(f: UserFunctionDeclaration) -> Self {
        UserFunctionRecord::new(f, Uuid::new_v4())
    }
}

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

///
/// A key-value store for the user-defined functions. Uses an RwLock for multi-threaded reads/writes. Can serialize itself to disk.
///
#[derive(Debug)]
pub struct FaaSDataStore {
    store: RwLock<InnerStorageType>,
    path: PathBuf,
    serialize_on_write: bool,
}

impl FaaSDataStore {
    pub fn new<P: Into<PathBuf>>(path: P, serialize_on_write: bool) -> Self {
        FaaSDataStore::with(HashMap::new(), path, serialize_on_write)
    }

    pub fn with<P: Into<PathBuf>>(
        map: InnerStorageType,
        path: P,
        serialize_on_write: bool,
    ) -> Self {
        FaaSDataStore {
            store: RwLock::new(map),
            path: path.into(),
            serialize_on_write,
        }
    }

    ///
    /// Insert an entry.
    ///
    pub async fn set(&self, key: String, value: UserFunctionRecord) {
        let val = Arc::new(Box::new(value));
        self.store.write().await.insert(key, val);
        if self.serialize_on_write {
            match self.write_to_disk().await {
                Ok(_) => {}
                Err(e) => error!("Couldn't serialze to disk: {}", e),
            }
        }
    }

    ///
    /// Removes an entry without returning the result.
    ///
    pub async fn delete(&self, key: &str) {
        let _old = self.store.write().await.remove(key);
        if self.serialize_on_write {
            match self.write_to_disk().await {
                Ok(_) => {}
                Err(e) => error!("Couldn't serialze to disk: {}", e),
            }
        }
    }

    ///
    /// Return a record based on the key.
    ///
    pub async fn get(&self, key: &str) -> Option<UserFunctionType> {
        self.store.read().await.get(key).cloned()
    }

    ///
    /// Creates a list of all stored records
    ///
    pub async fn values(&self) -> Vec<UserFunctionType> {
        self.store.read().await.values().cloned().collect()
    }

    ///
    /// Creates a list of all keys.
    ///
    pub async fn keys(&self) -> Vec<String> {
        self.store.read().await.keys().cloned().collect()
    }

    ///
    /// Creates a list of tuples (key, record)
    ///
    pub async fn items(&self) -> Vec<(String, UserFunctionType)> {
        self.store
            .read()
            .await
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    ///
    /// The number of items in the store
    ///
    pub async fn len(&self) -> usize {
        self.store.read().await.len()
    }

    ///
    /// Shortcut to whether the store is empty.
    ///
    pub async fn is_empty(&self) -> bool {
        self.store.read().await.is_empty()
    }

    ///
    /// Shortcut to whether the store is empty.
    ///
    pub async fn contains_key(&self, key: &str) -> bool {
        self.store.read().await.contains_key(key)
    }

    ///
    /// Serializes the store using the provided (async_std) writer
    ///
    pub async fn serialize(&self, mut writer: impl Write + Unpin) -> Result<()> {
        let db = (self.store.read().await).clone();
        println!("{}", serde_json::to_string_pretty(&db)?);
        let buf = task::spawn_blocking(move || {
            serde_json::to_string(&db)
                .expect("Couldn't serialize data store")
                .into_bytes()
        })
        .await;
        writer.write_all(&buf).await.map_err(|e| e.into())
    }

    ///
    /// Writes the store to disk at the (initially) provided location
    ///
    pub async fn write_to_disk(&self) -> Result<()> {
        let mut file = File::create(&self.path).await?;
        self.serialize(&mut file).await
    }

    ///
    /// Loads a store from the provided path.
    ///
    pub async fn from_path<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let p = path.into();
        info!(
            "Reading functions from store at '{}'",
            p.to_str().unwrap_or_default()
        );

        let _p = p.clone();
        let store: InnerStorageType = task::spawn_blocking(move || {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&_p)
                .expect("Could not open/create data store");
            let buf_reader = BufReader::new(file);
            serde_json::from_reader(buf_reader).unwrap_or_default()
        })
        .await;
        Ok(FaaSDataStore::with(store, p, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use minifaas_test::get_empty_tmp_dir;
    /*this somehow couldn't be serialized:

        {
      "asdf": {
        "func": {
          "name": "asdf",
          "code": "console.log(\"hello\");",
          "language": {
            "lang": "JavaScript"
          },
          "trigger": {
            "type": "Http",
            "when": "GET"
          }
        },
        "environment_id": "0cd84ce3-4296-4793-97e4-f354a5253b2c"
      }
    }*/

    #[async_std::test]
    async fn test_from_path_missing_paths_create_new() {
        let p = get_empty_tmp_dir();
        let store = FaaSDataStore::from_path(p.join("doesntexist")).await;
        assert!(store.is_ok());
        assert_eq!(store.unwrap().len().await, 0);
    }

    #[async_std::test]
    #[should_panic]
    async fn test_from_path_invalid_paths_panic() {
        let p = get_empty_tmp_dir();
        let _ = FaaSDataStore::from_path(p.join("hello").join("world")).await;
        let _ = std::fs::remove_dir_all(p);
    }

    #[async_std::test]
    #[should_panic]
    async fn test_from_path_invalid_files_panic() {
        let p = get_empty_tmp_dir();
        let f_name = "a-file";
        async_std::fs::write(p.join(f_name), b"invalidcontent")
            .await
            .unwrap();
        assert!(FaaSDataStore::from_path(p.join(f_name)).await.is_err());
    }
}
