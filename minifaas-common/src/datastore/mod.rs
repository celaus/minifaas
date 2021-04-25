mod config;
mod json_file;
mod record;

pub use crate::types::*;
use async_std::sync::Arc;
pub use record::UserFunctionRecord;
use std::boxed::Box;
use std::collections::HashMap;
pub type UserFunctionType = Arc<Box<UserFunctionRecord>>;

pub use config::DataStoreConfig;
pub use json_file::JsonFaaSDataStore as FaaSDataStore;

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
