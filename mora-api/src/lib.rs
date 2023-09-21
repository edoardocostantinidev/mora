use axum::{
    http::StatusCode,
    routing::{delete, get, post},
    Router,
};
use log::info;
use mora_channel::ChannelManager;
use mora_core::result::{MoraError, MoraResult};
use mora_queue::pool::QueuePool;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub(crate) mod routes;

pub type QueuePoolState = Arc<Mutex<QueuePool>>;
pub type ChannelManagerState = Arc<Mutex<ChannelManager>>;

#[derive(Clone)]
pub struct AppState {
    queue_pool: QueuePoolState,
    channel_manager: ChannelManagerState,
}

pub struct MoraApi {
    port: u16,
}

impl MoraApi {
    pub fn new(port: u16) -> Self {
        MoraApi { port }
    }
    pub async fn start_listening(&self) -> MoraResult<()> {
        #[derive(OpenApi)]
        #[openapi(
        paths(
            routes::health::get,
        ),
        components(
            schemas()
        ),
        tags(
            (name = "mora", description = "Mora REST API")
        )
    )]
        struct ApiDoc;
        let app_state = AppState {
            channel_manager: Arc::new(Mutex::new(ChannelManager::default())),
            queue_pool: Arc::new(Mutex::new(QueuePool::new(None))),
        };

        let app = Router::new()
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .route("/health", get(routes::health::get))
            .route("/queues", get(routes::queues::get_queues))
            .route("/queues/:queue_id", get(routes::queues::get_queue))
            .route("/queues", post(routes::queues::post_queue))
            .route("/queues/:queue_id", delete(routes::queues::delete_queue))
            .route("/events", post(routes::events::schedule_event))
            .route("/channels", post(routes::channels::create_channel))
            .route("/channels", get(routes::channels::list_channels))
            .route("/channels/:channel_id", get(routes::channels::get_channel))
            .route(
                "/channels/:channel_id",
                delete(routes::channels::delete_channel),
            )
            .route(
                "/channels/:channel_id/events",
                get(routes::channels::get_channel_events),
            )
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
