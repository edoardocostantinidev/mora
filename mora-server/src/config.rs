use config::Config;
use mora_core::result::{MoraError, MoraResult};

#[derive(Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct MoraConfig {
    pub port: u16,
}

impl Default for MoraConfig {
    fn default() -> Self {
        Self { port: 2626 }
    }
}

impl MoraConfig {
    pub fn from_env() -> MoraResult<Self> {
        let env_config = Config::builder()
            .add_source(config::Environment::with_prefix("MORA").try_parsing(true))
            .build()
            .unwrap()
            .try_deserialize()
            .map_err(|e| MoraError::ConfigError(e.to_string()));

        if env_config.is_ok() {
            env_config
        } else {
            Ok(MoraConfig::default())
        }
    }
}
