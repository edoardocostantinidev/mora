use rocket::{
    get,
    serde::{json::Json, Serialize},
    Route, State,
};

use crate::services::health::HealthCheckResult;
#[cfg_attr(test, mockall_double::double)]
use crate::services::health::HealthService;

use super::MutableMoraContext;

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
struct HealthCheckResponse {
    active: bool,
}

impl From<HealthCheckResult> for HealthCheckResponse {
    fn from(result: HealthCheckResult) -> Self {
        Self {
            active: result.active,
        }
    }
}

#[get("/")]
fn health(service: &State<HealthService>) -> Result<Json<HealthCheckResponse>, String> {
    service
        .check_system()
        .map(|h| {
            println!("{:?}", h);
            Json(h.into())
        })
        .map_err(|e| e.to_string())
}

pub fn all() -> Vec<Route> {
    routes![health]
}

pub fn state() -> HealthService {
    HealthService::new()
}

#[cfg(test)]
mod tests {

    use crate::services::health::MockHealthService;

    use super::*;

    use rocket::{http::Status, local::blocking::Client};

    #[test]
    fn active_system_responds_with_active_true() {
        let mut mock = MockHealthService::new();
        mock.expect_check_system()
            .return_once(|| Ok(HealthCheckResult { active: true }));

        let rocket = rocket::build().manage(mock).mount("/health", all());
        let client = Client::tracked(rocket).expect("client error");
        let response = client.get("/health").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string(),
            Some("{\"active\":true}".to_string())
        );
    }

    #[test]
    fn not_active_system_responds_with_active_false() {
        let mut mock = MockHealthService::new();
        mock.expect_check_system()
            .return_once(|| Ok(HealthCheckResult { active: false }));

        let rocket = rocket::build().manage(mock).mount("/health", all());
        let client = Client::tracked(rocket).expect("client error");
        let response = client.get("/health").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string(),
            Some("{\"active\":false}".to_string())
        );
    }
}
