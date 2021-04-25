use crate::errors::PreparationError::EnvironmentAddFailed;
use anyhow::Result;

use super::Environment;
use log::debug;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct Environments {
    pub envs: HashMap<Uuid, Environment>,
    root: PathBuf,
}

impl Environments {
    pub fn new<P: Into<PathBuf>>(root_path: P, environments: HashMap<Uuid, Environment>) -> Self {
        Environments {
            root: root_path.into(),
            envs: environments,
        }
    }

    pub fn from_vec<P: Into<PathBuf>>(root_path: P, environments: Vec<Environment>) -> Self {
        let inner: HashMap<_, _> = environments.into_iter().map(|e| (e.id, e)).collect();
        Environments {
            root: root_path.into(),
            envs: inner,
        }
    }

    ///
    /// Finds or creates an environment with the provided GUID. 
    ///
    pub async fn get_or_create(&mut self, environment_id: Uuid) -> Result<&Environment> {
        debug!("Environment with id '{}' requested", environment_id);
        if !self.envs.contains_key(&environment_id) {
            debug!("Creating '{}'", environment_id);
            let new = Environment::create_with_id(
                &self.root.join(&environment_id.to_string()),
                environment_id,
            )
            .await?;
            self.envs.insert(environment_id, new);
        };
        self.envs
            .get(&environment_id)
            .ok_or_else(|| EnvironmentAddFailed(self.root.to_str().unwrap().to_string()).into())
    }

    pub async fn get(&self, environment_id: &Uuid) -> Option<&Environment> {
        self.envs.get(environment_id)
    }

    pub async fn remove(&mut self, environment_id: &Uuid) -> Option<()> {
        let mut env = self.envs.remove(environment_id)?;
        env.delete().await.ok()?;
        Some(())
    }

    pub async fn count(&self) -> usize {
        self.envs.len()
    }
}

#[cfg(test)]
mod tests {}
