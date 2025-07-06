use crate::{handle_mora_error, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use tracing::{debug, error};

#[derive(serde::Serialize)]
pub struct GetQueuesResponse {
    queues: Vec<GetQueueResponse>,
}

#[derive(Clone, serde::Serialize)]
pub struct GetQueueResponse {
    id: String,
    pending_events_count: usize,
}

/// List all queues.
#[axum_macros::debug_handler]
pub async fn get_queues(
    State(app_state): State<AppState>,
) -> Result<Json<GetQueuesResponse>, (StatusCode, String)> {
    debug!("Received get_queues request");

    let queues: Vec<GetQueueResponse> = app_state
        .queue_pool
        .lock()
        .await
        .get_queues(regex::Regex::new(r".*").unwrap())
        .map_err(handle_mora_error)?
        .iter()
        .map(|q| GetQueueResponse {
            id: q.0.to_owned(),
            pending_events_count: 0,
        })
        .collect();

    Ok(Json(GetQueuesResponse { queues }))
}

/// Get informations about a queue.
#[axum_macros::debug_handler]
pub async fn get_queue(
    queue_id: Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<GetQueueResponse>, StatusCode> {
    debug!("Received get_queue request");

    let queue: GetQueueResponse = app_state
        .queue_pool
        .lock()
        .await
        .get_queues(regex::Regex::new(&queue_id.0).unwrap())
        .map_err(|e| {
            error!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .iter()
        .map(|q| GetQueueResponse {
            id: q.0.to_owned(),
            pending_events_count: 0,
        })
        .collect::<Vec<GetQueueResponse>>()
        .first()
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();

    Ok(Json(queue))
}

#[derive(Debug, serde::Deserialize)]
pub struct PostQueueRequest {
    id: String,
}

/// Creates a queue.
pub async fn create_queue(
    State(app_state): State<AppState>,
    queue_request: Json<PostQueueRequest>,
) -> Result<Json<GetQueueResponse>, (StatusCode, String)> {
    debug!("Received create_queues request: {:?}", &queue_request);

    let id = queue_request.id.to_owned();
    app_state
        .queue_pool
        .lock()
        .await
        .create_queue(id.to_owned())
        .map_err(|e| {
            error!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))
        })
        .map(|_| {
            Json(GetQueueResponse {
                id: id.to_owned(),
                pending_events_count: 0,
            })
        })
}

/// Deletes a queue.
pub async fn delete_queue(
    State(app_state): State<AppState>,
    queue_id: Path<String>,
) -> Result<String, (StatusCode, String)> {
    debug!("Received delete_queues request: {:?}", &queue_id);

    let queue_id = queue_id.0;
    let queue_id = app_state
        .queue_pool
        .lock()
        .await
        .delete_queue(queue_id)
        .map_err(|e| {
            let e_msg = format!("error deleting queue: {:?}", e);
            error!("{e_msg}");
            (StatusCode::INTERNAL_SERVER_ERROR, e_msg)
        })?;
    Ok(format!("{queue_id} deleted"))
}
