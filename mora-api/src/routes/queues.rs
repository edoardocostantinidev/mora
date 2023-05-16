use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use log::{debug, error};
use mora_queue::pool::QueuePool;
use std::sync::{Arc, Mutex};

type QueuePoolState = State<Arc<Mutex<QueuePool>>>;

#[derive(serde::Serialize)]
pub struct GetQueuesResponse {
    queues: Vec<GetQueueResponse>,
}

#[derive(Clone, serde::Serialize)]
pub struct GetQueueResponse {
    id: String,
    pending_events_count: usize,
}

pub async fn get_queues(queue_pool: QueuePoolState) -> Result<Json<GetQueuesResponse>, StatusCode> {
    debug!("Received get_queues request");
    let queue_pool = queue_pool.lock().map_err(|e| {
        error!("error acquiring queue_pool lock {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let queues: Vec<GetQueueResponse> = queue_pool
        .get_queues(regex::Regex::new(r".*").unwrap())
        .map_err(|e| {
            error!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .iter()
        .map(|q| GetQueueResponse {
            id: q.0.to_owned(),
            pending_events_count: 0,
        })
        .collect();

    Ok(Json(GetQueuesResponse { queues }))
}

pub async fn get_queue(
    queue_id: Path<String>,
    queue_pool: QueuePoolState,
) -> Result<Json<GetQueueResponse>, StatusCode> {
    debug!("Received get_queue request");
    let queue_pool = queue_pool.lock().map_err(|e| {
        error!("error acquiring queue_pool lock {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let queue: GetQueueResponse = queue_pool
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

pub async fn post_queue(
    queue_pool: QueuePoolState,
    queue_request: Json<PostQueueRequest>,
) -> Result<Json<GetQueueResponse>, (StatusCode, String)> {
    debug!("Received post_queues request: {:?}", &queue_request);
    let mut queue_pool = queue_pool.lock().map_err(|e| {
        error!("error acquiring queue_pool lock {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Cannot acquire lock on queue_pool".to_owned(),
        )
    })?;
    let id = queue_request.id.to_owned();
    queue_pool
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

pub async fn delete_queue(
    queue_pool: QueuePoolState,
    queue_id: Path<String>,
) -> Result<String, (StatusCode, String)> {
    debug!("Received delete_queues request: {:?}", &queue_id);
    let mut queue_pool = queue_pool.lock().map_err(|e| {
        error!("error acquiring queue_pool lock {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Cannot acquire lock on queue_pool".to_owned(),
        )
    })?;
    let queue_id = queue_id.0;
    let queue_id = queue_pool.delete_queue(queue_id).map_err(|e| {
        let e_msg = format!("error deleting queue: {:?}", e);
        error!("{e_msg}");
        (StatusCode::INTERNAL_SERVER_ERROR, e_msg)
    })?;
    Ok(format!("{queue_id} deleted"))
}
