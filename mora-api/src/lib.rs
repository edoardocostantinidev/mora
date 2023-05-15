pub mod model;
pub mod routes;
pub mod services;

use rocket::{Config, Ignite, Rocket};
use routes::{channels, events, health, queues};

#[macro_use]
extern crate rocket;

#[derive(Debug, Default)]
pub struct MoraApi;
impl MoraApi {
    #[rocket::main]
    pub async fn start_listening() -> Result<Rocket<Ignite>, rocket::Error> {
        rocket::custom()
            .manage(queues::state())
            .manage(health::state())
            .mount("/health", health::all())
            .mount("/queues", queues::all())
            .mount("/events", events::all())
            .mount("/channels", channels::all())
            .launch()
            .await
    }
}
