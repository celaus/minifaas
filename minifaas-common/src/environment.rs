use crate::errors::PreparationError::EnvironmentAddFailed;
use anyhow::{Error as AnyError, Result};
use async_std::fs::DirBuilder;
use async_std::fs::File;
use async_std::fs::{create_dir_all, read, write};
use async_std::task;
use log::info;
use std::collections::{HashMap, HashSet};
use std::fs::read_dir;
use std::iter::FromIterator;
use std::path::PathBuf;
use uuid::Uuid;
use std::fmt;

const ID_FILE_NAME: &str = ".minifaas-id";

#[derive(PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct Environment {
    root: String,
    pub id: Uuid,
}

impl Environment {
    ///
    ///
    ///
    pub async fn create_with_id<S: Into<PathBuf>>(root: S, env_id: Uuid) -> Result<Self> {
        let root = root.into();
        // create required environment artifacts (the directory and an id file)
        create_dir_all(&root).await?;
        let mut id_file_name = PathBuf::from(&root);
        id_file_name.push(ID_FILE_NAME);
        write(id_file_name, env_id.as_bytes()).await?;
        info!(
            "Added a new environment '{}' at '{}' ",
            env_id,
            root.to_str().unwrap()
        );
        Ok(Environment {
            root: String::from(
                root.to_str()
                    .ok_or_else(|| AnyError::msg("Could not convert env root to string"))?,
            ),
            id: env_id,
        })
    }

    pub async fn create<S: Into<PathBuf>>(root: S) -> Result<Self> {
        let env_id = Uuid::new_v4();
        Environment::create_with_id(root, env_id).await
    }

    // ///
    // /// Synchronizes a given directory by checking the id file
    // ///
    // pub async fn sync<S: Into<String>>(root: S, id: Uuid) -> io::Result<Self> {
    //     let root = root.into();

    //     match self.get_id_from(root).await {
    //         Some(existing_id) => Environment::create_with_id(root, id)
    //     }
    //     match read_dir(&root) {
    //         Err(_) => Environment::create(root).await,
    //         Ok(contents) => {
    //             if let Some(id_file) = contents
    //                 .filter(|r| r.is_ok())
    //                 .map(|e| e.unwrap().path())
    //                 .filter(|e| e.ends_with(ID_FILE_NAME))
    //                 .last()
    //             // if there are multiple files, take the last one
    //             {
    //                 match read(id_file).await {
    //                     Ok(b) => {
    //                         if let Ok(id) = Uuid::from_slice(&b) {
    //                             Ok(Environment { root, id })
    //                         } else {
    //                             Environment::create_with_id(root, id).await
    //                         }
    //                     }
    //                     Err(_) => Environment::create_with_id(root, id).await,
    //                 }
    //             } else {
    //                 Environment::create_with_id(root, id).await
    //             }
    //         }
    //     }
    // }

    ///
    /// Checks whether an environment contains the file specified
    ///
    pub async fn has_file<S: Into<PathBuf>>(&self, sub_path: S) -> bool {
        let p = PathBuf::from(&self.root);
        if let Ok(meta) = async_std::fs::metadata(p.join(sub_path.into())).await {
            meta.is_file()
        } else {
            false
        }
    }

    ///
    /// Checks whether an environment contains the file specified
    ///
    pub async fn has_dir<S: Into<PathBuf>>(&self, sub_path: S) -> bool {
        let p = PathBuf::from(&self.root);
        if let Ok(meta) = async_std::fs::metadata(p.join(sub_path.into())).await {
            meta.is_dir()
        } else {
            false
        }
    }

    async fn get_id_from(dir: PathBuf) -> Option<Uuid> {
        match read_dir(&dir) {
            Err(_) => None,
            Ok(contents) => {
                if let Some(id_file) = contents
                    .filter(|r| r.is_ok())
                    .map(|e| e.unwrap().path())
                    .filter(|e| e.ends_with(ID_FILE_NAME))
                    .last()
                {
                    match read(id_file).await {
                        Ok(b) => {
                            if let Ok(id) = Uuid::from_slice(&b) {
                                Some(id)
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
        }
    }

    ///
    /// Synchronizes a given directory by checking the id file
    ///
    pub async fn sync_all<S: Into<PathBuf>>(
        root: S,
        expected_ids: &[Uuid],
    ) -> anyhow::Result<Environments> {
        let root = root.into();

        let entries = read_dir(&root)?;
        let mut existing_envs: Vec<Environment> = entries
            .filter_map(|e| {
                if e.is_ok() && e.as_ref().unwrap().file_type().is_ok() {
                    e.ok()
                } else {
                    None
                }
            })
            .filter_map(|e| {
                if e.file_type().unwrap().is_dir() {
                    task::block_on(Self::get_id_from(e.path())).map(|i| (e, i))
                } else {
                    None
                }
            })
            .map(|(e, id)| task::block_on(Environment::create_with_id(e.path(), id)))
            .filter_map(Result::ok)
            .collect();

        let existing: HashSet<_> = HashSet::from_iter(existing_envs.iter().map(|e| e.id));
        let expected = HashSet::from_iter(expected_ids.iter().cloned());

        let mut remaining: Vec<Environment> = expected
            .difference(&existing)
            .map(|id| {
                task::block_on(Environment::create_with_id(
                    (&root).clone().join(id.to_string()),
                    *id,
                ))
                .unwrap()
            })
            .collect();
        remaining.append(&mut existing_envs);
        Ok(Environments::from_vec(root, remaining))
    }

    ///
    /// Adds a file to the environment.
    ///
    pub async fn add_file_async<S: Into<PathBuf>>(&self, sub_path: S) -> Result<File> {
        let mut p = PathBuf::from(&self.root);
        p.push(sub_path.into());
        Ok(File::create(p.to_str().unwrap()).await?)
    }

    ///
    /// Adds a file to the environment.
    ///
    pub fn add_file<S: Into<PathBuf>>(&self, sub_path: S) -> Result<std::fs::File> {
        let mut p = PathBuf::from(&self.root);
        p.push(sub_path.into());
        Ok(std::fs::File::create(p.to_str().unwrap())?)
    }

    ///
    /// Adds a directory to the environment
    ///
    pub async fn add_dir<S: Into<PathBuf>>(&self, sub_path: S) -> Result<()> {
        let mut p = PathBuf::from(&self.root);
        p.push(sub_path.into());
        Ok(DirBuilder::new()
            .recursive(true)
            .create(p.to_str().unwrap())
            .await?)
    }

    pub async fn relative_path<S: Into<PathBuf>>(&self, sub_path: S) -> PathBuf {
        PathBuf::from(&self.root).join(sub_path.into())
    }
}


impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Environment {} at {}", self.id, self.root)
    }
}

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
    ///
    ///
    pub async fn get_or_create(&mut self, environment_id: Uuid) -> Result<&Environment> {
        info!("Environment with id '{}' requested", environment_id);
        if !self.envs.contains_key(&environment_id) {
            info!("Creating '{}'", environment_id);
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

    pub async fn count(&self) -> usize {
        self.envs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use async_std::fs::read;
    use minifaas_test::get_empty_tmp_dir;

    #[async_std::test]
    async fn test_env_add_file_readable() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let f_name = "hello";
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        assert!(e.add_file(f_name).is_ok());
        assert_eq!(read(env_path.join(f_name)).await.unwrap(), Vec::<u8>::new()); // the file is empty
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }

    #[async_std::test]
    async fn test_env_add_file_duplicate() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let f_name = "hello";
        let env_path = root_dir.join(Uuid::new_v4().to_string());

        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        assert!(e.add_file(f_name).is_ok());
        assert!(e.add_file(f_name).is_ok());
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }

    #[async_std::test]
    async fn test_env_add_file_async_readable() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let f_name = "hello";
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        assert!(e.add_file_async(f_name).await.is_ok());
        assert_eq!(read(env_path.join(f_name)).await.unwrap(), Vec::<u8>::new()); // the file is empty
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }

    #[async_std::test]
    async fn test_env_add_file_async_duplicate() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let f_name = "hello";
        let env_path = root_dir.join(Uuid::new_v4().to_string());

        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        assert!(e.add_file_async(f_name).await.is_ok());
        assert!(e.add_file_async(f_name).await.is_ok());
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }

    #[async_std::test]
    async fn test_env_add_dir_readable() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let f_name = "hello";
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        assert!(e.add_dir(f_name).await.is_ok());

        let metadata = async_std::fs::metadata(env_path.join(f_name))
            .await
            .unwrap();

        assert!(metadata.file_type().is_dir());
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }

    #[async_std::test]
    async fn test_env_add_dir_duplicate() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let f_name = "hello";
        let env_path = root_dir.join(Uuid::new_v4().to_string());

        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        assert!(e.add_dir(f_name).await.is_ok());
        assert!(e.add_dir(f_name).await.is_ok());
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }

    #[async_std::test]
    async fn test_env_has_dir_valid() {
        let f_name = "hello";

        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();
        async_std::fs::create_dir(env_path.join(f_name))
            .await
            .unwrap();
        assert!(e.has_dir(f_name).await);
        assert!(!e.has_dir("other_dir").await);
    }

    #[async_std::test]
    async fn test_env_has_dir_no_files() {
        let f_name = "a-file";

        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();
        async_std::fs::write(env_path.join(f_name), b"content")
            .await
            .unwrap();
        assert!(!e.has_dir(f_name).await);
    }
}
