use crate::ext::toolchain::ToolchainLifecycle;
use crate::ext::toolchain::ToolchainSetup;
use crate::Environment;
use anyhow::Result;
use async_std::task;
use log::{debug, error, info, warn};
use minifaas_common::runtime::RawFunctionInput;
use std::env;
use std::io;
use std::io::Read;
use std::io::Write;
use std::process::{Command, Stdio};

pub const DEFAULT_VERSION: &str = "3.2.57";
const DEFAULT_BASH_EXE_NAME: &str = "bash";

#[derive(Clone, Debug)]
pub struct Bash {
    local_path: String,
    default_args: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct BashSetup {
    local_path: String,
    system: os_info::Info,
    version: String,
    pub installed: bool,
}

impl BashSetup {
    pub fn new<S: Into<String>>(bash_name: S, system: os_info::Info, version: S) -> Self {
        BashSetup {
            local_path: bash_name.into(),
            system,
            version: version.into(),
            installed: false,
        }
    }

    pub fn with_version<S: Into<String>>(version: S) -> Self {
        BashSetup::new(DEFAULT_BASH_EXE_NAME, os_info::get(), &version.into())
    }
}

impl Default for BashSetup {
    fn default() -> Self {
        BashSetup::with_version(DEFAULT_VERSION)
    }
}

impl Bash {
    pub fn new(default_args: Vec<String>) -> Self {
        Bash {
            local_path: DEFAULT_BASH_EXE_NAME.to_string(),
            default_args,
        }
    }
}

impl Default for Bash {
    fn default() -> Self {
        Bash {
            local_path: DEFAULT_BASH_EXE_NAME.into(),
            default_args: vec![],
        }
    }
}

#[async_trait::async_trait]
impl ToolchainLifecycle for Bash {
    async fn pre_build(&self) -> Result<()> {
        Ok(())
    }

    async fn _build(&self, code: &str) -> Result<Vec<u8>> {
        Ok(code.to_owned().into_bytes())
    }

    async fn post_build(&self) -> Result<()> {
        Ok(())
    }

    async fn pre_execute(&self, _input: &RawFunctionInput) -> Result<()> {
        Ok(())
    }

    async fn _execute(
        &self,
        code: Vec<u8>,
        input: &RawFunctionInput,
        env: &Environment,
    ) -> Result<String> {
        let exe = self.local_path.clone(); // bash should be in everyone's path on Linux

        let code = code.clone();
        info!("CODE ({}): {}",code.len(), std::str::from_utf8(&code)?);
        let default_args = self.default_args.clone();
        info!("Starting execution with {}", exe);
        task::spawn_blocking(move || {
            let mut child = Command::new(&*exe)
                .args(default_args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed to execute child");

            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin.write_all(&code).expect("Failed to write to stdin");
            let output = child.wait_with_output().expect("failed to wait on child");
            if output.status.success() {
                std::str::from_utf8(&output.stdout)
                    .map(|s| s.to_owned())
                    .map_err(|e| e.into())
            } else {
                Err(std::io::Error::from(std::io::ErrorKind::NotFound).into())
            }
        })
        .await
    }

    async fn post_execute(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl ToolchainSetup for BashSetup {
    async fn pre_setup(&mut self, _env: &Environment) -> Result<()> {
        self.installed = Command::new(&*self.local_path)
            .arg("--version")
            .spawn()
            .is_ok();
        info!("Is Bash available? {}", self.installed);
        Ok(())
    }

    async fn setup(&self, env: &Environment) -> Result<()> {
        if self.installed {
            Ok(())
        } else {
            info!("Could not find Bash in Env: {}", env);
            Err(anyhow::Error::msg("No Bash executable available"))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use minifaas_common::Environment;
    use minifaas_test::get_empty_tmp_dir;
    use uuid::Uuid;

    #[async_std::test]
    #[ignore] // download takes some time
    async fn bashsetup_setup_download() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        let bash_setup = BashSetup::default();
        bash_setup.setup(&e).await.unwrap();
        assert!(e.has_file("Bash").await);
    }

    #[async_std::test]
    async fn bashsetup_presetup_download() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        let mut bash_setup = BashSetup::default();
        e.add_file("Bash").unwrap();
        bash_setup.pre_setup(&e).await.unwrap();
        assert!(bash_setup.installed);
    }
}
