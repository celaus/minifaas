use minifaas_common::ProgrammingLanguage;
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};

use minifaas_common::HttpMethod;
use minifaas_common::UserFunctionRecord;
use uuid::Uuid;
use xactor::*;

use async_std::sync::Arc;

mod function_executor;
mod runtime_controller;
mod triggered;
pub use function_executor::FunctionExecutor;
pub use runtime_controller::RuntimeController;
pub use triggered::{HttpTriggered, TimerTriggered};

#[message(result = "anyhow::Result<()>")]
pub struct SetupMsg {
    pub env_id: Uuid,
    pub toolchain: ProgrammingLanguage,
}

#[message(result = "anyhow::Result<()>")]
pub struct StartExecutorMsg {
    pub code: Arc<Box<UserFunctionRecord>>,
}

#[message(result = "anyhow::Result<()>")]
pub struct StopExecutorMsg {
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
        when: DateTime<Utc>,
        addr: Addr<FunctionExecutor>,
    },
}

#[message]
pub enum OpsMsg {
    Shutdown,
}
