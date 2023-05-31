use axum::{
    extract::FromRef,
    http::StatusCode,
    routing::{delete, get, post},
    Router,
};
use log::info;
use mora_channel::ChannelManager;
use mora_core::result::{MoraError, MoraResult};
use mora_queue::pool::QueuePool;
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock, TryLockError},
};

pub(crate) mod routes;

#[derive(Clone)]
pub struct AppState {
    queue_pool: QueuePool,
    channel_manager: ChannelManager,
}

pub type SharedState = Arc<RwLock<AppState>>;

pub struct MoraApi {
    port: u16,
}

impl MoraApi {
    pub fn new(port: u16) -> Self {
        MoraApi { port }
    }
    pub async fn start_listening(&self) -> MoraResult<()> {
        let app_state = Arc::new(RwLock::new(AppState {
            channel_manager: ChannelManager::default(),
            queue_pool: QueuePool::new(None),
        }));

        let app = Router::new()
            .route("/health", get(routes::health::get))
            .route("/queues", get(routes::queues::get_queues))
            .route("/queues/:queue_id", get(routes::queues::get_queue))
            .route("/queues", post(routes::queues::post_queue))
            .route("/queues/:queue_id", delete(routes::queues::delete_queue))
            .route("/events", post(routes::events::schedule_event))
            .route("/channels", post(routes::channels::create_channel))
            .route("/channels", get(routes::channels::list_channels))
            .with_state(app_state);
        let addr: &SocketAddr = &format!("0.0.0.0:{}", self.port)
            .parse()
            .map_err(|e| MoraError::ApiError(format!("error parsing address: {e}")))?;
        info!("Starting API Server");
        axum::Server::bind(addr)
            .serve(app.into_make_service())
            .await
            .map_err(|e| MoraError::ApiError(format!("error starting api server: {e}")))?;

        Ok(())
    }
}

pub(crate) fn handle_mora_error(error: MoraError) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub(crate) fn handle_rw_lock_error<T>(error: TryLockError<T>) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
