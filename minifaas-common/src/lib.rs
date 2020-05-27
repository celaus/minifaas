mod datastore;
pub mod errors;
mod runtime;
mod types;

use log::{error, debug, info, trace, warn};


pub use crate::types::*;
pub use datastore::{DataStoreConfig, FaaSDataStore};

pub use runtime::{FunctionInputs, FunctionOutputs, RuntimeRequest, RuntimeResponse};

pub fn create_or_load_storage(config: DataStoreConfig) -> std::io::Result<FaaSDataStore> {
    let store = FaaSDataStore::from_path(&config.path)?;
    info!("Read {} functions from store", store.len());
    Ok(store)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
