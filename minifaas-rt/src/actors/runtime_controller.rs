use crate::ext::toolchain::BuildToolchain;
use crate::ext::toolchain::ToolchainSetup;
use crate::logs::collectors::{FileLogCollector, LogCollector};
use crate::{
    DestroyMsg, FunctionExecutor, HttpTriggerMsg, HttpTriggered, LogsMsg, OpsMsg, SetupMsg,
    StartExecutorMsg, StopExecutorMsg, TimerTriggered, ToolchainMap, Trigger,
};
use anyhow::Result;
use async_std::prelude::*;
use cron::Schedule;
use log::{debug, error, info};
use minifaas_common::Environments;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use xactor::*;

use super::IntervalTriggerMsg;

pub struct RuntimeController {
    environments: Environments,
    setup_map: ToolchainMap<BuildToolchain>,
    executors: HashMap<Uuid, Addr<FunctionExecutor>>,
    http_trigger: Addr<HttpTriggered>,
    timer_trigger: Addr<TimerTriggered>,
    log_collector: Arc<FileLogCollector>,
}

impl RuntimeController {
    pub fn new(
        existing_environments: Environments,
        toolchains: ToolchainMap<BuildToolchain>,
        http_trigger: Addr<HttpTriggered>,
        timer_trigger: Addr<TimerTriggered>,
        log_collector: Arc<FileLogCollector>,
    ) -> Self {
        RuntimeController {
            environments: existing_environments,
            setup_map: toolchains,
            executors: HashMap::default(),
            timer_trigger,
            http_trigger,
            log_collector,
        }
    }

    async fn subscribe_to_triggers(
        &self,
        msg: &StartExecutorMsg,
        addr: Addr<FunctionExecutor>,
        trigger: Trigger,
    ) -> Result<()> {
        match trigger {
            Trigger::Http(method) => {
                let sub = HttpTriggerMsg::Subscribe {
                    route: msg.code.name().clone(),
                    addr: addr,
                    method,
                };
                self.http_trigger.call(sub).await?;
                Ok(())
            }
            Trigger::Interval(cron_str) => {
                let schedule = cron_str
                    .parse::<Schedule>()
                    .map_err(|e| anyhow::Error::msg(e.to_string()))?;
                let sub = IntervalTriggerMsg::Subscribe {
                    schedule,
                    addr: addr,
                };
                self.timer_trigger.call(sub).await?;
                Ok(())
            }
            Trigger::None => Ok(()),
        }
    }

    async fn unsubscribe_from_triggers(
        &self,
        name: &String,
        _addr: Addr<FunctionExecutor>,
        trigger: Trigger,
    ) -> Result<()> {
        match trigger {
            Trigger::Http(_) => {
                let sub = HttpTriggerMsg::Unsubscribe {
                    route: name.clone(),
                };
                self.http_trigger.call(sub).await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[async_trait::async_trait]
impl Actor for RuntimeController {}

#[async_trait::async_trait]
impl Handler<LogsMsg> for RuntimeController {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: LogsMsg) -> Result<String> {
        match self.environments.get_or_create(msg.env_id).await {
            Ok(env) => {
                if env.has_file(&self.log_collector.file_name).await {
                    info!("Fetching logs for environment '{}'", msg.env_id);
                    let logs: Vec<String> = self
                        .log_collector
                        .reader(&env)
                        .await?
                        .lines()
                        .skip(msg.start_line)
                        .take(msg.lines)
                        .filter_map(Result::ok)
                        .collect()
                        .await;
                    Ok(logs.join("\n"))
                } else {
                    info!("No logs in environment '{}'", msg.env_id);
                    Ok("".to_owned())
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait::async_trait]
impl Handler<SetupMsg> for RuntimeController {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: SetupMsg) -> Result<()> {
        match self.environments.get_or_create(msg.env_id).await {
            Ok(env) => {
                info!("Found an environment for '{}'", msg.env_id);
                match self.setup_map.select_for_mut(&msg.toolchain) {
                    Some(toolchain) => {
                        toolchain.pre_setup(env).await?;
                        toolchain.setup(env).await?;
                        toolchain.post_setup(env).await?;
                        info!("Setup complete for '{}'", msg.env_id);
                        Ok(())
                    }
                    _ => {
                        let msg =
                            format!("Setup failed: no toolchain found for '{}'", msg.toolchain);
                        error!("Couldn't run setup: {}", &msg);
                        Err(anyhow::Error::msg(msg))
                    }
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait::async_trait]
impl Handler<DestroyMsg> for RuntimeController {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: DestroyMsg) -> Result<()> {
        match self.environments.remove(&msg.env_id).await {
            Some(_) => {
                debug!("Environment '{}' successfully deleted.", msg.env_id);
                Ok(())
            }
            None => {
                error!("Couldn't delete Environment '{}'.", msg.env_id);
                Err(anyhow::Error::msg(format!(
                    "Deleting environment '{}' failed",
                    msg.env_id
                )))
            }
        }
    }
}

#[async_trait::async_trait]
impl Handler<StartExecutorMsg> for RuntimeController {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: StartExecutorMsg) -> Result<()> {
        let env_id = msg.code.environment_id;
        debug!("Starting/replacing executors for env '{}'", env_id);
        match self.environments.get(&env_id).await {
            Some(env) => {
                if let Some(toolchain) = self.setup_map.select_executor(&msg.code.language()) {
                    let a = FunctionExecutor::new(
                        env.clone(),
                        msg.code.clone(),
                        toolchain.clone(),
                        self.log_collector.clone(),
                    )
                    .start()
                    .await?;
                    if let Some(existing) = self.executors.get(&env_id) {
                        // ignore result since it may be an error due to the executor already being shut down
                        let _ = existing.call(OpsMsg::Shutdown).await.map_err(|e| {
                            error!(
                                "While shutting down the executor for '{}': {:?}. Ignoring.",
                                env_id, e
                            );
                        });
                        self.unsubscribe_from_triggers(
                            &msg.code.name(),
                            existing.clone(),
                            msg.code.trigger().clone(),
                        )
                        .await?;
                    }
                    self.executors.insert(env_id, a.clone());
                    self.subscribe_to_triggers(&msg, a.clone(), msg.code.trigger().clone())
                        .await
                } else {
                    Err(anyhow::Error::msg(format!(
                        "Execute failed: no toolchain found for '{}'",
                        msg.code.language()
                    )))
                }
            }
            _ => Err(anyhow::Error::msg(format!(
                "Execute failed: no environment found for '{}'",
                env_id
            ))),
        }
    }
}

#[async_trait::async_trait]
impl Handler<StopExecutorMsg> for RuntimeController {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: StopExecutorMsg) -> Result<()> {
        let env_id = msg.code.environment_id;
        match self.environments.get(&env_id).await {
            Some(_env) => {
                if let Some(existing) = self.executors.get(&env_id) {
                    existing.call(OpsMsg::Shutdown).await?;
                    self.unsubscribe_from_triggers(
                        &msg.code.name(),
                        existing.clone(),
                        msg.code.trigger().clone(),
                    )
                    .await
                } else {
                    Ok(())
                }
            }
            _ => Err(anyhow::Error::msg(format!(
                "Stop failed: no environment found for '{}'",
                env_id
            ))),
        }
    }
}

#[async_trait::async_trait]
impl Handler<OpsMsg> for RuntimeController {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: OpsMsg) {
        match msg {
            OpsMsg::Shutdown => _ctx.stop(None),
        }
    }
}
