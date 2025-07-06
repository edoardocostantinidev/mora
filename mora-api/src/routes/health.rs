use opentelemetry::{
    global,
    trace::{Span, Tracer},
};

/// Health Check to verify system integrity and functionality.
pub(crate) async fn get() -> &'static str {
    //span example
    let mut span = global::tracer("mora-api").start("health_check");
    log::info!("Health check endpoint hit"); // log example
    let meter = global::meter("mora-api");
    let health_check_endpoint_hits = meter.u64_counter("health_check_endpoint_hits").build();
    health_check_endpoint_hits.add(1, &[]); // metric example
    span.set_status(opentelemetry::trace::Status::Ok);
    span.end();

    "200 OK"
}
