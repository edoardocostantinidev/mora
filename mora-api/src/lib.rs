use std::sync::{Arc, Mutex};

use mora_core::context::MoraContext;
use rocket::{
    serde::{json::Json, Deserialize},
    State,
};

type MutableMoraContext = Arc<Mutex<MoraContext>>;

#[macro_use]
extern crate rocket;

#[get("/health")]
fn health() -> &'static str {
    "Healthy"
}

#[get("/context")]
fn context(state: &State<MutableMoraContext>) -> String {
    let context = state.lock().unwrap();
    format!("{:?}", context)
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct AddQueueRequest<'r> {
    queue_name: &'r str,
}

#[post("/queue", data = "<add_queue_request>")]
fn add_queue(
    add_queue_request: Json<AddQueueRequest<'_>>,
    state: &State<MutableMoraContext>,
) -> String {
    let mut context = state.lock().unwrap();
    context
        .add_queue(add_queue_request.queue_name.to_string())
        .unwrap();
    format!("{:?}", context)
}

#[derive(Debug, Default)]
pub struct MoraApi;
impl MoraApi {
    #[rocket::main]
    pub async fn start_listening() -> Result<(), rocket::Error> {
        let _rocket = rocket::build()
            .manage(MutableMoraContext::new(Mutex::new(MoraContext::default())))
            .mount("/", routes![health, context, add_queue])
            .launch()
            .await?;
        Ok(())
    }
}
