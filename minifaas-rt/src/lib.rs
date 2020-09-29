mod ext;
pub mod languages;

use crate::ext::deno::Deno;
use crate::ext::deno::DenoSetup;
use crate::ext::toolchain::ActiveToolchain;
use crate::ext::toolchain::BuildToolchain;
use crate::languages::ToolchainMap;
use log::{debug, error, info, trace, warn};
use minifaas_common::*;
use std::sync::Arc;
use xactor::*;
mod actors;
use actors::*;
use futures::future::join_all;
/// Move this crate to actors:
/// Management actor that manages access to the function db?
/// Runtime actors: per each language one actor?
///

///
/// Configuration for the Function runtime
///
#[derive(Clone, Debug, Default)]
pub struct RuntimeConfiguration {
    num_threads: usize,
}

impl RuntimeConfiguration {
    ///
    /// New runtime config
    ///
    pub fn new(num_threads: usize) -> Self {
        RuntimeConfiguration { num_threads }
    }
}

#[derive(Clone)]
pub struct RuntimeConnection {
    controller_addr: Addr<RuntimeController>,
    http_addr: Addr<HttpTriggered>,
}

impl RuntimeConnection {
    pub async fn send(&self, req: RuntimeRequest) -> Result<RuntimeResponse> {
        match req {
            RuntimeRequest::Shutdown => self
                .controller_addr
                .call(OpsMsg::Shutdown)
                .await
                .map(|_| RuntimeResponse::Ok),
            RuntimeRequest::FunctionCall(_, inputs) => match inputs {
                FunctionInputs::Http(inp) => {
                    self.http_addr.call(inp).await?.map(RuntimeResponse::from)
                }
            },
            RuntimeRequest::NewFunction(code) => {
                let _ = self
                    .controller_addr
                    .call(SetupMsg {
                        env_id: code.environment_id,
                        toolchain: code.language(),
                    })
                    .await?;

                self.controller_addr
                    .call(StartExecutorMsg { code })
                    .await
                    .map(|_| RuntimeResponse::Ok)
            }

            _ => Ok(RuntimeResponse::FunctionRuntimeUnavailable(
                ProgrammingLanguage::JavaScript,
            )),
        }
    }
}

///
/// Creates a runtime based on the configuration and returns a command channel to invoke things with.
///
pub async fn create_runtime(
    config: RuntimeConfiguration,
    predefined_envs: Environments,
    deployments: Arc<FaaSDataStore>,
) -> Result<RuntimeConnection> {
    let setup_map = ToolchainMap::new(
        vec![
            (
                ProgrammingLanguage::JavaScript,
                BuildToolchain::Deno(DenoSetup::default()),
            ),
            (
                ProgrammingLanguage::Unknown,
                BuildToolchain::Deno(DenoSetup::default()),
            ),
        ],
        vec![
            (
                ProgrammingLanguage::JavaScript,
                Arc::new(ActiveToolchain::Deno(Deno::default())),
            ),
            (
                ProgrammingLanguage::Unknown,
                Arc::new(ActiveToolchain::Deno(Deno::default())),
            ),
        ],
    );

    info!(
        "Found {} executors & {} setups",
        setup_map.len_toolchain_executors(),
        setup_map.len_toolchain_setups()
    );

    let _http = Supervisor::start(HttpTriggered::new).await?;
    let _http2 = _http.clone();
    let _timer = Supervisor::start(TimerTriggered::new).await?;

    let _env_setup = Supervisor::start(move || {
        RuntimeController::new(
            predefined_envs.clone(),
            setup_map.clone(),
            _http2.clone(),
            _timer.clone(),
        )
    })
    .await?;

    info!("Runtime controller successfully started");
    let setup: Vec<Result<_>> = join_all(deployments.values().await.iter().map(|v| {
        _env_setup.call(SetupMsg {
            env_id: v.environment_id,
            toolchain: v.language(),
        })
    }))
    .await;
    let setup_ok: Vec<_> = setup.iter().filter(|f| f.is_ok()).collect();

    info!(
        "Success setting up {}/{} environments",
        setup_ok.len(),
        setup.len()
    );

    let started: Vec<Result<_>> = join_all(
        deployments
            .values()
            .await
            .iter()
            .map(|v| _env_setup.call(StartExecutorMsg { code: v.clone() })),
    )
    .await;

    let started_ok: Vec<_> = started.iter().filter(|f| f.is_ok()).collect();
    info!(
        "Success starting {}/{} executors",
        started_ok.len(),
        started.len()
    );

    Ok(RuntimeConnection {
        controller_addr: _env_setup,
        http_addr: _http.clone(),
    })
}
