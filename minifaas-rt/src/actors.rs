use crate::ext::toolchain::ActiveToolchain;
use crate::ext::toolchain::BuildToolchain;
use crate::ext::toolchain::ToolchainSetup;
use crate::runtime::RawFunctionInput;
use crate::runtime::RawFunctionOutput;
use crate::ProgrammingLanguage;
use crate::ToolchainMap;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use minifaas_common::triggers::http::HttpTrigger;
use minifaas_common::triggers::http::HttpTriggerOutputs;
use minifaas_common::Environment;
use minifaas_common::Environments;
use minifaas_common::FunctionCode;
use minifaas_common::HttpMethod;
use minifaas_common::Trigger;
use minifaas_common::UserFunctionRecord;
use std::collections::HashMap;
use uuid::Uuid;
use xactor::*;

use async_std::sync::Arc;

#[message(result = "anyhow::Result<()>")]
pub struct SetupMsg {
    pub env_id: Uuid,
    pub toolchain: ProgrammingLanguage,
}

#[message(result = "anyhow::Result<()>")]
pub struct StartExecutorMsg {
    pub code: Arc<Box<UserFunctionRecord>>,
}

#[message]
#[derive(Clone)]
pub enum HttpTriggerMsg {
    Subscribe {
        route: String,
        addr: Addr<FunctionExecutor>,
        method: HttpMethod,
    },
    Unsubscribe {
        route: String,
    },
}

#[message]
pub enum TimerTriggerMsg {
    Subscribe {
        when: DateTime<Utc>,
        addr: Addr<FunctionExecutor>,
    },
    Unsubscribe {
        addr: Addr<FunctionExecutor>,
    },
}

#[message]
pub struct SubscribeToIntervalTrigger {
    when: Duration,
    addr: Addr<FunctionExecutor>,
}

#[message]
pub enum OpsMsg {
    Shutdown,
}

// ---------------------------------

pub struct FunctionExecutor {
    environment: Environment,
    code: Arc<Box<UserFunctionRecord>>,
    toolchain: Arc<ActiveToolchain>,
}

impl FunctionExecutor {
    pub fn new(
        environment: Environment,
        code: Arc<Box<UserFunctionRecord>>,
        toolchain: Arc<ActiveToolchain>,
    ) -> Self {
        FunctionExecutor {
            code,
            environment,
            toolchain,
        }
    }
}

#[async_trait::async_trait]
impl Actor for FunctionExecutor {
    // async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
    // }
}

#[async_trait::async_trait]
impl Handler<RawFunctionInput> for FunctionExecutor {
    async fn handle(
        &mut self,
        _ctx: &mut Context<Self>,
        msg: RawFunctionInput,
    ) -> Result<RawFunctionOutput> {
        info!("Running Function: '{}'", self.code.name());
        let bytes = self.toolchain.build(&self.code.code().code).await?;
        info!("Built!");
        let output = self
            .toolchain
            .execute(bytes, msg, &self.environment)
            .await
            .map(RawFunctionOutput::from);
        debug!("Function output: {:?}", output);
        output
    }
}

#[async_trait::async_trait]
impl Handler<OpsMsg> for FunctionExecutor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: OpsMsg) {
        match msg {
            OpsMsg::Shutdown => _ctx.stop(None),
        }
    }
}

// ---------------------------------

#[derive(Default)]
pub struct HttpTriggered {
    route_table: HashMap<String, Addr<FunctionExecutor>>,
}

impl HttpTriggered {
    pub fn new() -> Self {
        HttpTriggered {
            route_table: HashMap::default(),
        }
    }
}

#[async_trait::async_trait]
impl Actor for HttpTriggered {
    //    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {}
}

#[async_trait::async_trait]
impl Handler<HttpTrigger> for HttpTriggered {
    async fn handle(
        &mut self,
        _ctx: &mut Context<Self>,
        msg: HttpTrigger,
    ) -> Result<HttpTriggerOutputs> {
        debug!("Triggering route: {}", msg.route);
        let result = match self.route_table.get(&msg.route) {
            Some(addr) => {
                debug!("Found matching executor for '{}'", msg.route);
                match addr.call(RawFunctionInput::from(msg)).await? {
                    Ok(mut output) => Ok(output.into_http()),
                    _ => Err(Error::msg("Nothing found")),
                }
            }
            None => Ok(HttpTriggerOutputs::default()),
        };
        result
    }
}

#[async_trait::async_trait]
impl Handler<HttpTriggerMsg> for HttpTriggered {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: HttpTriggerMsg) {
        match msg {
            HttpTriggerMsg::Subscribe {
                route,
                addr,
                method,
            } => self.route_table.insert(route, addr),
            HttpTriggerMsg::Unsubscribe { route } => self.route_table.remove(&route),
        };
    }
}

#[async_trait::async_trait]
impl Handler<OpsMsg> for HttpTriggered {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: OpsMsg) {
        match msg {
            OpsMsg::Shutdown => _ctx.stop(None),
        }
    }
}

// ---------------------------------
#[derive(Default)]
pub struct TimerTriggered {
    route_table: HashMap<DateTime<Utc>, Vec<Addr<FunctionExecutor>>>,
}

impl TimerTriggered {
    pub fn new() -> Self {
        TimerTriggered {
            route_table: HashMap::default(),
        }
    }
}

#[async_trait::async_trait]
impl Actor for TimerTriggered {}

// #[async_trait::async_trait]
// impl Handler<Timer> for TimerTriggered {
//     async fn handle(
//         &mut self,
//         _ctx: &mut Context<Self>,
//         msg: HttpTrigger,
//     ) -> xactor::Result<HttpTriggerOutputs> {
//         match self.route_table.get(&msg.route) {
//             Some(addr) => {
//                 //addr.send(msg)?.await;
//             }
//             None => {}
//         }
//         HttpTriggerOutputs::default()
//     }
// }

// #[async_trait::async_trait]
// impl Handler<SubscribeToTimerTrigger> for TimerTriggered {
//     async fn handle(&mut self, _ctx: &mut Context<Self>, msg: SubscribeToTimerTrigger) {
//         self.route_table.insert(msg.when, msg.addr);
//     }
// }

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

    async fn unsubscribe_to_triggers(
        &self,
        msg: &StartExecutorMsg,
        addr: Addr<FunctionExecutor>,
        trigger: Trigger,
    ) -> Result<()> {
        match trigger {
            Trigger::Http(_) => {
                let sub = HttpTriggerMsg::Unsubscribe {
                    route: msg.code.name().clone(),
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
                        self.unsubscribe_to_triggers(&msg, existing.clone(), msg.code.trigger())
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
impl Handler<OpsMsg> for RuntimeController {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: OpsMsg) {
        match msg {
            OpsMsg::Shutdown => _ctx.stop(None),
        }
    }
}
