use mora_core::result::MoraResult;
use tracing::{info, warn, Level};

const DEFAULT_PORT: u16 = 2626;
const DEFAULT_CHANNEL_TIMEOUT_IN_MSEC: usize = 3600 * 1000;
const DEFAULT_QUEUE_POOL_CAPACITY: usize = usize::MAX;

#[derive(Debug, PartialEq, Eq)]
pub struct MoraConfig {
    channel_timeout_in_msec: usize,
    port: u16,
    queue_pool_capacity: usize,
    log_level: Level,
}

impl MoraConfig {
    pub fn build() -> MoraResult<Self> {
        let port_var = std::env::var("MORA_PORT");
        let channel_timeout_in_msec_var = std::env::var("MORA_CHANNEL_TIMEOUT_IN_MSEC");
        let queue_pool_capacity_var = std::env::var("MORA_QUEUE_POOL_CAPACITY");

        let port = if let Ok(port_str) = port_var {
            port_str.parse().unwrap_or_else(|_| {
                warn!("{port_str} not a valid port number, reverting to default ({DEFAULT_PORT})");
                DEFAULT_PORT
            })
        } else {
            DEFAULT_PORT
        };

        let channel_timeout_in_msec = if let Ok(channel_timeout_in_msec_str) =
            channel_timeout_in_msec_var
        {
            channel_timeout_in_msec_str.parse().unwrap_or_else(|_| {
                warn!("{channel_timeout_in_msec_str} not a valid channel timeout number, reverting to default ({DEFAULT_CHANNEL_TIMEOUT_IN_MSEC})");
                DEFAULT_CHANNEL_TIMEOUT_IN_MSEC
            })
        } else {
            DEFAULT_CHANNEL_TIMEOUT_IN_MSEC
        };

        let queue_pool_capacity = if let Ok(queue_pool_capacity_str) = queue_pool_capacity_var {
            queue_pool_capacity_str.parse().unwrap_or_else(|_| {
                warn!("{queue_pool_capacity_str} not a valid queue pool capacity number, reverting to default ({DEFAULT_QUEUE_POOL_CAPACITY})");
                DEFAULT_QUEUE_POOL_CAPACITY
            })
        } else {
            DEFAULT_QUEUE_POOL_CAPACITY
        };

        let log_level = if let Ok(log_level_str) = std::env::var("MORA_LOG_LEVEL") {
            log_level_str.parse().unwrap_or_else(|_| {
                warn!("{log_level_str} not a valid log level, reverting to default info level)");
                Level::INFO
            })
        } else {
            info!("No log level provided, reverting to default info level");
            Level::INFO
        };

        Ok(Self {
            channel_timeout_in_msec,
            port,
            queue_pool_capacity,
            log_level,
        })
    }

    pub fn channel_timeout_in_msec(&self) -> usize {
        self.channel_timeout_in_msec
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn queue_pool_capacity(&self) -> usize {
        self.queue_pool_capacity
    }

    pub fn log_level(&self) -> Level {
        self.log_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        std::env::set_var("MORA_CHANNEL_TIMEOUT_IN_MSEC", "100");
        assert!(matches!(
            MoraConfig::build().unwrap(),
            MoraConfig {
                channel_timeout_in_msec: 100,
                ..
            }
        ));
        std::env::remove_var("MORA_CHANNEL_TIMEOUT_IN_MSEC");
    }
}
