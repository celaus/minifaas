use crate::ext::toolchain::ActiveToolchain;
use crate::logs::collectors::FileLogCollector;
use crate::logs::collectors::LogCollector;
use crate::output_parser::Parser;
use crate::output_parser::ReaderInput;
use crate::output_parser::STDOUT_PREFIX;
use minifaas_common::runtime::{RawFunctionInput, RawFunctionOutputWrapper};
use crate::OpsMsg;
use anyhow::Result;
use async_std::sync::Arc;
use log::{debug, error, info, warn};
use minifaas_common::Environment;
use minifaas_common::UserFunctionRecord;
use std::io::Cursor;
use xactor::*;

pub struct FunctionExecutor {
    environment: Environment,
    code: Arc<Box<UserFunctionRecord>>,
    toolchain: Arc<ActiveToolchain>,
    log_collector: Arc<FileLogCollector>,
}

impl FunctionExecutor {
    pub fn new(
        environment: Environment,
        code: Arc<Box<UserFunctionRecord>>,
        toolchain: Arc<ActiveToolchain>,
        log_collector: Arc<FileLogCollector>,
    ) -> Self {
        info!(
            "Function executor for {} started. Toolchain {:?}",
            code.name(),
            toolchain
        );
        FunctionExecutor {
            code,
            environment,
            toolchain,
            log_collector,
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
    ) -> Result<RawFunctionOutputWrapper> {
        let p = Parser::new(
            STDOUT_PREFIX.to_string(),
            vec![|v| hex::decode(v).ok(), |v| Some(v.as_bytes().to_vec())],
        );

        info!(
            "Running Function: '{}' with {:?}",
            self.code.name(),
            self.toolchain
        );
        let bytes = self.toolchain.build(&self.code.code().code).await?;
        info!("Built!");
        let stdout = self
            .toolchain
            .execute(bytes, msg, &self.environment)
            .await?;
        self.log_collector
            .collect(&stdout, &self.environment)
            .await?;
        let output = p.parse_to_map(Cursor::new(stdout))?;
        debug!("Function output: {:?}", output);
        Ok(RawFunctionOutputWrapper::from(output))
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
