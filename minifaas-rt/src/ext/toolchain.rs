use std::sync::Arc;

use crate::ext::bash::Bash;
use crate::ext::bash::BashSetup;
use crate::ext::deno::Deno;
use crate::DenoSetup;
use anyhow::Result;
use minifaas_common::runtime::RawFunctionInput;
use minifaas_common::Environment;

#[derive(Debug, Clone)]
pub enum ActiveToolchain {
    Deno(Deno),
    Bash(Bash),
    Noop,
}

impl ActiveToolchain {
    pub async fn build(&self, code: &str) -> Result<Vec<u8>> {
        match self {
            ActiveToolchain::Deno(deno) => deno._build(code).await,
            ActiveToolchain::Bash(bash) => bash._build(code).await,

            _ => Ok(vec![]),
        }
    }

    pub async fn execute(
        &self,
        code: Vec<u8>,
        input: Arc<RawFunctionInput>,
        env: &Environment,
    ) -> Result<String> {
        match self {
            ActiveToolchain::Deno(deno) => {
                deno.pre_execute(input.clone()).await?;
                deno._execute(code, input, env).await
            }
            ActiveToolchain::Bash(bash) => {
                bash.pre_execute(input.clone()).await?;
                bash._execute(code, input, env).await
            }
            _ => Ok(String::new()),
        }
    }
}

impl Default for ActiveToolchain {
    fn default() -> Self {
        ActiveToolchain::Noop
    }
}

#[derive(Debug, Clone)]
pub enum BuildToolchain {
    Deno(DenoSetup),
    Bash(BashSetup),
    Noop,
}

#[async_trait::async_trait]
impl ToolchainSetup for BuildToolchain {
    async fn pre_setup(&mut self, env: &Environment) -> Result<()> {
        match self {
            BuildToolchain::Deno(d) => d.pre_setup(env).await,
            _ => Ok(()),
        }
    }

    async fn setup(&self, env: &Environment) -> Result<()> {
        match self {
            BuildToolchain::Deno(d) => d.setup(env).await,
            _ => Ok(()),
        }
    }

    async fn post_setup(&self, env: &Environment) -> Result<()> {
        match self {
            BuildToolchain::Deno(d) => d.post_setup(env).await,
            _ => Ok(()),
        }
    }
}

impl Default for BuildToolchain {
    fn default() -> Self {
        BuildToolchain::Noop
    }
}

#[async_trait::async_trait]
pub trait ToolchainSetup {
    ///
    /// Runs before the actual toolchain setup.
    ///
    async fn pre_setup(&mut self, _env: &Environment) -> Result<()> {
        Ok(())
    }

    ///
    /// The main toolchain setup.
    ///
    async fn setup(&self, _env: &Environment) -> Result<()>;

    ///
    /// Runs after the setup.
    ///
    async fn post_setup(&self, _env: &Environment) -> Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait ToolchainLifecycle {
    async fn pre_build(&self) -> Result<()> {
        Ok(())
    }

    async fn _build(&self, _code: &str) -> Result<Vec<u8>> {
        Ok(Vec::default())
    }

    async fn post_build(&self) -> Result<()> {
        Ok(())
    }

    async fn pre_execute(&self, _input: Arc<RawFunctionInput>) -> Result<()> {
        Ok(())
    }

    async fn _execute(
        &self,
        code: Vec<u8>,
        _input: Arc<RawFunctionInput>,
        env: &Environment,
    ) -> Result<String>;

    async fn post_execute(&self) -> Result<()> {
        Ok(())
    }
}
