use axum::{extract::State, http::StatusCode, Json};
use mora_core::result::MoraError;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ScheduleEventRequest {
    data: String,
    schedule_rules: Vec<ScheduleRules>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ScheduleRules {
    schedule_for: u128,
    queue: String,
    recurring_options: Option<RecurringOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RecurringOptions {
    times: u128,
    delay: u128,
}

/// Schedule event POST.
pub(crate) async fn schedule_event(
    State(app_state): State<AppState>,
    schedule_event_request: Json<ScheduleEventRequest>,
) -> Result<(), (StatusCode, String)> {
    debug!(
        "Received schedule_event request: {:?}",
        &schedule_event_request
    );

    let binary_data = schedule_event_request.data.clone().into_bytes();
    for rule in schedule_event_request.schedule_rules.clone() {
        let queue_name = rule.queue.clone();
        let schedule_for = rule.schedule_for;
        let mut queue_pool = app_state.queue_pool.lock().await;
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
            .enqueue(schedule_for, binary_data.clone())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    Ok(())
}
