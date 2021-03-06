use crate::runtime::RawFunctionInput;
use crate::{FunctionExecutor, HttpTriggerMsg, OpsMsg};
use anyhow::Result;
use chrono::{DateTime, Utc};
use cron::Schedule;
use futures::future::join_all;
use log::{debug, info, warn};
use minifaas_common::triggers::http::HttpTrigger;
use minifaas_common::triggers::http::HttpTriggerOutputs;
use minifaas_common::triggers::timer::TimerTrigger;
use std::collections::{BTreeMap, HashMap};
use std::ops::Bound::Included;
use std::time::Duration;
use xactor::*;

use super::IntervalTriggerMsg;

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
                method: _,
            } => {
                debug!("New route subscribed: {}", route);
                self.route_table.insert(route, addr)
            }
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
struct ScheduleAddr {
    pub addr: Addr<FunctionExecutor>,
    pub schedule: Schedule,
}

impl ScheduleAddr {
    pub fn next(&self) -> DateTime<Utc> {
        self.schedule.upcoming(Utc).next().unwrap()
    }

    pub fn id(&self) -> u64 {
        self.addr.actor_id()
    }
}

pub struct TimerTriggered {
    schedules: HashMap<u64, ScheduleAddr>,
    next: BTreeMap<DateTime<Utc>, Vec<Addr<FunctionExecutor>>>,
    resolution: Duration,
    since: DateTime<Utc>,
}

impl TimerTriggered {
    pub fn new(resolution: Duration) -> Self {
        TimerTriggered {
            schedules: HashMap::default(),
            next: BTreeMap::default(),
            resolution,
            since: Utc::now(),
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
        let triggered: Vec<DateTime<Utc>> = self
            .next
            .range((Included(&self.since), Included(&msg.when)))
            .map(|(k, _)| k.clone())
            .collect();

        let addrs: Vec<_> = triggered
            .iter()
            .filter_map(|t| self.next.remove(t))
            .flatten()
            .collect();

        let input: RawFunctionInput = msg.into();
        let tasks: Vec<_> = addrs.iter().map(|a| a.call(input.clone())).collect();

        let _ = join_all(tasks).await.into_iter().map(|r| match r {
            Ok(_) => info!("Timer trigger went through ok."),
            Err(e) => warn!("One of the timer triggers failed: {:?}", e),
        });
        let schedules = &self.schedules;
        let new_next: Vec<_> = addrs
            .iter()
            .filter_map(|a| schedules.get(&a.actor_id()))
            .collect();

        for sa in new_next {
            let next = sa.next();
            self.next
                .entry(next)
                .and_modify(|e| e.push(sa.addr.clone()))
                .or_insert(vec![sa.addr.clone()]);
        }
    }
}

#[async_trait::async_trait]
impl Handler<IntervalTriggerMsg> for TimerTriggered {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: IntervalTriggerMsg) {
        match msg {
            IntervalTriggerMsg::Subscribe { addr, schedule } => {
                let sa = ScheduleAddr {
                    addr: addr.clone(),
                    schedule,
                };

                let next = sa.next();
                self.schedules.entry(sa.id()).or_insert(sa);
                self.next
                    .entry(next)
                    .and_modify(|e| e.push(addr.clone()))
                    .or_insert(vec![addr]);
            }
            IntervalTriggerMsg::Unsubscribe { schedule, addr } => {
                let sa = ScheduleAddr {
                    addr: addr.clone(),
                    schedule,
                };
                let next = sa.next();
                match self.schedules.remove(&sa.id()) {
                    Some(_) => {
                        self.next.entry(next).and_modify(|e| {
                            if let Some(p) = e.iter().position(|a| a == &addr) {
                                e.remove(p);
                            }
                        });
                    }
                    _ => {}
                }
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
