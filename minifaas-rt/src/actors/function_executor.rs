use crate::ext::toolchain::ActiveToolchain;
use crate::runtime::RawFunctionInput;
use crate::runtime::RawFunctionOutput;
use crate::OpsMsg;
use anyhow::Result;
use log::{debug, error, info, warn};
use minifaas_common::Environment;
use minifaas_common::UserFunctionRecord;
use xactor::*;

use async_std::sync::Arc;

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
        info!("Function executor for {} started. Toolchain {:?}", code.name(), toolchain);
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
        info!("Running Function: '{}' with {:?}", self.code.name(), self.toolchain);
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
