use crate::config::MoraConfig;
use mora_api::MoraApi;
use mora_core::result::{MoraError, MoraResult};
use simple_logger::SimpleLogger;

pub mod config;

#[derive(Debug, Default)]
pub struct Server {
    config: MoraConfig,
}

impl Server {
    pub fn new(config: Option<MoraConfig>) -> MoraResult<Self> {
        Ok(Self {
            config: config.unwrap_or_default(),
        })
    }

    pub async fn run(self) -> MoraResult<()> {
        MoraApi::new(self.config.port)
            .start_listening()
            .await
            .map_err(|e| MoraError::ApiError(e.to_string()))
    }
}

#[tokio::main]
async fn main() -> MoraResult<()> {
    SimpleLogger::new().init().unwrap();
    let config = MoraConfig::from_env();
    let server = Server::new(config.ok())?;
    server.run().await
}
