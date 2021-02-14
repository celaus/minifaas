use crate::ext::toolchain::ToolchainLifecycle;
use crate::ext::toolchain::ToolchainSetup;
use crate::Environment;
use anyhow::Result;
use async_std::task;
use log::{debug, error, info, warn};
use minifaas_common::runtime::RawFunctionInput;
use std::io::Read;
use std::io::Write;
use std::process::{Command, Stdio};
use std::{io, sync::Arc};

/*
-A, --allow-all Allow all permissions. This disables all security.
--allow-env Allow environment access for things like getting and setting of environment variables.
--allow-hrtime Allow high-resolution time measurement. High-resolution time can be used in timing attacks and fingerprinting.
--allow-net=<allow-net> Allow network access. You can specify an optional, comma-separated list of domains to provide an allow-list of allowed domains.
--allow-plugin Allow loading plugins. Please note that --allow-plugin is an unstable feature.
--allow-read=<allow-read> Allow file system read access. You can specify an optional, comma-separated list of directories or files to provide a allow-list of allowed file system access.
--allow-run Allow running subprocesses. Be aware that subprocesses are not run in a sandbox and therefore do not have the same security restrictions as the deno process. Therefore, use with caution.
--allow-write=<allow-write> Allow file system write access. You can specify an optional, comma-separated list of directories or files to provide a allow-list of allowed file system access.
*/

pub const DEFAULT_VERSION: &str = "1.7.4";
const DEFAULT_DENO_EXE_NAME: &str = "deno";

#[derive(Clone, Debug)]
pub struct Deno {
    local_path: String,
    default_args: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct DenoSetup {
    local_path: String,
    system: os_info::Info,
    version: String,
    pub installed: bool,
}

impl DenoSetup {
    pub fn new<S: Into<String>>(deno_name: S, system: os_info::Info, version: S) -> Self {
        DenoSetup {
            local_path: deno_name.into(),
            system,
            version: version.into(),
            installed: false,
        }
    }

    pub fn with_version<S: Into<String>>(version: S) -> Self {
        DenoSetup::new(DEFAULT_DENO_EXE_NAME, os_info::get(), &version.into())
    }
}

impl Default for DenoSetup {
    fn default() -> Self {
        DenoSetup::with_version(DEFAULT_VERSION)
    }
}

impl Deno {
    pub fn new(default_args: Vec<String>) -> Self {
        Deno {
            local_path: DEFAULT_DENO_EXE_NAME.to_string(),
            default_args,
        }
    }
}

impl Default for Deno {
    fn default() -> Self {
        Deno {
            local_path: DEFAULT_DENO_EXE_NAME.into(),
            default_args: vec!["run".to_owned(), "-".to_owned()],
        }
    }
}

#[async_trait::async_trait]
impl ToolchainLifecycle for Deno {
    async fn pre_build(&self) -> Result<()> {
        Ok(())
    }

    async fn _build(&self, code: &str) -> Result<Vec<u8>> {
        Ok(code.to_owned().into_bytes())
    }

    async fn post_build(&self) -> Result<()> {
        Ok(())
    }

    async fn pre_execute(&self, _input: Arc<RawFunctionInput>) -> Result<()> {
        Ok(())
    }

    async fn _execute(
        &self,
        code: Vec<u8>,
        input: Arc<RawFunctionInput>,
        env: &Environment,
    ) -> Result<String> {
        let exe = env
            .absolute_path(&self.local_path)
            .await
            .into_os_string()
            .into_string()
            .expect("Invalid chars in path");

        let code = code.clone();
        debug!(
            "Executing on Deno: {} (length: {} chars)",
            std::str::from_utf8(&code)?,
            code.len()
        );
        let default_args = self.default_args.clone();
        debug!("Starting execution with {}", exe);
        task::spawn_blocking(move || {
            let mut child = Command::new(&*exe)
                .args(default_args)
                .env_clear()
                .env("__MF__INPUTS", serde_json::to_string(&input)?)
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
impl ToolchainSetup for DenoSetup {
    async fn pre_setup(&mut self, env: &Environment) -> Result<()> {
        self.installed = env.has_file(&self.local_path).await;
        debug!("Is Deno installed in {}? {}", env, self.installed);
        Ok(())
    }

    async fn setup(&self, env: &Environment) -> Result<()> {
        if self.installed {
            info!("Found Deno in {}, skipping setup", env);
            Ok(())
        } else {
            warn!("Could not find Deno in Env: {}, downloading", env);
            let origin = format!(
                "https://github.com/denoland/deno/releases/download/v{}/deno-{}.zip",
                self.version,
                os_arch_tuple(&self.system)
            );
            debug!("Downloading from '{}'", origin);
            let mut file = env.add_file(&self.local_path)?;

            task::spawn_blocking(move || {
                let resp = ureq::get(&origin).call();
                let mut r = resp.into_reader();
                let mut zipped = vec![];
                r.read_to_end(&mut zipped)?;
                let mut extracted = zip::ZipArchive::new(std::io::Cursor::new(zipped)).unwrap();
                let mut deno_exe = extracted.by_index(0).unwrap();
                if io::copy(&mut deno_exe, &mut file)? == deno_exe.size() {
                    Ok(())
                } else {
                    Err(io::Error::from(io::ErrorKind::NotFound).into())
                }
            })
            .await
        }
    }

    async fn post_setup(&self, env: &Environment) -> Result<()> {
        if cfg!(target_family = "unix") {
            use std::os::unix::fs::PermissionsExt;
            let path = env.absolute_path(&self.local_path).await;
            // permissions for this should be -rwxr-xr-x, or 755
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        Ok(())
    }
}

#[cfg(target_arch = "x86_64")]
const SYSTEM_ARCH: &str = "x86_64";
#[cfg(target_arch = "aarch64")]
const SYSTEM_ARCH: &str = "aarch64";
#[cfg(target_arch = "arm")]
const SYSTEM_ARCH: &str = "arm";

///
/// Guesses the OS string from the current operating system.
///
fn os_arch_tuple(info: &os_info::Info) -> String {
    let os = String::from(match info.os_type() {
        os_info::Type::Windows => "pc-windows-msvc",
        os_info::Type::Macos => "apple-darwin",
        _ => "unknown-linux-gnu",
    });
    format!("{}-{}", SYSTEM_ARCH, os)
}

#[cfg(test)]
mod tests {

    use super::*;

    use minifaas_common::Environment;
    use minifaas_test::get_empty_tmp_dir;
    use uuid::Uuid;

    async fn create_temp_env(expected_id: Option<Uuid>) -> Environment {
        let root_dir = get_empty_tmp_dir();
        let expected_id = expected_id.unwrap_or(Uuid::new_v4());
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap()
    }

    #[async_std::test]
    #[ignore] // download takes some time
    async fn denosetup_setup_download() {
        let e = create_temp_env(None).await;
        let deno_setup = DenoSetup::default();
        deno_setup.setup(&e).await.unwrap();
        assert!(e.has_file("deno").await);
    }

    #[async_std::test]
    async fn denosetup_presetup_download() {
        let e = create_temp_env(None).await;
        let mut deno_setup = DenoSetup::default();
        e.add_file("deno").unwrap();
        deno_setup.pre_setup(&e).await.unwrap();
        assert!(deno_setup.installed);
    }

    #[async_std::test]
    #[cfg(unix)]
    async fn denosetup_postsetup_set_execute_flag() {
        use std::os::unix::fs::PermissionsExt;
        let filename = "deno";
        let e = create_temp_env(None).await;

        e.add_file(filename).unwrap();
        DenoSetup::default().post_setup(&e).await.unwrap();
        let meta = std::fs::metadata(e.absolute_path(filename).await).unwrap();
        let perm = meta.permissions();
        assert_eq!(0o100755, perm.mode());
    }
}
