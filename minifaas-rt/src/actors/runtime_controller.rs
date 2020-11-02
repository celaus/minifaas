use crate::ext::toolchain::BuildToolchain;
use crate::ext::toolchain::ToolchainSetup;
use crate::StopExecutorMsg;
use crate::{
    FunctionExecutor, HttpTriggerMsg, HttpTriggered, OpsMsg, SetupMsg, StartExecutorMsg,
    TimerTriggered, ToolchainMap, Trigger,
};
use anyhow::Result;
use log::{debug, error, info, warn};
use minifaas_common::Environments;
use std::collections::HashMap;
use uuid::Uuid;
use xactor::*;

pub struct RuntimeController {
    environments: Environments,
    setup_map: ToolchainMap<BuildToolchain>,
    executors: HashMap<Uuid, Addr<FunctionExecutor>>,
    http_trigger: Addr<HttpTriggered>,
    timer_trigger: Addr<TimerTriggered>,
}

impl RuntimeController {
    pub fn new(
        existing_environments: Environments,
        toolchains: ToolchainMap<BuildToolchain>,
        http_trigger: Addr<HttpTriggered>,
        timer_trigger: Addr<TimerTriggered>,
    ) -> Self {
        RuntimeController {
            environments: existing_environments,
            setup_map: toolchains,
            executors: HashMap::default(),
            timer_trigger,
            http_trigger,
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
            _ => Ok(()),
        }
    }

    async fn unsubscribe_from_triggers(
        &self,
        name: &String,
        addr: Addr<FunctionExecutor>,
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
impl Handler<SetupMsg> for RuntimeController {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: SetupMsg) -> Result<()> {
        match self.environments.get_or_create(msg.env_id).await {
            Ok(env) => {
                info!("Found an environment for '{}'", msg.env_id);
                if let Some(toolchain) = self.setup_map.select_for_mut(&msg.toolchain) {
                    toolchain.pre_setup(env).await?;
                    toolchain.setup(env).await?;
                    toolchain.post_setup(env).await?;
                    info!("Setup complete for '{}'", msg.env_id);
                    Ok(())
                } else {
                    Err(anyhow::Error::msg(format!(
                        "Setup failed: no toolchain found for '{}'",
                        msg.toolchain
                    )))
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait::async_trait]
impl Handler<StartExecutorMsg> for RuntimeController {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: StartExecutorMsg) -> Result<()> {
        let env_id = msg.code.environment_id;
        match self.environments.get(&env_id).await {
            Some(env) => {
                if let Some(toolchain) = self.setup_map.select_executor(&msg.code.language()) {
                    let a = FunctionExecutor::new(env.clone(), msg.code.clone(), toolchain.clone())
                        .start()
                        .await?;

                    if let Some(existing) = self.executors.get(&env_id) {
                        existing.call(OpsMsg::Shutdown).await?;
                        self.unsubscribe_from_triggers(&msg.code.name(), existing.clone(), msg.code.trigger())
                            .await?;
                    }
                    self.executors.insert(env_id, a.clone());
                    self.subscribe_to_triggers(&msg, a.clone(), msg.code.trigger())
                        .await
                } else {
                    Err(anyhow::Error::msg(format!(
                        "Execute failed: no toolchain found for '{}",
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
                    self.unsubscribe_from_triggers(&msg.code.name(), existing.clone(), msg.code.trigger())
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
