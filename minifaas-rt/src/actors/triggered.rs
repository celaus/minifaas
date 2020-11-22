use crate::runtime::RawFunctionInput;
use crate::{FunctionExecutor, HttpTriggerMsg, OpsMsg, TimerTriggerMsg};
use anyhow::Result;
use async_std::sync::Arc;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use log::{debug, error, info, warn};
use minifaas_common::triggers::http::HttpTrigger;
use minifaas_common::triggers::http::HttpTriggerOutputs;
use minifaas_common::triggers::timer::TimerTrigger;
use std::collections::HashMap;
use std::convert::From;
use std::time::Duration;
use xactor::*;

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
                let inputs: RawFunctionInput = msg.into();
                match addr.call(inputs).await? {
                    Ok(output) => Ok(output.into()),
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
pub struct TimerTriggered {
    route_table: HashMap<DateTime<Utc>, Vec<Addr<FunctionExecutor>>>,
    resolution: Duration,
}

impl TimerTriggered {
    pub fn new(resolution: Duration) -> Self {
        TimerTriggered {
            route_table: HashMap::default(),
            resolution,
        }
    }
}

impl Default for TimerTriggered {
    fn default() -> Self {
        let interval = chrono::Duration::seconds(1).to_std().unwrap();
        TimerTriggered::new(interval)
    }
}

#[async_trait::async_trait]
impl Actor for TimerTriggered {
    async fn started(&mut self, ctx: &mut Context<Self>) -> anyhow::Result<()> {
        ctx.send_interval_with(|| TimerTrigger { when: Utc::now() }, self.resolution);
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<TimerTrigger> for TimerTriggered {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: TimerTrigger) {
        debug!("Triggering timer: {}", msg.when);
        let _result = match self.route_table.get(&msg.when) {
            Some(addrs) => {
                debug!(
                    "Found matching {} executors for '{}'",
                    addrs.len(),
                    msg.when
                );
                let input: RawFunctionInput = msg.into();
                let tasks = addrs.iter().map(|a| a.call(input.clone()));
                let _ = join_all(tasks).await.into_iter().map(|r| match r {
                    Ok(_) => info!("Timer trigger went through ok."),
                    Err(e) => warn!("One of the timer triggers failed: {:?}", e),
                });
            }
            None => {}
        };
    }
}

#[async_trait::async_trait]
impl Handler<TimerTriggerMsg> for TimerTriggered {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: TimerTriggerMsg) {
        match msg {
            TimerTriggerMsg::Subscribe { when, addr } => {
                self.route_table
                    .entry(when)
                    .and_modify(|e| e.push(addr.clone()))
                    .or_insert(vec![addr]);
            }
            TimerTriggerMsg::Unsubscribe { when, addr } => {
                self.route_table.get_mut(&when).map(|e| {
                    if let Some(p) = e.iter().position(|a| a == &addr) {
                        e.remove(p);
                    }
                });
            }
        };
    }
}

#[async_trait::async_trait]
impl Handler<OpsMsg> for TimerTriggered {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: OpsMsg) {
        match msg {
            OpsMsg::Shutdown => _ctx.stop(None),
        }
    }
}
