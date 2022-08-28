pub mod routes;
pub mod services;

#[cfg(test)]
use rocket::Build;
use rocket::{Ignite, Rocket};
use routes::{channels, events, health, queue};
use services::health::HealthService;

#[macro_use]
extern crate rocket;

#[derive(Debug, Default)]
pub struct MoraApi;
impl MoraApi {
    #[rocket::main]
    pub async fn start_listening() -> Result<Rocket<Ignite>, rocket::Error> {
        rocket::build()
            .manage(queue::state())
            .manage(health::state())
            .mount("/health", health::all())
            .mount("/queues", queue::all())
            .mount("/events", events::all())
            .mount("/channels", channels::all())
            .launch()
            .await
    }

    #[cfg(test)]
    #[launch]
    pub fn test_rocket() -> Rocket<Build> {
        rocket::build()
            .manage(queue::state())
            .mount("/health", health::all())
            .mount("/queues", queue::all())
            .mount("/events", events::all())
            .mount("/channels", channels::all())
    }
}
