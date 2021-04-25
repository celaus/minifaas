mod ext;
pub mod languages;
mod logs;
mod output_parser;

use crate::ext::bash::Bash;
use crate::ext::bash::BashSetup;
use crate::ext::deno::Deno;
use crate::ext::deno::DenoSetup;
use crate::ext::toolchain::ActiveToolchain;
use crate::ext::toolchain::BuildToolchain;
use crate::languages::ToolchainMap;
use crate::logs::collectors::FileLogCollector;
use log::{debug, error, info, trace, warn};
use minifaas_common::*;
use std::sync::Arc;
use xactor::*;
mod actors;
use actors::*;
use chrono::Duration;
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
    timer_resolution_ms: i64,
}

impl RuntimeConfiguration {
    ///
    /// New runtime config
    ///
    pub fn new(num_threads: usize, timer_resolution_ms: i64) -> Self {
        RuntimeConfiguration {
            num_threads,
            timer_resolution_ms,
        }
    }
}

#[derive(Clone)]
pub struct RuntimeConnection {
    controller_addr: Addr<RuntimeController>,
    http_addr: Addr<HttpTriggered>,
    timer_addr: Addr<TimerTriggered>,
}

impl RuntimeConnection {
    ///
    ///
    ///
    pub async fn send(&self, req: RuntimeRequest) -> Result<RuntimeResponse> {
        match req {
            RuntimeRequest::Shutdown => self
                .controller_addr
                .call(OpsMsg::Shutdown)
                .await
                .map(|_| RuntimeResponse::Ok),

            RuntimeRequest::FetchLogs {
                env_id,
                start_line,
                lines,
            } => self
                .controller_addr
                .call(LogsMsg {
                    env_id,
                    start_line,
                    lines,
                })
                .await?
                .map(|s| RuntimeResponse::LogResponse(s)),
            RuntimeRequest::FunctionCall(_, inputs) => match inputs {
                FunctionInputs::Http(inp) => {
                    self.http_addr.call(inp).await?.map(RuntimeResponse::from)
                }
                FunctionInputs::Timer(_) => Err(Error::msg("Cannot call timers explicitly")),
            },
            RuntimeRequest::NewFunction(code) => {
                debug!("New function request received. {:?}", code);
                let _ = self
                    .controller_addr
                    .call(SetupMsg {
                        env_id: code.environment_id,
                        toolchain: *code.language(),
                    })
                    .await?;
                debug!("Setup completed for {:?}", code);
                self.controller_addr
                    .call(StartExecutorMsg { code })
                    .await
                    .map(|_| {
                        debug!("Started executors.");
                        RuntimeResponse::Ok
                    })
            }
            RuntimeRequest::DeleteFunction(code) => {
                let _ = self
                    .controller_addr
                    .call(SetupMsg {
                        env_id: code.environment_id,
                        toolchain: *code.language(),
                    })
                    .await?;

                self.controller_addr
                    .call(StartExecutorMsg { code })
                    .await
                    .map(|_| RuntimeResponse::Ok)
            }
            RuntimeRequest::Disable(code) => self
                .controller_addr
                .call(StopExecutorMsg { code })
                .await
                .map(|_| RuntimeResponse::Ok),
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
                ProgrammingLanguage::Bash,
                BuildToolchain::Bash(BashSetup::default()),
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
                ProgrammingLanguage::Bash,
                Arc::new(ActiveToolchain::Bash(Bash::default())),
            ),
            (
                ProgrammingLanguage::Unknown,
                Arc::new(ActiveToolchain::Deno(Deno::default())),
            ),
        ],
    );

    let timer_resolution = Duration::milliseconds(config.timer_resolution_ms).to_std()?;

    info!(
        "Found {} executors & {} setups",
        setup_map.len_toolchain_executors(),
        setup_map.len_toolchain_setups()
    );

    let _http = Supervisor::start(HttpTriggered::new).await?;
    let _http2 = _http.clone();
    let _timer = Supervisor::start(move || TimerTriggered::new(timer_resolution)).await?;
    let _timer2 = _timer.clone();
    let log_collector = Arc::new(FileLogCollector::new("logs"));

    let _env_setup = Supervisor::start(move || {
        RuntimeController::new(
            predefined_envs.clone(),
            setup_map.clone(),
            _http2.clone(),
            _timer2.clone(),
            log_collector.clone(),
        )
    })
    .await?;

    info!("Runtime controller successfully started");
    let setup: Vec<Result<_>> = join_all(deployments.values().await.iter().map(|v| {
        _env_setup.call(SetupMsg {
            env_id: v.environment_id,
            toolchain: *v.language(),
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
        timer_addr: _timer.clone(),
    })
}
