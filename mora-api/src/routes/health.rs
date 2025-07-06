use opentelemetry::global;
use tracing::{info, instrument};

/// Health Check to verify system integrity and functionality.
#[instrument]
pub(crate) async fn get() -> &'static str {
    // example of using tracing to log a message
    info!("performing health check");

    // example of using opentelemetry to track metrics
    let meter = global::meter("mora-api");
    let counter = meter.u64_counter("health_check_performed").build();
    counter.add(1, &[]);

    "200 OK"
}
