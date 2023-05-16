use axum::{
    routing::{delete, get, post},
    Router,
};
use log::info;
use mora_core::result::{MoraError, MoraResult};
use mora_queue::pool::QueuePool;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub(crate) mod routes;
pub struct MoraApi {
    port: u16,
}

impl MoraApi {
    pub fn new(port: u16) -> Self {
        MoraApi { port }
    }
    pub async fn start_listening(&self) -> MoraResult<()> {
        let queue_pool = Arc::new(Mutex::new(QueuePool::new(None)));
        let app = Router::new()
            .route("/health", get(routes::health::get))
            .route("/queues", get(routes::queues::get_queues))
            .route("/queues/:queue_id", get(routes::queues::get_queue))
            .route("/queues", post(routes::queues::post_queue))
            .route("/queues/:queue_id", delete(routes::queues::delete_queue))
            .with_state(queue_pool);
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
