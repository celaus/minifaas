mod datastore;
mod environment;
pub mod errors;
pub mod runtime;
pub mod triggers;
mod types;

pub use crate::types::*;
use anyhow::Result;
use async_std::path::PathBuf;
pub use datastore::{DataStoreConfig, FaaSDataStore, UserFunctionRecord, UserFunctionType};
pub use environment::{Environment, Environments};
use log::info;
use std::sync::Arc;
pub use triggers::Trigger;
use uuid::Uuid;

pub use runtime::{FunctionInputs, FunctionOutputs, RuntimeRequest, RuntimeResponse};
///
/// Creates and prepares the file system
///
pub async fn create_or_load_storage(config: DataStoreConfig) -> Result<FaaSDataStore> {
    let store = FaaSDataStore::from_path(&config.path).await?;
    info!("Read {} functions from store", store.len().await);
    Ok(store)
}

///
/// Sets up the enviornment directories based on the IDs contained in the datastore.
///
pub async fn sync_environments<P: Into<PathBuf>>(
    root: P,
    datastore: Arc<FaaSDataStore>,
) -> Result<Environments> {
    let ids = datastore
        .items()
        .await
        .iter()
        .map(|(_k, f)| (*f.language(), f.environment_id))
        .collect::<Vec<(ProgrammingLanguage, Uuid)>>();
    let expected_env_ids: Vec<_> = ids.iter().map(|i| i.1).collect();
    Environment::sync_all(root.into(), &expected_env_ids).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triggers::http::HttpMethod;
    use crate::{datastore::UserFunctionRecord, runtime::FunctionCode};
    use minifaas_test::get_empty_tmp_dir;

    #[async_std::test]
    async fn test_sync_environments_environment_ids() {
        let root_dir = get_empty_tmp_dir();

        let declaration = UserFunctionDeclaration {
            code: FunctionCode::new(
                "no-code-necessary".to_string(),
                ProgrammingLanguage::Unknown,
            ),
            trigger: Trigger::Http(HttpMethod::ALL),
            name: "a-name".to_string(),
        };
        let record = UserFunctionRecord::from(declaration);

        let expected_id = record.environment_id;

        let datastore = FaaSDataStore::new(root_dir.join("testing.db"), false);
        datastore.set("a".to_string(), record).await;

        let envs = sync_environments(&root_dir, Arc::new(datastore))
            .await
            .unwrap();
        assert_eq!(envs.count().await, 1);
        assert_eq!(envs.get(&expected_id).await.unwrap().id, expected_id);
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }
}
