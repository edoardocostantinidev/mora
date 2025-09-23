use axum::{http::StatusCode, Json};
use mora_core::{
    clock::Clock,
    entities::cluster_status::{ClusterStatus, ClusterStatusData},
};
use opentelemetry::{
    global,
    trace::{Span, Tracer},
};

/// Health Check to verify system integrity and functionality.
pub(crate) async fn get() -> Result<Json<ClusterStatus>, (StatusCode, String)> {
    //span example
    let mut span = global::tracer("mora-api").start("health_check");
    log::info!("Health check endpoint hit"); // log example
    let meter = global::meter("mora-api");
    let health_check_endpoint_hits = meter.u64_counter("health_check_endpoint_hits").build();
    health_check_endpoint_hits.add(1, &[]); // metric example
    span.set_status(opentelemetry::trace::Status::Ok);
    span.end();

    Ok(Json(ClusterStatus::Online(ClusterStatusData {
        version: "1.0.0".to_string(),
        current_time_in_ns: Clock::now(),
    })))
}
