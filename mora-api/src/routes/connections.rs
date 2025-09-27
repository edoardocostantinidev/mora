use axum::{extract::State, http::StatusCode, Json};
use mora_core::entities::connections_info::ConnectionsInfo;
use opentelemetry::{
    global,
    trace::{Span, Tracer},
};

use crate::AppState;

/// Get active connections
pub async fn get_connections_info(
    State(app_state): State<AppState>,
) -> Result<Json<ConnectionsInfo>, (StatusCode, String)> {
    let mut span = global::tracer("mora-api").start("connections_info");
    log::info!("Connections info endpoint hit");
    let meter = global::meter("mora-api");
    let connections_info_endpoint_hits =
        meter.u64_counter("connections_info_endpoint_hits").build();
    let connections = app_state.connections.lock().await.clients_connected();
    connections_info_endpoint_hits.add(1, &[]);
    span.set_status(opentelemetry::trace::Status::Ok);
    span.end();
    Ok(Json(ConnectionsInfo {
        clients_connected: connections,
    }))
}
