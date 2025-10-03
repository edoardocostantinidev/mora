use axum::{
    http::StatusCode,
    middleware,
    routing::{delete, get, post},
    Router,
};
use log::info;
use mora_channel::ChannelManager;
use mora_core::result::{MoraError, MoraResult};
use mora_queue::pool::QueuePool;
use mora_storage::wal_file_storage::WalFileStorage;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::Mutex;

use crate::{connections::Connections, middlewares::connections::ConnectionMiddleware};

pub(crate) mod connections;
pub(crate) mod middlewares;
pub(crate) mod routes;

pub type QueuePoolState = Arc<Mutex<QueuePool<WalFileStorage>>>;
pub type ChannelManagerState = Arc<Mutex<ChannelManager>>;
pub type ConnectionsState = Arc<Mutex<Connections>>;

#[derive(Clone)]
pub struct AppState {
    queue_pool: QueuePoolState,
    channel_manager: ChannelManagerState,
    connections: ConnectionsState,
}

pub struct MoraApi {
    port: u16,
}

impl MoraApi {
    pub fn new(port: u16) -> Self {
        MoraApi { port }
    }
    pub async fn start_listening(
        &self,
        channel_manager: Arc<Mutex<ChannelManager>>,
        queue_pool: Arc<Mutex<QueuePool<WalFileStorage>>>,
    ) -> MoraResult<()> {
        let connections = Arc::new(Mutex::new(Connections::default()));
        let app_state = AppState {
            channel_manager,
            queue_pool,
            connections: connections.clone(),
        };

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                connections.clone().lock().await.purge_old_connections();
            }
        });

        let app = Router::new()
            .route("/health", get(routes::health::get))
            .route("/queues", get(routes::queues::list_queues))
            .route("/queues/{queue_id}", get(routes::queues::get_queue))
            .route("/queues", post(routes::queues::create_queue))
            .route("/queues/{queue_id}", delete(routes::queues::delete_queue))
            .route("/events", post(routes::events::schedule_event))
            .route("/channels", post(routes::channels::create_channel))
            .route("/channels", get(routes::channels::list_channels))
            .route("/channels/{channel_id}", get(routes::channels::get_channel))
            .route(
                "/channels/{channel_id}",
                delete(routes::channels::delete_channel),
            )
            .route(
                "/channels/{channel_id}/events",
                get(routes::channels::get_channel_events),
            )
            .route(
                "/connections/info",
                get(routes::connections::get_connections_info),
            )
            .route_layer(middleware::from_extractor_with_state::<
                ConnectionMiddleware,
                AppState,
            >(app_state.clone()))
            .with_state(app_state);
        let addr: &SocketAddr = &format!("0.0.0.0:{}", self.port)
            .parse()
            .map_err(|e| MoraError::ApiError(format!("error parsing address: {e}")))?;
        info!("Starting API Server");

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| MoraError::ApiError(format!("error binding listener: {e}")))?;
        let service = app.into_make_service_with_connect_info::<SocketAddr>();

        axum::serve(listener, service)
            .await
            .map_err(|e| MoraError::ApiError(format!("error serving: {e}")))?;

        Ok(())
    }
}

pub(crate) fn handle_mora_error(error: MoraError) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
