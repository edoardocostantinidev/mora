use crate::config::MoraConfig;
use mora_api::MoraApi;
use mora_core::result::{MoraError, MoraResult};

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

    pub fn run(self) -> MoraResult<()> {
        dbg!(self.config);
        MoraApi::start_listening()
            .map(|_| ())
            .map_err(|e| MoraError::ApiError(e.to_string()))
    }
}

fn main() -> MoraResult<()> {
    let config = MoraConfig::from_env();
    let server = Server::new(config.ok())?;
    server.run()
}
