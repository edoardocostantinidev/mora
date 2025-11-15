use log::info;
use mora_core::result::{MoraError, MoraResult};
use mora_queue::{channel_manager::ChannelManager, pool::QueuePool};
use mora_storage::wal_file_storage::WalFileStorage;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::connections::Connections;

pub(crate) mod connections;
pub(crate) mod grpc;

pub type QueuePoolState = Arc<Mutex<QueuePool<WalFileStorage>>>;
pub type ChannelManagerState = Arc<Mutex<ChannelManager>>;
pub type ConnectionsState = Arc<Mutex<Connections>>;

pub struct MoraApi {
    port: u16,
}

impl MoraApi {
    pub fn new(port: u16) -> Self {
        MoraApi { port }
    }

    pub async fn start_grpc_server(
        &self,
        channel_manager: Arc<Mutex<ChannelManager>>,
        queue_pool: Arc<Mutex<QueuePool<WalFileStorage>>>,
    ) -> MoraResult<()> {
        use mora_proto::{
            channels::channel_service_server::ChannelServiceServer,
            connections::connection_service_server::ConnectionServiceServer,
            events::event_service_server::EventServiceServer,
            health::health_service_server::HealthServiceServer,
            queues::queue_service_server::QueueServiceServer,
        };

        let connections = Arc::new(Mutex::new(Connections::default()));

        // Spawn connection cleanup task
        let connections_clone = connections.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                connections_clone
                    .clone()
                    .lock()
                    .await
                    .purge_old_connections();
            }
        });

        let health_service = grpc::health::HealthServiceImpl;
        let queue_service = grpc::queues::QueueServiceImpl {
            queue_pool: queue_pool.clone(),
        };
        let channel_service = grpc::channels::ChannelServiceImpl {
            channel_manager: channel_manager.clone(),
            queue_pool: queue_pool.clone(),
        };
        let event_service = grpc::events::EventServiceImpl {
            queue_pool: queue_pool.clone(),
        };
        let connection_service = grpc::connections::ConnectionServiceImpl {
            connections: connections.clone(),
        };

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(mora_proto::FILE_DESCRIPTOR_SET)
            .include_reflection_service(true)
            .build_v1()
            .unwrap();

        let addr: std::net::SocketAddr = format!("0.0.0.0:{}", self.port)
            .parse()
            .map_err(|e| MoraError::ApiError(format!("error parsing address: {e}")))?;

        info!("Starting gRPC Server on {}", addr);

        tonic::transport::Server::builder()
            .add_service(HealthServiceServer::new(health_service))
            .add_service(QueueServiceServer::new(queue_service))
            .add_service(ChannelServiceServer::new(channel_service))
            .add_service(EventServiceServer::new(event_service))
            .add_service(ConnectionServiceServer::new(connection_service))
            .add_service(reflection_service)
            .serve(addr)
            .await
            .map_err(|e| MoraError::ApiError(format!("error serving gRPC: {e}")))?;

        Ok(())
    }
}
