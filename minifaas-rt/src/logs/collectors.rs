use anyhow::Result;
use async_std::fs::File;
use async_std::fs::OpenOptions;
use async_std::io::BufReader;
use async_std::io::BufWriter;
use async_std::io::Read;
use async_std::prelude::*;
use log::info;
use minifaas_common::Environment;

pub struct FileLogCollector {
    pub file_name: String,
}

impl FileLogCollector {
    pub fn new<S: Into<String>>(file_name: S) -> Self {
        FileLogCollector {
            file_name: file_name.into(),
        }
    }
}

#[async_trait::async_trait]
impl LogCollector for FileLogCollector {
    type Reader = BufReader<File>;

    async fn collect(&self, logs: &str, env: &Environment) -> Result<()> {
        let file_name = env.absolute_path(&self.file_name).await;
        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(&file_name)
            .await?;
        let mut writer = BufWriter::new(file);
        writer.write_all(logs.as_bytes()).await?;
        writer.flush().await?;
        info!("Wrote to {:?}: {}", file_name, logs);
        Ok(())
    }

    async fn reader(&self, env: &Environment) -> Result<Self::Reader> {
        let path = env.absolute_path(&self.file_name).await;
        Ok(BufReader::new(File::open(path).await?))
    }
}

#[async_trait::async_trait]
pub trait LogCollector {
    type Reader: Read;

    ///
    ///
    ///
    async fn collect(&self, logs: &str, env: &Environment) -> Result<()>;

    async fn reader(&self, env: &Environment) -> Result<Self::Reader>;
}
