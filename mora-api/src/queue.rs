use std::sync::{Arc, Mutex};

use mora_core::context::MoraContext;
use rocket::{
    response::{status::NotFound, Responder},
    serde::{json::Json, Deserialize, Serialize},
    Route, State,
};

type MutableMoraContext = Arc<Mutex<MoraContext>>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct AddQueueRequest<'r> {
    queue_name: &'r str,
}

#[derive(Responder, Serialize)]
#[response(status = 200, content_type = "json")]
#[serde(crate = "rocket::serde")]
struct GetQueueResponse {
    queue_name: String,
}

#[post("/", data = "<add_queue_request>")]
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

#[get("/<queue_name>", format = "json")]
fn get_queue(
    queue_name: String,
    state: &State<MutableMoraContext>,
) -> Result<Json<GetQueueResponse>, NotFound<String>> {
    let mut context = state.lock().unwrap();
    let queue = context.get_queue(&queue_name);
    if queue.is_none() {
        return Err(NotFound("no queue with that name".to_string()));
    }
    Ok(Json(GetQueueResponse {
        queue_name: queue_name.to_string(),
    }))
}

pub fn all() -> Vec<Route> {
    routes![add_queue, get_queue]
}

pub fn state() -> MutableMoraContext {
    MutableMoraContext::new(Mutex::new(MoraContext::default()))
}

#[cfg(test)]
mod tests {
    use crate::MoraApi;
    use rocket::{http::Status, local::blocking::Client};

    #[test]
    fn add_queue_adds_a_queue() {
        let client = Client::tracked(MoraApi::test_rocket()).expect("client error");
        let response = client
            .post("/queues")
            .body("{\"queue_name\":\"test\"}")
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let response = client.get("/queues/test").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string(),
            Some("{\"queue_name\":\"test\"}".to_string())
        );
    }
}
