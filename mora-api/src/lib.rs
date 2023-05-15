use axum::{routing::get, Router};
use log::info;
use mora_core::result::{MoraError, MoraResult};
use std::net::SocketAddr;

pub(crate) mod routes;
pub struct MoraApi {
    port: u16,
}

impl MoraApi {
    pub fn new(port: u16) -> Self {
        MoraApi { port }
    }
    pub async fn start_listening(&self) -> MoraResult<()> {
        let app = Router::new().route("/health", get(routes::health::get));
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
