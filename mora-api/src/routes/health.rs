/// Health Check to verify system integrity and functionality.
#[utoipa::path(
        get,
        path = "/health",
        responses(
            (status = 200, description= "System is UP and RUNNING.", body = String)
        )
    )]
pub(crate) async fn get() -> &'static str {
    "200 OK"
}
