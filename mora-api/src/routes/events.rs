use axum::{http::StatusCode, Json};
use log::{debug, error};
use mora_core::result::MoraError;
use serde::{Deserialize, Serialize};

use crate::QueuePoolState;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ScheduleEventRequest {
    data: String,

    schedule_rules: Vec<ScheduleRules>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ScheduleRules {
    schedule_at: u128,
    queue: String,
    recurring_options: Option<RecurringOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RecurringOptions {
    times: u128,
    delay: u128,
}

pub(crate) async fn schedule_event(
    queue_pool: QueuePoolState,
    schedule_event_request: Json<ScheduleEventRequest>,
) -> Result<(), (StatusCode, String)> {
    debug!(
        "Received schedule_event request: {:?}",
        &schedule_event_request
    );
    let mut queue_pool = queue_pool.lock().map_err(|e| {
        let e_msg = format!("error acquiring queue_pool lock {e}");
        error!("{e_msg}");
        (StatusCode::INTERNAL_SERVER_ERROR, e_msg)
    })?;

    let binary_data = schedule_event_request.data.clone().into_bytes();
    schedule_event_request
        .schedule_rules
        .clone()
        .into_iter()
        .map(|rule| {
            let queue_name = rule.queue.clone();
            let schedule_at = rule.schedule_at;
            let queue = queue_pool.get_queue_mut(&queue_name).map_err(|e| {
                if let MoraError::QueueNotFound(..) = e {
                    (
                        StatusCode::NOT_FOUND,
                        format!("{} queue does not exist", &queue_name),
                    )
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))
                }
            })?;

            queue
                .enqueue(schedule_at, binary_data.clone())
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")))
        })
        .collect::<Result<(), (StatusCode, String)>>()
}
