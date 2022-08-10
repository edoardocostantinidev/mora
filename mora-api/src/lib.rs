#[cfg(test)]
use rocket::Build;
use rocket::{Ignite, Rocket};

#[macro_use]
extern crate rocket;

mod health;
mod queue;

#[derive(Debug, Default)]
pub struct MoraApi;
impl MoraApi {
    #[rocket::main]
    pub async fn start_listening() -> Result<Rocket<Ignite>, rocket::Error> {
        rocket::build()
            .manage(queue::state())
            .mount("/health", health::all())
            .mount("/queues", queue::all())
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
    }
}
