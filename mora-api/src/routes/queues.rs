#[cfg_attr(test, mockall_double::double)]
use crate::services::queues::QueueService;
use crate::{model::queue::Queue, routes::MutableMoraContext};
use rocket::{
    serde::{json::Json, Serialize},
    Route, State,
};

// #[derive(Serialize, Deserialize)]
// #[serde(crate = "rocket::serde")]
// struct AddQueueRequest<'r> {
//     queue_name: &'r str,
// }

// #[derive(Responder, Serialize)]
// #[response(status = 200, content_type = "json")]
// #[serde(crate = "rocket::serde")]
// struct GetQueueResponse {
//     queue: Queue,
// }

// #[post("/", data = "<add_queue_request>")]
// fn create_queue(
//     add_queue_request: Json<AddQueueRequest<'_>>,
//     state: &State<MutableMoraContext>,
// ) -> String {
//     let mut context = state.lock().unwrap();
//     context
//         .add_queue(add_queue_request.queue_name.to_string())
//         .unwrap();
//     format!("{:?}", context)
// }

// #[get("/<queue_name>", format = "json")]
// fn queue(
//     queue_name: String,
//     state: &State<MutableMoraContext>,
// ) -> Result<Json<GetQueueResponse>, NotFound<String>> {
//     let mut context = state.lock().unwrap();
//     let queue = context.get_queue(&queue_name);
//     if queue.is_none() {
//         return Err(NotFound("no queue with that name".to_string()));
//     }
//     Ok(Json(GetQueueResponse {
//         queue: queue_name.to_string(),
//     }))
// }

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct GetQueuesResponse {
    queues: Vec<Queue>,
}

#[get("/", format = "json")]
fn queues(
    context: &State<MutableMoraContext>,
    service: &State<QueueService>,
) -> Result<Json<GetQueuesResponse>, String> {
    let context = context
        .lock()
        .map_err(|e| format!("error retrieving context: {:?}", e))?;
    let queues = service
        .get_queues(&context)
        .map_err(|e| format!("error retrieving queues: {:?}", e))?;

    Ok(Json(GetQueuesResponse {
        queues: queues.into_iter().map(|q| q.into()).collect(),
    }))
}

// #[delete("/<queue_name>", format = "json")]
// fn delete_queue(queue_name: String, state: &State<MutableMoraContext>) -> String {
//     "".to_string()
// }

pub fn all() -> Vec<Route> {
    routes![
        //create_queue,
        //queue,
        queues,
        //elete_queue
    ]
}

pub fn state() -> MutableMoraContext {
    MutableMoraContext::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::queues::{MockQueueService, Queue};
    use rocket::{http::Status, local::blocking::Client};

    #[test]
    fn get_queues_returns_all_the_available_queues() {
        let mut mock = MockQueueService::new();
        mock.expect_get_queues().return_once(|_| {
            Ok(vec![
                Queue::new("Test1".to_string()),
                Queue::new("Test2".to_string()),
            ])
        });
        let context = MutableMoraContext::default();
        let rocket = rocket::build()
            .manage(context)
            .manage(mock)
            .mount("/queues", all());
        let client = Client::tracked(rocket).expect("client error");
        let response = client.get("/queues").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string(),
            Some("{\"queues\":[{\"name\":\"Test1\"},{\"name\":\"Test2\"}]}".to_string())
        );
    }
}
