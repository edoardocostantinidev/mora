use crate::config::MoraConfig;
use log::info;
use mora_api::MoraApi;
use mora_channel::ChannelManager;
use mora_core::result::MoraResult;
use mora_queue::pool::QueuePool;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinSet, time::sleep};

pub mod config;
pub mod otel;

#[derive(Debug)]
pub struct Server {
    config: MoraConfig,
}

impl Server {
    pub fn new(config: MoraConfig) -> Self {
        Self { config }
    }

    pub async fn run(self) -> MoraResult<()> {
        let mut tasks = JoinSet::new();
        let channel_manager = Arc::new(Mutex::new(ChannelManager::new()));
        let queue_pool = Arc::new(Mutex::new(QueuePool::new(
            self.config.queue_pool_capacity(),
        )));
        let api = MoraApi::new(self.config.port());
        let channel_manager_for_api = channel_manager.clone();
        let queue_pool_for_api = queue_pool.clone();
        tasks.spawn(async move {
            api.start_listening(channel_manager_for_api, queue_pool_for_api)
                .await
        });

        let channel_manager_for_checker = channel_manager.clone();
        tasks.spawn(async move {
            loop {
                sleep(Duration::from_millis(1)).await;
                let mut binding = channel_manager_for_checker.lock().await;

                binding
                    .get_mut_channels()?
                    .into_iter()
                    .for_each(|channel| channel.update_msec_from_last_op(1));

                let channels_to_delete: Vec<String> = binding
                    .get_channels()?
                    .into_iter()
                    .filter(|c| c.msec_from_last_op() > self.config.channel_timeout_in_msec())
                    .map(|c| c.id().to_owned())
                    .collect();
                if !channels_to_delete.is_empty() {
                    info!(
                        "Will close {} active channels due to inactivity.",
                        channels_to_delete.len()
                    )
                }
                for channel_id in channels_to_delete {
                    binding.close_channel(&channel_id.to_owned());
                }
            }
        });

        while let Some(_) = tasks.join_next().await {
            info!("Tasks completed");
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> MoraResult<()> {
    let config = MoraConfig::build()?;
    info!("Config built");
    info!("Starting OTEL");
    otel::init_otel()?;
    info!("OTEL initialized");
    info!("Starting mora-server");
    let server = Server::new(config);
    server.run().await
}
