use tracing::{instrument, trace};

/// Health Check to verify system integrity and functionality.
#[instrument]
pub(crate) async fn get() -> &'static str {
    trace!("performing health check");
    "200 OK"
}
